pub mod opcodes;
pub mod registers;

use super::memory::Memory;
use registers::{Register16, Register8, Registers};

use Register16::*;
use Register8::*;

pub struct CPU {
  registers: Registers,
  ram: Memory,
}

pub enum Arg {
  R8(Register8),
  R16(Register16),
  N8(u8),
  N16(u16),
  Addr(u16),
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
      0x04 => {
        self.exec_ld(Addr(self.read_arg16()), R16(SP));

        pc + 3
      }

      // LD R, N
      _ if opcode_match(opcode, 0b_1100_1111, 0b0000_0001) => {
        // self.exec_ld(self.read_reg16((opcode & 0x0011_0000)) >> 4, N16(self.read_arg16()))
        pc + 3
      }

      // ADD HL, R
      _ if opcode_match(opcode, 0b1100_1111, 0b0000_1001) => pc + 1,

      // LD (R), A
      _ if opcode_match(opcode, 0b1110_1111, 0b0000_0010) => pc + 1,

      // LD A, (R)
      _ if opcode_match(opcode, 0b1110_1111, 0b0000_1010) => pc + 1,

      // LD A, (R)
      _ if opcode_match(opcode, 0b1110_1111, 0b0000_1010) => pc + 1,

      // INC R
      _ if opcode_match(opcode, 0b1100_1111, 0b0000_0011) => pc + 1,

      // DEC R
      _ if opcode_match(opcode, 0b1100_1111, 0b0000_1011) => pc + 1,

      // INC D
      _ if opcode_match(opcode, 0b1100_0111, 0b0000_0100) => pc + 1,

      // DEC D
      _ if opcode_match(opcode, 0b1100_0111, 0b0000_0101) => pc + 1,

      // LD D, N
      _ if opcode_match(opcode, 0b1100_0111, 0b0000_0110) => pc + 2,

      // RdCA
      _ if opcode_match(opcode, 0b1111_0111, 0b0000_0111) => pc + 1,

      // RdA
      _ if opcode_match(opcode, 0b1111_0111, 0b0001_0111) => pc + 1,

      // STOP
      0x10 => pc + 1,

      // JR N
      0x18 => pc + 1,

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
      0x3f => pc + 1,

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

  fn exec_ld(&mut self, orig: Arg, dest: Arg) -> u16 {
    self.registers.read16(PC) + 3
  }

  fn i_unknown(&self, opcode: u8) -> u16 {
    panic!("Failed to execute unknown opcode: {:x}", opcode);
  }

  fn exec_arg8(&self) -> u8 {
    let pc = self.registers.read16(PC);

    self.ram.read8((pc + 1) as usize)
  }

  fn read_arg16(&self) -> u16 {
    let pc = self.registers.read16(PC);

    self.ram.read16((pc + 1) as usize)
  }

  fn read_reg16(reg: u8) -> Arg {
    R16(PC)
    // match reg {

    // }
    // R8()
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
  fn exec_i_nop() {
    let mut cpu = CPU::new();

    cpu.ram.write8(0, 0b00000000);

    cpu.exec();

    assert_eq!(cpu.registers.read16(Register16::PC), 1);
  }

  #[test]
  fn exec_known_opcodes() {
    for opcode in 0x00..0xff {
      let mut cpu = CPU::new();

      cpu.ram.write8(0, opcode);

      cpu.exec();
    }
  }
}
