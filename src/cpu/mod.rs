pub mod opcodes;
pub mod registers;

use super::memory::Memory;
use registers::{Flag, Register16, Register8, Registers};

use Flag::*;
use Register16::*;
use Register8::*;

pub struct CPU {
  registers: Registers,
  ram: Memory,
}

#[derive(Debug, Copy, Clone)]
pub enum Arg {
  R8(Register8),
  R16(Register16),
  N8(u8),
  N16(u16),
  Ptr(usize),
  Ptr_R16(Register16),
}

use Arg::*;

impl CPU {
  pub fn new() -> CPU {
    CPU {
      registers: Registers::new(),
      ram: Memory::new(8 * 1024),
    }
  }

  // executes the next instruction referenced by PC
  pub fn exec(&mut self) {
    let current_pc = self.registers.read16(PC);

    let byte = self.ram.read8(current_pc as usize);
    let new_pc = self.exec_opcode(byte, current_pc);

    self.registers.write16(PC, new_pc);
  }

  // executes the given opcode
  fn exec_opcode(&mut self, opcode: u8, pc: u16) -> u16 {
    match opcode {
      // NOP, do nothing
      0x00 => pc + 1,

      // LD (N), SP
      0x08 => {
        self.exec_ld(Ptr(self.read_arg16() as usize), R16(SP));

        pc + 3
      }

      // LD R, N
      _ if opcode_match(opcode, 0b_1100_1111, 0b0000_0001) => {
        let reg = R16(self.read_reg16((opcode & 0x30) >> 4, 0));
        let val = N16(self.read_arg16());

        self.exec_ld(reg, val);
        pc + 3
      }

      // ADD HL, R
      _ if opcode_match(opcode, 0b1100_1111, 0b0000_1001) => {
        let reg1 = R16(HL);
        let reg2 = R16(self.read_reg16((opcode & 0x30) >> 4, 0));
        self.exec_add(reg1, reg2);
        self.registers.set_flag(NF, 0);

        pc + 1
      }

      // LD (R), A
      _ if opcode_match(opcode, 0b1110_1111, 0b0000_0010) => {
        let arg1 = Ptr_R16(self.read_reg16((opcode & 0x30) >> 4, 1));
        // let arg1 = Ptr_R16(reg1); self.registers.read16(reg1) as usize);
        let arg2 = R8(A);
        self.exec_ld(arg1, arg2);

        pc + 1
      }

      // LD A, (R)
      _ if opcode_match(opcode, 0b1110_1111, 0b0000_1010) => {
        let arg1 = R8(A);
        let reg2 = self.read_reg16((opcode & 0x30) >> 4, 1);
        let arg2 = Ptr(self.registers.read16(reg2) as usize);
        self.exec_ld(arg1, arg2);

        pc + 1
      }

      // INC R
      _ if opcode_match(opcode, 0b1100_1111, 0b0000_0011) => {
        let arg = R16(self.read_reg16((opcode & 0x30) >> 4, 0));
        self.exec_inc(arg, 1);

        pc + 1
      }

      // DEC R
      _ if opcode_match(opcode, 0b1100_1111, 0b0000_1011) => {
        let arg = R16(self.read_reg16((opcode & 0x30) >> 4, 0));
        self.exec_inc(arg, -1);

        pc + 1
      }

      // INC D
      _ if opcode_match(opcode, 0b1100_0111, 0b0000_0100) => {
        let arg = self.read_reg8((opcode & 0x38) >> 3);
        self.exec_inc(arg, 1);

        pc + 1
      }

      // DEC D
      _ if opcode_match(opcode, 0b1100_0111, 0b0000_0101) => {
        let arg = self.read_reg8((opcode & 0x38) >> 3);
        self.exec_inc(arg, -1);

        pc + 1
      }

      // LD D, N
      _ if opcode_match(opcode, 0b1100_0111, 0b0000_0110) => {
        let reg = self.read_reg8((opcode & 0x38) >> 3);
        let val = N8(self.read_arg8());
        self.exec_ld(reg, val);

        pc + 2
      }
      // TODO

      // RdCA
      _ if opcode_match(opcode, 0b1111_0111, 0b0000_0111) => pc + 1,

      // RdA
      _ if opcode_match(opcode, 0b1111_0111, 0b0001_0111) => pc + 1,

      // STOP
      0x10 => pc + 1,

      // JR N
      0x18 => pc + 2,

      // JR F, N
      _ if opcode_match(opcode, 0b1110_0111, 0b0010_0000) => pc + 2,

      // LDI (HL), A
      0x22 => pc + 1,

      // LDI A, (HL)
      0x2a => pc + 1,

      // LDD (HL), A
      0x32 => pc + 1,

      // LDD A, (HL)
      0x3a => pc + 1,

      // DAA
      0x27 => pc + 1,

      // CPL
      0x2f => pc + 1,

      // SCF
      0x37 => pc + 1,

      // CCF
      0x3f => pc + 1,

      // LD D, D
      _ if opcode_match(opcode, 0b1100_0000, 0b0100_0000) => pc + 1,

      // HALT
      0x74 => pc + 1,

      // ALU A, D
      _ if opcode_match(opcode, 0b1100_0000, 0b1000_0000) => pc + 1,

      // ALU A, N
      _ if opcode_match(opcode, 0b1100_0111, 0b1100_0110) => pc + 2,

      // POP R
      _ if opcode_match(opcode, 0b1100_1111, 0b1100_0001) => pc + 2,

      // PUSH R
      _ if opcode_match(opcode, 0b1100_1111, 0b1100_0101) => pc + 2,

      // RST N
      _ if opcode_match(opcode, 0b1100_0111, 0b1100_0111) => pc + 1,

      // RET F
      _ if opcode_match(opcode, 0b1110_0111, 0b1100_0000) => pc + 1,

      // RET
      0xc9 => pc + 1,

      // RETI
      0xd9 => pc + 1,

      // JP F, N
      _ if opcode_match(opcode, 0b1110_0111, 0b1100_0010) => pc + 3,

      // JP N
      0xc3 => pc + 3,

      // CALL F, N
      _ if opcode_match(opcode, 0b1110_0111, 0b1100_0100) => pc + 3,

      // CALL N
      0xcd => pc + 3,

      // ADD SP, N
      0xe8 => pc + 2,

      // LD HL, SP + N
      0xf8 => pc + 2,

      // LD (FF00+N), A
      0xe0 => pc + 2,

      // LD A, (FF00+N)
      0xf0 => pc + 2,

      // LD (C), A
      0xe2 => pc + 1,

      // LD A, (C)
      0xf2 => pc + 1,

      // LD (N), A
      0xe6 => pc + 3,

      // LD A, (N)
      0xf6 => pc + 3,

      // JP HL
      0xe9 => pc + 1,
      // LD SP, HL
      0xf9 => pc + 1,
      // DI
      0xf3 => pc + 1,
      // EI
      0xfb => pc + 1,

      // read instr from byte 2
      0xcb => pc + 2,

      _ => self.i_unknown(opcode),
    }
  }

  fn i_nop(&self) -> u16 {
    self.registers.read16(PC) + 1
  }

  fn exec_ld(&mut self, dest: Arg, orig: Arg) {
    match (dest.clone(), orig) {
      (Ptr(addr), R16(reg)) => self.ram.write16(addr, self.registers.read16(reg)),
      (R16(reg), N16(val)) => self.registers.write16(reg, val),
      (R8(reg), N8(val)) => self.registers.write8(reg, val),
      (Ptr(addr), R8(reg)) => self.ram.write8(addr, self.registers.read8(reg)),
      (R8(reg), Ptr(addr)) => self.registers.write8(reg, self.ram.read8(addr)),
      (Ptr_R16(reg1), R8(reg2)) => self.ram.write8(
        self.registers.read16(reg1) as usize,
        self.registers.read8(reg2),
      ),
      (Ptr_R16(reg), N8(val)) => self.ram.write8(self.registers.read16(reg) as usize, val),

      _ => panic!("Can't handle LD opcode arguments {:?}", (dest, orig)),
    };
  }

  fn exec_add(&mut self, dest: Arg, orig: Arg) {
    match (dest, orig) {
      (R16(reg1), R16(reg2)) => self.registers.write16(
        reg1,
        self.registers.read16(reg1) + self.registers.read16(reg2),
      ),

      _ => panic!("Can't handle ADD opcode arguments {:?}", (dest, orig)),
    }
  }

  fn exec_inc(&mut self, dest: Arg, inc: i16) {
    match (dest, inc) {
      (R16(reg), 1) => self.registers.write16(reg, self.registers.read16(reg) + 1),

      (R16(reg), -1) => self.registers.write16(reg, self.registers.read16(reg) - 1),

      _ => panic!("Can't handle INC/DEC opcode argument {:?}", dest),
    }
  }

  fn i_unknown(&self, opcode: u8) -> u16 {
    panic!(
      "Failed to execute unknown opcode: 0x{:02x} (0b{0:b})",
      opcode
    );
  }

  fn exec_arg8(&self) -> u8 {
    let pc = self.registers.read16(PC);

    self.ram.read8((pc + 1) as usize)
  }

  fn read_arg16(&self) -> u16 {
    let pc = self.registers.read16(PC);

    self.ram.read16((pc + 1) as usize)
  }

  fn read_arg8(&self) -> u8 {
    let pc = self.registers.read16(PC);

    self.ram.read8((pc + 1) as usize)
  }

  fn read_reg16(&self, reg: u8, mode: u8) -> Register16 {
    match (reg, mode) {
      (0x0, _) => BC,
      (0x1, _) => DE,
      (0x2, 0) => HL,
      (0x3, 0) => SP,

      _ => panic!("Unkonwn R16 code: 0x{:x} (mode: {})", reg, mode),
    }
  }

  fn read_reg8(&self, reg: u8) -> Arg {
    match reg {
      0x0 => R8(B),
      0x1 => R8(C),
      0x2 => R8(D),
      0x3 => R8(E),
      0x4 => R8(H),
      0x5 => R8(L),
      0x6 => Ptr_R16(HL),
      0x7 => R8(A),

      _ => panic!("Unkonwn R16 code: 0x{:x}", reg),
    }
  }
}

fn opcode_match(opcode: u8, mask: u8, expectation: u8) -> bool {
  opcode & mask == expectation
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn new_cpu() {
    let cpu = CPU::new();

    assert_eq!(cpu.registers.read16(Register16::AF), 0);
  }

  #[test]
  fn exec_known_opcodes() {
    for opcode in 0x00..0xff {
      let mut cpu = CPU::new();

      cpu.ram.write8(0, opcode);

      cpu.exec();
    }
  }

  #[test]
  fn opcode_nop() {
    let mut cpu = CPU::new();

    cpu.ram.write8(0, 0b00000000);

    cpu.exec();

    assert_eq!(cpu.registers.read16(Register16::PC), 1);
  }

  #[test]
  fn opcode_ld_ptr16_sp() {
    let mut cpu = CPU::new();
    cpu.registers.write16(SP, 2047);

    cpu.ram.write8(0, 0b0000_1000);
    cpu.ram.write16(1, 511);

    cpu.exec();

    assert_eq!(cpu.ram.read16(511), 2047);
  }

  #[test]
  fn opcode_ld_r16_n16() {
    let mut cpu = CPU::new();

    // LD BC, 511
    cpu.ram.write8(0, 0x01);
    cpu.ram.write16(1, 511);

    // LD DE, 1023
    cpu.ram.write8(3, 0x11);
    cpu.ram.write16(4, 1023);

    // LD HL, 2047
    cpu.ram.write8(6, 0x21);
    cpu.ram.write16(7, 2047);

    // LD SP, 4095
    cpu.ram.write8(9, 0x31);
    cpu.ram.write16(10, 4095);

    // exec all 4 instructions
    cpu.exec();
    cpu.exec();
    cpu.exec();
    cpu.exec();

    assert_eq!(cpu.registers.read16(BC), 511);
    assert_eq!(cpu.registers.read16(DE), 1023);
    assert_eq!(cpu.registers.read16(HL), 2047);
    assert_eq!(cpu.registers.read16(SP), 4095);
  }

  #[test]
  fn opcode_add_hl_r16() {
    let mut cpu = CPU::new();

    // ADD HL, BC
    cpu.registers.write16(HL, 128);
    cpu.registers.write16(BC, 127);
    cpu.ram.write8(0, 0x09);
    cpu.exec();
    assert_eq!(cpu.registers.read16(HL), 255);

    // ADD HL, DE
    cpu.registers.write16(HL, 256);
    cpu.registers.write16(DE, 255);
    cpu.ram.write8(1, 0x19);
    cpu.exec();
    assert_eq!(cpu.registers.read16(HL), 511);

    // ADD HL, HL
    cpu.registers.write16(HL, 511);
    cpu.ram.write8(2, 0x29);
    cpu.exec();
    assert_eq!(cpu.registers.read16(HL), 1022);

    // ADD HL, SP
    cpu.registers.write16(HL, 1024);
    cpu.registers.write16(SP, 1023);
    cpu.ram.write8(3, 0x39);
    cpu.exec();
    assert_eq!(cpu.registers.read16(HL), 2047);
  }

  #[test]
  fn opcode_add_hl_r16_flags() {
    panic!("Not yet implemented")
  }

  #[test]
  fn opcode_ld_r16_a() {
    let mut cpu = CPU::new();

    // LD BC, A
    cpu.registers.write8(A, 127);
    cpu.registers.write16(BC, 1024);
    cpu.ram.write8(0, 0x02);
    cpu.exec();
    assert_eq!(cpu.ram.read8(1024), 127);

    // LD DE, A
    cpu.registers.write8(A, 63);
    cpu.registers.write16(DE, 150);
    cpu.ram.write8(1, 0x12);
    cpu.exec();
    assert_eq!(cpu.ram.read8(150), 63);
  }

  #[test]
  fn opcode_ld_a_r16() {
    let mut cpu = CPU::new();

    // LD BC, A
    cpu.ram.write8(1024, 127);
    cpu.registers.write16(BC, 1024);
    cpu.ram.write8(0, 0x0a);
    cpu.exec();
    assert_eq!(cpu.registers.read8(A), 127);

    // LD DE, A
    cpu.ram.write8(150, 63);
    cpu.registers.write16(DE, 150);
    cpu.ram.write8(1, 0x1a);
    cpu.exec();
    assert_eq!(cpu.registers.read8(A), 63);
  }

  #[test]
  fn opcode_inc_r16() {
    let mut cpu = CPU::new();

    // INC BC
    cpu.registers.write16(BC, 257);
    cpu.ram.write8(0, 0x03);
    cpu.exec();
    assert_eq!(cpu.registers.read16(BC), 258);

    // INC DE
    cpu.registers.write16(DE, 511);
    cpu.ram.write8(1, 0x13);
    cpu.exec();
    assert_eq!(cpu.registers.read16(DE), 512);

    // INC HL
    cpu.registers.write16(HL, 1023);
    cpu.ram.write8(2, 0x23);
    cpu.exec();
    assert_eq!(cpu.registers.read16(HL), 1024);

    // INC SP
    cpu.registers.write16(SP, 2047);
    cpu.ram.write8(3, 0x33);
    cpu.exec();
    assert_eq!(cpu.registers.read16(SP), 2048);
  }

  #[test]
  fn opcode_inc_r16_flags() {
    panic!("Not implemented yet");
  }

  #[test]
  fn opcode_dec_r16() {
    let mut cpu = CPU::new();

    // INC BC
    cpu.registers.write16(BC, 257);
    cpu.ram.write8(0, 0x0b);
    cpu.exec();
    assert_eq!(cpu.registers.read16(BC), 256);

    // INC DE
    cpu.registers.write16(DE, 511);
    cpu.ram.write8(1, 0x1b);
    cpu.exec();
    assert_eq!(cpu.registers.read16(DE), 510);

    // INC HL
    cpu.registers.write16(HL, 1023);
    cpu.ram.write8(2, 0x2b);
    cpu.exec();
    assert_eq!(cpu.registers.read16(HL), 1022);

    // INC SP
    cpu.registers.write16(SP, 2047);
    cpu.ram.write8(3, 0x3b);
    cpu.exec();
    assert_eq!(cpu.registers.read16(SP), 2046);
  }

  #[test]
  fn opcode_dec_r16_flags() {
    panic!("Not implemented yet");
  }

  #[test]
  fn opcode_ld_r8_n8() {
    let mut cpu = CPU::new();

    // LD B, 1
    cpu.ram.write8(0, 0b00_000_110);
    cpu.ram.write8(1, 1);
    cpu.exec();
    assert_eq!(cpu.registers.read8(B), 1);

    // LD C, 2
    cpu.ram.write8(2, 0b00_001_110);
    cpu.ram.write8(3, 2);
    cpu.exec();
    assert_eq!(cpu.registers.read8(C), 2);

    // LD D, 3
    cpu.ram.write8(4, 0b00_010_110);
    cpu.ram.write8(5, 3);
    cpu.exec();
    assert_eq!(cpu.registers.read8(D), 3);

    // LD E, 4
    cpu.ram.write8(6, 0b00_011_110);
    cpu.ram.write8(7, 4);
    cpu.exec();
    assert_eq!(cpu.registers.read8(E), 4);

    // LD H, 5
    cpu.ram.write8(8, 0b00_100_110);
    cpu.ram.write8(9, 5);
    cpu.exec();
    assert_eq!(cpu.registers.read8(H), 5);

    // LD L, 6
    cpu.ram.write8(10, 0b00_101_110);
    cpu.ram.write8(11, 6);
    cpu.exec();
    assert_eq!(cpu.registers.read8(L), 6);

    // LD (HL), 7
    cpu.registers.write16(HL, 1024);
    cpu.ram.write8(12, 0b00_110_110);
    cpu.ram.write8(13, 7);
    cpu.exec();
    assert_eq!(cpu.ram.read16(1024), 7);

    // LD A, 8
    cpu.ram.write8(14, 0b00_111_110);
    cpu.ram.write8(15, 8);
    cpu.exec();
    assert_eq!(cpu.registers.read8(A), 8);
  }
}
