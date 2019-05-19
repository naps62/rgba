pub mod opcodes;
pub mod registers;

use super::memory::Memory;
use registers::{Flag, Register16, Register8, Registers};
use std::num::Wrapping;

use Flag::*;
use Register16::*;
use Register8::*;

pub struct CPU {
  regs: Registers,
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
      regs: Registers::new(),
      ram: Memory::new(8 * 1024),
    }
  }

  // executes the next instruction referenced by PC
  pub fn exec(&mut self) {
    let current_pc = self.regs.read16(PC);

    let byte = self.ram.read8(current_pc as usize);
    let new_pc = self.exec_opcode(byte, current_pc);

    self.regs.set_pc(new_pc);
  }

  // executes the given opcode
  fn exec_opcode(&mut self, opcode: u8, pc: u16) -> u16 {
    match opcode {
      // NOP, do nothing
      0x00 => pc + 1,

      // LD (N), SP
      // no flags affected
      // DONE
      0x08 => {
        self.exec_ld(Ptr(self.read_arg16() as usize), R16(SP));
        pc + 3
      }

      // // LD R, N
      // // no flags affected
      // // DONE
      0b0000_0001 => {
        self.regs.write16(BC, self.read_arg16());
        pc + 3
      }
      0b0001_0001 => {
        self.regs.write16(DE, self.read_arg16());
        pc + 3
      }
      0b0010_0001 => {
        self.regs.write16(HL, self.read_arg16());
        pc + 3
      }
      0b0011_0001 => {
        self.regs.write16(SP, self.read_arg16());
        pc + 3
      }

      // ADD HL, R
      // all flags affected
      // DONE
      0b0000_1001 => {
        self.alu_add16(BC);
        pc + 1
      }
      0b0001_1001 => {
        self.alu_add16(DE);
        pc + 1
      }
      0b0010_1001 => {
        self.alu_add16(HL);
        pc + 1
      }
      0b0011_1001 => {
        self.alu_add16(SP);
        pc + 1
      }

      // LD (R), A
      // no flags affected
      // DONE
      0b0000_0010 => {
        self.ram.write8(self.regs.bc() as usize, self.regs.a());
        pc + 1
      }
      0b0001_0010 => {
        self.ram.write8(self.regs.de() as usize, self.regs.a());
        pc + 1
      }

      // LD A, (R)
      // no flags affected
      // DONE
      0b0000_1010 => {
        self.regs.write8(A, self.ram.read8(self.regs.bc() as usize));
        pc + 1
      }
      0b0001_1010 => {
        self.regs.write8(A, self.ram.read8(self.regs.de() as usize));
        pc + 1
      }

      // INC R
      // no flags affected
      // DONE
      0b0000_0011 => {
        self.regs.set_bc(self.regs.bc().wrapping_add(1));
        pc + 1
      }
      0b0001_0011 => {
        self.regs.set_de(self.regs.de().wrapping_add(1));
        pc + 1
      }
      0b0010_0011 => {
        self.regs.set_hl(self.regs.hl().wrapping_add(1));
        pc + 1
      }
      0b0011_0011 => {
        self.regs.set_sp(self.regs.sp().wrapping_add(1));
        pc + 1
      }

      // DEC R
      // no flags affected
      // DONE
      0b0000_1011 => {
        self.regs.set_bc(self.regs.bc().wrapping_sub(1));
        pc + 1
      }
      0b0001_1011 => {
        self.regs.set_de(self.regs.de().wrapping_sub(1));
        pc + 1
      }
      0b0010_1011 => {
        self.regs.set_hl(self.regs.hl().wrapping_sub(1));
        pc + 1
      }
      0b0011_1011 => {
        self.regs.set_sp(self.regs.sp().wrapping_sub(1));
        pc + 1
      }

      // INC D
      // Z, N, H
      // DONE
      0b0000_0100 => {
        let v = self.alu_inc(self.regs.b());
        self.regs.set_b(v);
        pc + 1
      }
      0b0000_1100 => {
        let v = self.alu_inc(self.regs.c());
        self.regs.set_c(v);
        pc + 1
      }
      0b0001_0100 => {
        let v = self.alu_inc(self.regs.d());
        self.regs.set_d(v);
        pc + 1
      }
      0b0001_1100 => {
        let v = self.alu_inc(self.regs.e());
        self.regs.set_e(v);
        pc + 1
      }
      0b0010_0100 => {
        let v = self.alu_inc(self.regs.h());
        self.regs.set_h(v);
        pc + 1
      }
      0b0010_1100 => {
        let v = self.alu_inc(self.regs.l());
        self.regs.set_l(v);
        pc + 1
      }
      0b0011_0100 => {
        let ptr: usize = self.regs.hl() as usize;
        let v = self.alu_inc(self.ram.read8(ptr));
        self.ram.write8(ptr, v);
        pc + 1
      }
      0b0011_1100 => {
        let v = self.alu_inc(self.regs.a());
        self.regs.set_a(v);
        pc + 1
      }

      // DEC D
      // Z, N, H
      // DONE
      0b0000_0101 => {
        let v = self.alu_dec(self.regs.b());
        self.regs.set_b(v);
        pc + 1
      }
      0b0000_1101 => {
        let v = self.alu_dec(self.regs.c());
        self.regs.set_c(v);
        pc + 1
      }
      0b0001_0101 => {
        let v = self.alu_dec(self.regs.d());
        self.regs.set_d(v);
        pc + 1
      }
      0b0001_1101 => {
        let v = self.alu_dec(self.regs.e());
        self.regs.set_e(v);
        pc + 1
      }
      0b0010_0101 => {
        let v = self.alu_dec(self.regs.h());
        self.regs.set_h(v);
        pc + 1
      }
      0b0010_1101 => {
        let v = self.alu_dec(self.regs.l());
        self.regs.set_l(v);
        pc + 1
      }
      0b0011_0101 => {
        let ptr: usize = self.regs.hl() as usize;
        let v = self.alu_dec(self.ram.read8(ptr));
        self.ram.write8(ptr, v);
        pc + 1
      }
      0b0011_1101 => {
        let v = self.alu_dec(self.regs.a());
        self.regs.set_a(v);
        pc + 1
      }

      0b0000_0110 => {
        self.regs.set_b(self.read_arg8());
        pc + 2
      }
      0b0000_1110 => {
        self.regs.set_c(self.read_arg8());
        pc + 2
      }
      0b0001_0110 => {
        self.regs.set_d(self.read_arg8());
        pc + 2
      }
      0b0001_1110 => {
        self.regs.set_e(self.read_arg8());
        pc + 2
      }
      0b0010_0110 => {
        self.regs.set_h(self.read_arg8());
        pc + 2
      }
      0b0010_1110 => {
        self.regs.set_l(self.read_arg8());
        pc + 2
      }
      0b0011_0110 => {
        let ptr: usize = self.regs.hl() as usize;
        self.ram.write8(ptr, self.read_arg8());
        pc + 2
      }
      0b0011_1110 => {
        self.regs.set_a(self.read_arg8());
        pc + 2
      }

      // RdCA
      // all flags affected
      // DONE
      0b0000_0111 => {
        let v = self.alu_rlc(self.regs.a());
        self.regs.set_a(v);
        pc + 1
      }
      0b0000_1111 => {
        let v = self.alu_rrc(self.regs.a());
        self.regs.set_a(v);
        pc + 1
      }

      0b0001_0111 => {
        let v = self.alu_rl(self.regs.a());
        self.regs.set_a(v);
        pc + 1
      }

      0b0001_1111 => {
        let v = self.alu_rr(self.regs.a());
        self.regs.set_a(v);
        pc + 1
      }

      0b0001_0000 => {
        // STOP
        // TODO
        1
      }

      // JR N
      // DONE
      0b0001_1000 => pc + self.read_arg8() as u16,

      // JR F, N
      0b0010_0000 => {
        if !self.regs.get_flag(ZF) {
          pc + self.read_arg8() as u16
        } else {
          pc + 2
        }
      }
      0b0010_1000 => {
        if self.regs.get_flag(ZF) {
          pc + self.read_arg8() as u16
        } else {
          pc + 2
        }
      }
      0b0011_0000 => {
        if !self.regs.get_flag(CF) {
          pc + self.read_arg8() as u16
        } else {
          pc + 2
        }
      }
      0b0011_1000 => {
        if self.regs.get_flag(CF) {
          pc + self.read_arg8() as u16
        } else {
          pc + 2
        }
      }

      // LDI (HL), A
      // DONE
      0b0010_0010 => {
        self.ram.write8(self.regs.hl() as usize, self.regs.a());
        self.regs.set_hl(self.regs.hl().wrapping_add(1));
        pc + 1
      }

      // LDI A, (HL)
      // DONE
      0b0010_1010 => {
        self.regs.set_a(self.ram.read8(self.regs.hl() as usize));
        self.regs.set_hl(self.regs.hl().wrapping_add(1));
        pc + 1
      }

      // LDD (HL), A
      // DONE
      0b0011_0010 => {
        self.ram.write8(self.regs.hl() as usize, self.regs.a());
        self.regs.set_hl(self.regs.hl().wrapping_sub(1));
        pc + 1
      }

      // LDD A, (HL)
      // DONE
      0b0011_1010 => {
        self.regs.set_a(self.ram.read8(self.regs.hl() as usize));
        self.regs.set_hl(self.regs.hl().wrapping_sub(1));
        pc + 1
      }

      // DAA
      // DONE
      0b0010_0111 => {
        self.alu_daa();
        pc + 1
      }

      // CPL
      // DONE
      0b0010_1111 => {
        self.regs.set_a(!self.regs.a());
        self.regs.set_flag(NF, true);
        self.regs.set_flag(HF, true);
        pc + 1
      }

      // SCF
      // DONE
      0b0011_0111 => {
        self.regs.set_flag(CF, true);
        self.regs.set_flag(NF, false);
        self.regs.set_flag(HF, false);
        pc + 1
      }

      // CCF
      0b0011_1111 => {
        self.regs.set_flag(CF, !self.regs.get_flag(CF));
        self.regs.set_flag(NF, false);
        self.regs.set_flag(HF, false);
        pc + 1
      }

      // LD B, r8
      0b0100_0000 => {
        self.regs.set_b(self.regs.b()); // no-op
        pc + 1
      }
      0b0100_0001 => {
        self.regs.set_b(self.regs.c());
        pc + 1
      }
      0b0100_0010 => {
        self.regs.set_b(self.regs.d()); // no-op
        pc + 1
      }
      0b0100_0011 => {
        self.regs.set_b(self.regs.e());
        pc + 1
      }
      0b0100_0100 => {
        self.regs.set_b(self.regs.h()); // no-op
        pc + 1
      }
      0b0100_0101 => {
        self.regs.set_b(self.regs.l());
        pc + 1
      }
      0b0100_0110 => {
        self.regs.set_b(self.ram.read8(self.regs.hl() as usize)); // no-op
        pc + 1
      }
      0b0100_0111 => {
        self.regs.set_b(self.regs.a());
        pc + 1
      }

      // LD C, r8
      0b0100_1000 => {
        self.regs.set_c(self.regs.b()); // no-op
        pc + 1
      }
      0b0100_1001 => {
        self.regs.set_c(self.regs.c());
        pc + 1
      }
      0b0100_1010 => {
        self.regs.set_c(self.regs.d()); // no-op
        pc + 1
      }
      0b0100_1011 => {
        self.regs.set_c(self.regs.e());
        pc + 1
      }
      0b0100_1100 => {
        self.regs.set_c(self.regs.h()); // no-op
        pc + 1
      }
      0b0100_1101 => {
        self.regs.set_c(self.regs.l());
        pc + 1
      }
      0b0100_1110 => {
        self.regs.set_c(self.ram.read8(self.regs.hl() as usize)); // no-op
        pc + 1
      }
      0b0100_1111 => {
        self.regs.set_c(self.regs.a());
        pc + 1
      }

      // LD D, r8
      0b0101_0000 => {
        self.regs.set_d(self.regs.b()); // no-op
        pc + 1
      }
      0b0101_0001 => {
        self.regs.set_d(self.regs.c());
        pc + 1
      }
      0b0101_0010 => {
        self.regs.set_d(self.regs.d()); // no-op
        pc + 1
      }
      0b0101_0011 => {
        self.regs.set_d(self.regs.e());
        pc + 1
      }
      0b0101_0100 => {
        self.regs.set_d(self.regs.h()); // no-op
        pc + 1
      }
      0b0101_0101 => {
        self.regs.set_d(self.regs.l());
        pc + 1
      }
      0b0101_0110 => {
        self.regs.set_d(self.ram.read8(self.regs.hl() as usize)); // no-op
        pc + 1
      }
      0b0101_0111 => {
        self.regs.set_d(self.regs.a());
        pc + 1
      }

      // LD E, r8
      0b0101_1000 => {
        self.regs.set_e(self.regs.b()); // no-op
        pc + 1
      }
      0b0101_1001 => {
        self.regs.set_e(self.regs.c());
        pc + 1
      }
      0b0101_1010 => {
        self.regs.set_e(self.regs.d()); // no-op
        pc + 1
      }
      0b0101_1011 => {
        self.regs.set_e(self.regs.e());
        pc + 1
      }
      0b0101_1100 => {
        self.regs.set_e(self.regs.h()); // no-op
        pc + 1
      }
      0b0101_1101 => {
        self.regs.set_e(self.regs.l());
        pc + 1
      }
      0b0101_1110 => {
        self.regs.set_e(self.ram.read8(self.regs.hl() as usize)); // no-op
        pc + 1
      }
      0b0101_1111 => {
        self.regs.set_e(self.regs.a());
        pc + 1
      }

      // LD H, r8
      0b0110_0000 => {
        self.regs.set_h(self.regs.b()); // no-op
        pc + 1
      }
      0b0110_0001 => {
        self.regs.set_h(self.regs.c());
        pc + 1
      }
      0b0110_0010 => {
        self.regs.set_h(self.regs.d()); // no-op
        pc + 1
      }
      0b0110_0011 => {
        self.regs.set_h(self.regs.e());
        pc + 1
      }
      0b0110_0100 => {
        self.regs.set_h(self.regs.h()); // no-op
        pc + 1
      }
      0b0110_0101 => {
        self.regs.set_h(self.regs.l());
        pc + 1
      }
      0b0110_0110 => {
        self.regs.set_h(self.ram.read8(self.regs.hl() as usize)); // no-op
        pc + 1
      }
      0b0110_0111 => {
        self.regs.set_h(self.regs.a());
        pc + 1
      }

      // LD L, r8
      0b0110_1000 => {
        self.regs.set_l(self.regs.b()); // no-op
        pc + 1
      }
      0b0110_1001 => {
        self.regs.set_l(self.regs.c());
        pc + 1
      }
      0b0110_1010 => {
        self.regs.set_l(self.regs.d()); // no-op
        pc + 1
      }
      0b0110_1011 => {
        self.regs.set_l(self.regs.e());
        pc + 1
      }
      0b0110_1100 => {
        self.regs.set_l(self.regs.h()); // no-op
        pc + 1
      }
      0b0110_1101 => {
        self.regs.set_l(self.regs.l());
        pc + 1
      }
      0b0110_1110 => {
        self.regs.set_l(self.ram.read8(self.regs.hl() as usize)); // no-op
        pc + 1
      }
      0b0110_1111 => {
        self.regs.set_l(self.regs.a());
        pc + 1
      }

      // LD (HL), r8
      0b0111_0000 => {
        self.ram.write8(self.regs.hl() as usize, self.regs.b());
        pc + 1
      }
      0b0111_0001 => {
        self.ram.write8(self.regs.hl() as usize, self.regs.c());
        pc + 1
      }
      0b0111_0010 => {
        self.ram.write8(self.regs.hl() as usize, self.regs.d());
        pc + 1
      }
      0b0111_0011 => {
        self.ram.write8(self.regs.hl() as usize, self.regs.e());
        pc + 1
      }
      0b0111_0100 => {
        self.ram.write8(self.regs.hl() as usize, self.regs.h());
        pc + 1
      }
      0b0111_0101 => {
        self.ram.write8(self.regs.hl() as usize, self.regs.l());
        pc + 1
      }
      0b0111_0111 => {
        self.ram.write8(self.regs.hl() as usize, self.regs.a());
        pc + 1
      }

      // LD A, r8
      0b0111_1000 => {
        self.regs.set_a(self.regs.b()); // no-op
        pc + 1
      }
      0b0111_1001 => {
        self.regs.set_a(self.regs.c());
        pc + 1
      }
      0b0111_1010 => {
        self.regs.set_a(self.regs.d()); // no-op
        pc + 1
      }
      0b0111_1011 => {
        self.regs.set_a(self.regs.e());
        pc + 1
      }
      0b0111_1100 => {
        self.regs.set_a(self.regs.h()); // no-op
        pc + 1
      }
      0b0111_1101 => {
        self.regs.set_a(self.regs.l());
        pc + 1
      }
      0b0111_1110 => {
        self.regs.set_a(self.ram.read8(self.regs.hl() as usize)); // no-op
        pc + 1
      }
      0b0111_1111 => {
        self.regs.set_a(self.regs.a());
        pc + 1
      }

      // HALT
      0b0111_0110 => {
        // HALT
        // TODO
        pc + 1
      }

      // // ALU A, D
      // _ if opcode_match(opcode, 0b1100_0000, 0b1000_0000) => pc + 1,

      // // ALU A, N
      // _ if opcode_match(opcode, 0b1100_0111, 0b1100_0110) => pc + 2,

      // // POP R
      // _ if opcode_match(opcode, 0b1100_1111, 0b1100_0001) => pc + 2,

      // // PUSH R
      // _ if opcode_match(opcode, 0b1100_1111, 0b1100_0101) => pc + 2,

      // // RST N
      // _ if opcode_match(opcode, 0b1100_0111, 0b1100_0111) => pc + 1,

      // // RET F
      // _ if opcode_match(opcode, 0b1110_0111, 0b1100_0000) => pc + 1,

      // // RET
      // 0xc9 => pc + 1,

      // // RETI
      // 0xd9 => pc + 1,

      // // JP F, N
      // _ if opcode_match(opcode, 0b1110_0111, 0b1100_0010) => pc + 3,

      // // JP N
      // 0xc3 => pc + 3,

      // // CALL F, N
      // _ if opcode_match(opcode, 0b1110_0111, 0b1100_0100) => pc + 3,

      // // CALL N
      // 0xcd => pc + 3,

      // // ADD SP, N
      // 0xe8 => pc + 2,

      // // LD HL, SP + N
      // 0xf8 => pc + 2,

      // // LD (FF00+N), A
      // 0xe0 => pc + 2,

      // // LD A, (FF00+N)
      // 0xf0 => pc + 2,

      // // LD (C), A
      // 0xe2 => pc + 1,

      // // LD A, (C)
      // 0xf2 => pc + 1,

      // // LD (N), A
      // 0xe6 => pc + 3,

      // // LD A, (N)
      // 0xf6 => pc + 3,

      // // JP HL
      // 0xe9 => pc + 1,
      // // LD SP, HL
      // 0xf9 => pc + 1,
      // // DI
      // 0xf3 => pc + 1,
      // // EI
      // 0xfb => pc + 1,

      // // read instr from byte 2
      // 0xcb => pc + 2,
      _ => self.i_unknown(opcode),
    }
  }

  fn i_nop(&self) -> u16 {
    self.regs.read16(PC) + 1
  }

  fn exec_ld(&mut self, dest: Arg, orig: Arg) {
    match (dest.clone(), orig) {
      (Ptr(addr), R16(reg)) => self.ram.write16(addr, self.regs.read16(reg)),
      (R16(reg), N16(val)) => self.regs.write16(reg, val),
      (R8(reg), N8(val)) => self.regs.write8(reg, val),
      (Ptr(addr), R8(reg)) => self.ram.write8(addr, self.regs.read8(reg)),
      (R8(reg), Ptr(addr)) => self.regs.write8(reg, self.ram.read8(addr)),
      (Ptr_R16(reg1), R8(reg2)) => self
        .ram
        .write8(self.regs.read16(reg1) as usize, self.regs.read8(reg2)),
      (Ptr_R16(reg), N8(val)) => self.ram.write8(self.regs.read16(reg) as usize, val),

      _ => panic!("Can't handle LD opcode arguments {:?}", (dest, orig)),
    };
  }

  fn exec_add(&mut self, dest: Arg, orig: Arg) -> (u32, u32) {
    match (dest, orig) {
      (R16(reg1), R16(reg2)) => {
        let (n1, n2) = (self.regs.read16(reg1), self.regs.read16(reg2));

        self.regs.write16(reg1, (Wrapping(n1) + Wrapping(n2)).0);

        (n1 as u32, n2 as u32)
      }

      _ => panic!("Can't handle ADD opcode arguments {:?}", (dest, orig)),
    }
  }

  fn exec_inc(&mut self, dest: Arg, inc: i16) -> u32 {
    match (dest, inc) {
      // (R16(reg), 1) => {
      //   let n = self.regs.read16(reg);

      //   self.regs.write16(reg, n + inc);

      //   n as u32
      // }

      // R8(reg) => {
      //   let n = self.regs.read8(reg);

      //   self.regs.write8(reg, n + inc);

      //   n as u32
      // }

      // Ptr_R16(reg) => {
      //   let addr = self.regs.read16(reg);

      //   let n = self.ram.read8(addr);

      //   self.ram.write8(addr, n + inc);

      //   // self.regs.write8(reg, n + inc);

      //   n as u32
      // }
      _ => panic!("Can't handle INC/DEC opcode argument {:?}", dest),
    }
  }

  fn alu_add16(&mut self, reg: Register16) {
    let hl = self.regs.read16(HL);
    let r = self.regs.read16(reg);

    let v = hl.wrapping_add(r);

    self.regs.write16(HL, v);

    self.regs.set_flag(NF, false);
    self.regs.set_flag(HF, self.overflow16(hl, r, 11));
    self.regs.set_flag(CF, self.overflow16(hl, r, 15));
  }

  fn alu_inc(&mut self, initial: u8) -> u8 {
    let v = initial.wrapping_add(1);

    self.regs.set_flag(ZF, v == 0);
    self.regs.set_flag(NF, false);
    self.regs.set_flag(HF, self.overflow8(initial, 1, 3));

    v
  }

  fn alu_dec(&mut self, initial: u8) -> u8 {
    let v = initial.wrapping_sub(1);

    self.regs.set_flag(ZF, v == 0);
    self.regs.set_flag(NF, true);
    self.regs.set_flag(HF, initial & 0x0F == 0);

    v
  }

  fn alu_rlc(&mut self, v: u8) -> u8 {
    let c = v & 0x80 == 0x80;
    let r = (v << 1) | (if c { 1 } else { 0 });

    self.regs.set_flag(HF, false);
    self.regs.set_flag(NF, false);
    self.regs.set_flag(ZF, false);
    self.regs.set_flag(CF, c);
    r
  }

  fn alu_rl(&mut self, v: u8) -> u8 {
    let c = v & 0x80 == 0x80;
    let r = (v << 1) | (if self.regs.get_flag(CF) { 1 } else { 0 });

    self.regs.set_flag(HF, false);
    self.regs.set_flag(NF, false);
    self.regs.set_flag(ZF, false);
    self.regs.set_flag(CF, c);
    r
  }

  fn alu_rrc(&mut self, v: u8) -> u8 {
    let c = v & 0x01 == 0x01;
    let r = (v >> 1) | (if c { 0x80 } else { 0 });

    self.regs.set_flag(HF, false);
    self.regs.set_flag(NF, false);
    self.regs.set_flag(ZF, false);
    self.regs.set_flag(CF, c);
    r
  }

  fn alu_rr(&mut self, v: u8) -> u8 {
    let c = v & 0x01 == 0x01;
    let r = (v >> 1) | (if self.regs.get_flag(CF) { 0x80 } else { 0 });

    self.regs.set_flag(HF, false);
    self.regs.set_flag(NF, false);
    self.regs.set_flag(ZF, false);
    self.regs.set_flag(CF, c);
    r
  }

  // implementation taken from
  // https://forums.nesdev.com/viewtopic.php?f=20&t=15944#p196282
  fn alu_daa(&mut self) {
    let a = self.regs.a();
    let mut adjust = 0x0;

    let new_a = if !self.regs.get_flag(NF) {
      if self.regs.get_flag(CF) || a > 0x99 {
        adjust |= 0x60;
        self.regs.set_flag(CF, true);
      };

      if self.regs.get_flag(HF) || (a & 0x0F) > 0x09 {
        adjust |= 0x06;
      };

      a.wrapping_add(adjust)
    } else {
      if self.regs.get_flag(CF) {
        adjust |= 0x60;
      };
      if self.regs.get_flag(HF) {
        adjust |= 0x06;
      };

      a.wrapping_sub(adjust)
    };

    self.regs.set_flag(CF, adjust >= 0x60);
    self.regs.set_flag(HF, false);
    self.regs.set_flag(ZF, new_a == 0);
    self.regs.set_a(new_a);
  }

  fn i_unknown(&self, opcode: u8) -> u16 {
    panic!(
      "Failed to execute unknown opcode: 0x{:02x} (0b{0:b})",
      opcode
    );
  }

  fn exec_arg8(&self) -> u8 {
    let pc = self.regs.read16(PC);

    self.ram.read8((pc + 1) as usize)
  }

  fn read_arg16(&self) -> u16 {
    let pc = self.regs.read16(PC);

    self.ram.read16((pc + 1) as usize)
  }

  fn read_arg8(&self) -> u8 {
    let pc = self.regs.read16(PC);

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

  fn overflow8(&self, n1: u8, n2: u8, index: u16) -> bool {
    self.overflow32(n1 as u32, n2 as u32, index)
  }

  fn overflow16(&self, n1: u16, n2: u16, index: u16) -> bool {
    self.overflow32(n1 as u32, n2 as u32, index)
  }

  fn overflow32(&self, n1: u32, n2: u32, index: u16) -> bool {
    let index_mask: u32 = 1 << index + 1;
    let mask: u32 = index_mask - 1;

    ((n1 & mask) + (n2 & mask) & index_mask) == index_mask
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

    assert_eq!(cpu.regs.read16(Register16::AF), 0);
  }

  // #[test]
  // fn exec_known_opcodes() {
  //   for opcode in 0x00..0xff {
  //     let mut cpu = CPU::new();

  //     cpu.ram.write8(0, opcode);

  //     cpu.exec();
  //   }
  // }

  #[test]
  fn opcode_nop() {
    let mut cpu = CPU::new();

    cpu.ram.write8(0, 0b00000000);

    cpu.exec();

    assert_eq!(cpu.regs.read16(Register16::PC), 1);
  }

  #[test]
  fn opcode_ld_ptr16_sp() {
    let mut cpu = CPU::new();
    cpu.regs.write16(SP, 2047);

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

    assert_eq!(cpu.regs.read16(BC), 511);
    assert_eq!(cpu.regs.read16(DE), 1023);
    assert_eq!(cpu.regs.read16(HL), 2047);
    assert_eq!(cpu.regs.read16(SP), 4095);
  }

  #[test]
  fn opcode_add_hl_r16() {
    let mut cpu = CPU::new();

    // ADD HL, BC
    cpu.regs.write16(HL, 128);
    cpu.regs.write16(BC, 127);
    cpu.ram.write8(0, 0x09);
    cpu.exec();
    assert_eq!(cpu.regs.read16(HL), 255);

    // ADD HL, DE
    cpu.regs.write16(HL, 256);
    cpu.regs.write16(DE, 255);
    cpu.ram.write8(1, 0x19);
    cpu.exec();
    assert_eq!(cpu.regs.read16(HL), 511);

    // ADD HL, HL
    cpu.regs.write16(HL, 511);
    cpu.ram.write8(2, 0x29);
    cpu.exec();
    assert_eq!(cpu.regs.read16(HL), 1022);

    // ADD HL, SP
    cpu.regs.write16(HL, 1024);
    cpu.regs.write16(SP, 1023);
    cpu.ram.write8(3, 0x39);
    cpu.exec();
    assert_eq!(cpu.regs.read16(HL), 2047);
  }

  #[test]
  fn opcode_add_hl_r16_flags() {
    let mut cpu = CPU::new();

    // carry from bit 11
    cpu.regs.write16(HL, 0b0000_1000_0000_0000);
    cpu.regs.write16(BC, 0b0000_1000_0000_0000);
    cpu.ram.write8(0, 0x09);
    cpu.exec();

    assert_eq!(cpu.regs.get_flag(NF), false);
    assert_eq!(cpu.regs.get_flag(HF), true);
    assert_eq!(cpu.regs.get_flag(CF), false);

    // carry from bit 15
    cpu.regs.write16(HL, 0b1000_0000_0000_0000);
    cpu.regs.write16(BC, 0b1000_0000_0000_0000);
    cpu.ram.write8(1, 0x09);
    cpu.exec();

    assert_eq!(cpu.regs.get_flag(NF), false);
    assert_eq!(cpu.regs.get_flag(HF), false);
    assert_eq!(cpu.regs.get_flag(CF), true);

    // carry from bit 11 and 15
    cpu.regs.write16(HL, 0b1000_1000_0000_0000);
    cpu.regs.write16(BC, 0b1000_1000_0000_0000);
    cpu.ram.write8(2, 0x09);
    cpu.exec();

    assert_eq!(cpu.regs.get_flag(NF), false);
    assert_eq!(cpu.regs.get_flag(HF), true);
    assert_eq!(cpu.regs.get_flag(CF), true);

    // carry from bit 11 and 15 indirectly
    cpu.regs.write16(HL, 0b1100_0100_0000_0000);
    cpu.regs.write16(BC, 0b0100_1100_0000_0000);
    cpu.ram.write8(2, 0x09);
    cpu.exec();

    assert_eq!(cpu.regs.get_flag(NF), false);
    assert_eq!(cpu.regs.get_flag(HF), true);
    assert_eq!(cpu.regs.get_flag(CF), true);
  }

  #[test]
  fn opcode_ld_r16_a() {
    let mut cpu = CPU::new();

    // LD BC, A
    cpu.regs.write8(A, 127);
    cpu.regs.write16(BC, 1024);
    cpu.ram.write8(0, 0x02);
    cpu.exec();
    assert_eq!(cpu.ram.read8(1024), 127);

    // LD DE, A
    cpu.regs.write8(A, 63);
    cpu.regs.write16(DE, 150);
    cpu.ram.write8(1, 0x12);
    cpu.exec();
    assert_eq!(cpu.ram.read8(150), 63);
  }

  #[test]
  fn opcode_ld_a_r16() {
    let mut cpu = CPU::new();

    // LD BC, A
    cpu.ram.write8(1024, 127);
    cpu.regs.write16(BC, 1024);
    cpu.ram.write8(0, 0x0a);
    cpu.exec();
    assert_eq!(cpu.regs.read8(A), 127);

    // LD DE, A
    cpu.ram.write8(150, 63);
    cpu.regs.write16(DE, 150);
    cpu.ram.write8(1, 0x1a);
    cpu.exec();
    assert_eq!(cpu.regs.read8(A), 63);
  }

  #[test]
  fn opcode_inc_r16() {
    let mut cpu = CPU::new();

    // INC BC
    cpu.regs.write16(BC, 257);
    cpu.ram.write8(0, 0x03);
    cpu.exec();
    assert_eq!(cpu.regs.read16(BC), 258);

    // INC DE
    cpu.regs.write16(DE, 511);
    cpu.ram.write8(1, 0x13);
    cpu.exec();
    assert_eq!(cpu.regs.read16(DE), 512);

    // INC HL
    cpu.regs.write16(HL, 1023);
    cpu.ram.write8(2, 0x23);
    cpu.exec();
    assert_eq!(cpu.regs.read16(HL), 1024);

    // INC SP
    cpu.regs.write16(SP, 2047);
    cpu.ram.write8(3, 0x33);
    cpu.exec();
    assert_eq!(cpu.regs.read16(SP), 2048);
  }

  #[test]
  fn opcode_inc_r16_flags() {
    // no flags touched
  }

  #[test]
  fn opcode_dec_r16() {
    let mut cpu = CPU::new();

    // INC BC
    cpu.regs.write16(BC, 257);
    cpu.ram.write8(0, 0x0b);
    cpu.exec();
    assert_eq!(cpu.regs.read16(BC), 256);

    // INC DE
    cpu.regs.write16(DE, 511);
    cpu.ram.write8(1, 0x1b);
    cpu.exec();
    assert_eq!(cpu.regs.read16(DE), 510);

    // INC HL
    cpu.regs.write16(HL, 1023);
    cpu.ram.write8(2, 0x2b);
    cpu.exec();
    assert_eq!(cpu.regs.read16(HL), 1022);

    // INC SP
    cpu.regs.write16(SP, 2047);
    cpu.ram.write8(3, 0x3b);
    cpu.exec();
    assert_eq!(cpu.regs.read16(SP), 2046);
  }

  #[test]
  fn opcode_dec_r16_flags() {
    // no flags touched
  }

  #[test]
  fn opcode_inc_r8() {
    let mut cpu = CPU::new();

    // INC B
    cpu.regs.set_b(1);
    cpu.ram.write8(0, 0x04);
    cpu.exec();
    assert_eq!(cpu.regs.read8(B), 2);

    // INC C
    cpu.regs.set_c(2);
    cpu.ram.write8(1, 0x0c);
    cpu.exec();
    assert_eq!(cpu.regs.read8(C), 3);

    // INC D
    cpu.regs.set_d(3);
    cpu.ram.write8(2, 0x14);
    cpu.exec();
    assert_eq!(cpu.regs.read8(D), 4);
    // INC E
    cpu.regs.set_e(4);
    cpu.ram.write8(3, 0x1c);
    cpu.exec();
    assert_eq!(cpu.regs.read8(E), 5);
    // INC H
    cpu.regs.set_h(5);
    cpu.ram.write8(4, 0x24);
    cpu.exec();
    assert_eq!(cpu.regs.read8(H), 6);
    // INC L
    cpu.regs.set_l(6);
    cpu.ram.write8(5, 0x2c);
    cpu.exec();
    assert_eq!(cpu.regs.read8(L), 7);

    // INC (HL)
    cpu.regs.set_hl(1023);
    cpu.ram.write8(6, 0x34);
    cpu.ram.write8(1023, 7);
    cpu.exec();
    assert_eq!(cpu.ram.read8(1023), 8);

    // INC A
    cpu.regs.set_a(8);
    cpu.ram.write8(7, 0x3c);
    cpu.exec();
    assert_eq!(cpu.regs.read8(A), 9);
  }

  #[test]
  fn opcode_inc_r8_flags() {
    let mut cpu = CPU::new();

    // NF is set to false
    cpu.regs.set_a(8);
    cpu.ram.write8(0, 0x3c);
    cpu.exec();
    assert_eq!(cpu.regs.get_flag(NF), false);

    // ZF is set to true if result is 0
    cpu.regs.set_a(0xFF);
    cpu.ram.write8(1, 0x3c);
    cpu.exec();
    assert_eq!(cpu.regs.get_flag(ZF), true);

    // ZF is set to false if result is not 0
    cpu.regs.set_a(0xFE);
    cpu.ram.write8(2, 0x3c);
    cpu.exec();
    assert_eq!(cpu.regs.get_flag(ZF), false);

    // HF is set to true if overflows from bit 3
    cpu.regs.set_a(0b0000_1111);
    cpu.ram.write8(3, 0x3c);
    cpu.exec();
    assert_eq!(cpu.regs.get_flag(HF), true);

    // HF is set to false if does not overflow from bit 3
    cpu.regs.set_a(0b0000_0111);
    cpu.ram.write8(4, 0x3c);
    cpu.exec();
    assert_eq!(cpu.regs.get_flag(HF), false);
  }

  #[test]
  fn opcode_dec_r8() {
    let mut cpu = CPU::new();

    // DEC B
    cpu.regs.set_b(1);
    cpu.ram.write8(0, 0x05);
    cpu.exec();
    assert_eq!(cpu.regs.b(), 0);

    // DEC C
    cpu.regs.set_c(2);
    cpu.ram.write8(1, 0x0d);
    cpu.exec();
    assert_eq!(cpu.regs.c(), 1);

    // DEC D
    cpu.regs.set_d(3);
    cpu.ram.write8(2, 0x15);
    cpu.exec();
    assert_eq!(cpu.regs.d(), 2);
    // DEC E
    cpu.regs.set_e(4);
    cpu.ram.write8(3, 0x1d);
    cpu.exec();
    assert_eq!(cpu.regs.e(), 3);
    // DEC H
    cpu.regs.set_h(5);
    cpu.ram.write8(4, 0x25);
    cpu.exec();
    assert_eq!(cpu.regs.h(), 4);
    // DEC L
    cpu.regs.set_l(6);
    cpu.ram.write8(5, 0x2d);
    cpu.exec();
    assert_eq!(cpu.regs.l(), 5);

    // DEC (HL)
    cpu.regs.set_hl(1023);
    cpu.ram.write8(6, 0x35);
    cpu.ram.write8(1023, 7);
    cpu.exec();
    assert_eq!(cpu.ram.read8(1023), 6);

    // DEC A
    cpu.regs.set_a(8);
    cpu.ram.write8(7, 0x3d);
    cpu.exec();
    assert_eq!(cpu.regs.a(), 7);
  }

  #[test]
  fn opcode_dec_r8_flags() {
    let mut cpu = CPU::new();

    // NF is set to true
    cpu.regs.set_a(8);
    cpu.ram.write8(0, 0x3d);
    cpu.exec();
    assert_eq!(cpu.regs.get_flag(NF), true);

    // ZF is set to true if result is 0
    cpu.regs.set_a(0x01);
    cpu.ram.write8(1, 0x3d);
    cpu.exec();
    assert_eq!(cpu.regs.get_flag(ZF), true);

    // ZF is set to false if result is not 0
    cpu.regs.set_a(0x02);
    cpu.ram.write8(2, 0x3d);
    cpu.exec();
    assert_eq!(cpu.regs.get_flag(ZF), false);

    // HF is set to true if overflows from bit 3
    cpu.regs.set_a(0b0000_0000);
    cpu.ram.write8(3, 0x3d);
    cpu.exec();
    assert_eq!(cpu.regs.get_flag(HF), true);

    // HF is set to false if does not overflow from bit 3
    cpu.regs.set_a(0b0000_1000);
    cpu.ram.write8(4, 0x3d);
    cpu.exec();
    assert_eq!(cpu.regs.get_flag(HF), false);
  }

  #[test]
  fn opcode_ld_r8_n8() {
    let mut cpu = CPU::new();

    // LD B, 1
    cpu.ram.write8(0, 0b00_000_110);
    cpu.ram.write8(1, 1);
    cpu.exec();
    assert_eq!(cpu.regs.read8(B), 1);

    // LD C, 2
    cpu.ram.write8(2, 0b00_001_110);
    cpu.ram.write8(3, 2);
    cpu.exec();
    assert_eq!(cpu.regs.read8(C), 2);

    // LD D, 3
    cpu.ram.write8(4, 0b00_010_110);
    cpu.ram.write8(5, 3);
    cpu.exec();
    assert_eq!(cpu.regs.read8(D), 3);

    // LD E, 4
    cpu.ram.write8(6, 0b00_011_110);
    cpu.ram.write8(7, 4);
    cpu.exec();
    assert_eq!(cpu.regs.read8(E), 4);

    // LD H, 5
    cpu.ram.write8(8, 0b00_100_110);
    cpu.ram.write8(9, 5);
    cpu.exec();
    assert_eq!(cpu.regs.read8(H), 5);

    // LD L, 6
    cpu.ram.write8(10, 0b00_101_110);
    cpu.ram.write8(11, 6);
    cpu.exec();
    assert_eq!(cpu.regs.read8(L), 6);

    // LD (HL), 7
    cpu.regs.write16(HL, 1024);
    cpu.ram.write8(12, 0b00_110_110);
    cpu.ram.write8(13, 7);
    cpu.exec();
    assert_eq!(cpu.ram.read16(1024), 7);

    // LD A, 8
    cpu.ram.write8(14, 0b00_111_110);
    cpu.ram.write8(15, 8);
    cpu.exec();
    assert_eq!(cpu.regs.read8(A), 8);
  }

  #[test]
  fn opcode_rdca() {
    let mut cpu = CPU::new();

    // RLCA
    cpu.regs.set_a(0b0000_0010);
    cpu.ram.write8(0, 0b0000_0111);
    cpu.exec();
    assert_eq!(cpu.regs.a(), 0b0000_0100);

    // RRCA
    cpu.regs.set_a(0b0000_0010);
    cpu.ram.write8(1, 0b0000_1111);
    cpu.exec();
    assert_eq!(cpu.regs.a(), 0b0000_0001);
  }

  #[test]
  fn opcode_rdca_flags() {
    let mut cpu = CPU::new();

    // ZH, HF and NF flags set to false
    cpu.regs.set_a(0b0000_0010);
    cpu.ram.write8(0, 0b0000_0111);
    cpu.exec();
    assert_eq!(cpu.regs.get_flag(HF), false);
    assert_eq!(cpu.regs.get_flag(NF), false);
    assert_eq!(cpu.regs.get_flag(ZF), false);

    // CF flag set to false if carry not used
    cpu.regs.set_a(0b0000_0010);
    cpu.ram.write8(1, 0b0000_0111);
    cpu.exec();
    assert_eq!(cpu.regs.get_flag(CF), false);

    // CF flag set to false if carry used
    cpu.regs.set_a(0b1000_0000);
    cpu.ram.write8(2, 0b0000_0111);
    cpu.exec();
    assert_eq!(cpu.regs.get_flag(CF), true);
  }

  #[test]
  fn opcode_rda() {
    let mut cpu = CPU::new();

    // RLA
    cpu.regs.set_a(0b0000_0010);
    cpu.regs.set_flag(CF, false);
    cpu.ram.write8(0, 0b0001_0111);
    cpu.exec();
    assert_eq!(cpu.regs.a(), 0b0000_0100);

    // RLA without carry flag
    cpu.regs.set_a(0b1000_0000);
    cpu.regs.set_flag(CF, false);
    cpu.ram.write8(1, 0b0001_0111);
    cpu.exec();
    assert_eq!(cpu.regs.a(), 0b0000_0000);

    // RLA with carry flag
    cpu.regs.set_a(0b1000_0000);
    cpu.regs.set_flag(CF, true);
    cpu.ram.write8(2, 0b0001_0111);
    cpu.exec();
    assert_eq!(cpu.regs.a(), 0b0000_0001);

    // RRA
    cpu.regs.set_a(0b0000_0010);
    cpu.regs.set_flag(CF, false);
    cpu.ram.write8(3, 0b0001_1111);
    cpu.exec();
    assert_eq!(cpu.regs.a(), 0b0000_0001);

    // RRA without carry flag
    cpu.regs.set_a(0b0000_0001);
    cpu.regs.set_flag(CF, false);
    cpu.ram.write8(4, 0b0001_1111);
    cpu.exec();
    assert_eq!(cpu.regs.a(), 0b0000_0000);

    // RRA with carry flag
    cpu.regs.set_a(0b0000_0001);
    cpu.regs.set_flag(CF, true);
    cpu.ram.write8(5, 0b0001_1111);
    cpu.exec();
    assert_eq!(cpu.regs.a(), 0b1000_0000);
  }

  #[test]
  fn opcode_rda_flags() {
    let mut cpu = CPU::new();

    // ZH, HF and NF flags set to false
    cpu.regs.set_a(0b0000_0010);
    cpu.ram.write8(0, 0b0001_0111);
    cpu.exec();
    assert_eq!(cpu.regs.get_flag(HF), false);
    assert_eq!(cpu.regs.get_flag(NF), false);
    assert_eq!(cpu.regs.get_flag(ZF), false);

    // CF flag set to false if carry not used
    cpu.regs.set_a(0b0000_0010);
    cpu.ram.write8(1, 0b0001_0111);
    cpu.exec();
    assert_eq!(cpu.regs.get_flag(CF), false);

    // CF flag set to false if carry used
    cpu.regs.set_a(0b1000_0000);
    cpu.ram.write8(2, 0b0001_0111);
    cpu.exec();
    assert_eq!(cpu.regs.get_flag(CF), true);
  }

  #[test]
  fn opcode_jr_n() {
    let mut cpu = CPU::new();

    // new PC is incremented by N
    cpu.ram.write8(0, 0b0001_1000);
    cpu.ram.write8(1, 0b0000_0011);
    cpu.exec();
    assert_eq!(cpu.regs.pc(), 3);
  }

  #[test]
  fn opcode_jr_f_n() {
    let mut cpu = CPU::new();

    // JR NZ, N increments by N if NZ
    cpu.regs.set_pc(0);
    cpu.regs.set_flag(ZF, false);
    cpu.ram.write8(0, 0b0010_0000);
    cpu.ram.write8(1, 0b0000_1000);
    cpu.exec();
    assert_eq!(cpu.regs.pc(), 8);

    // JR NZ, N increments by 2 if not NZ
    cpu.regs.set_pc(0);
    cpu.regs.set_flag(ZF, true);
    cpu.ram.write8(0, 0b0010_0000);
    cpu.ram.write8(1, 0b0000_1000);
    cpu.exec();
    assert_eq!(cpu.regs.pc(), 2);

    // JR Z, N increments by N if not Z
    cpu.regs.set_pc(0);
    cpu.regs.set_flag(ZF, true);
    cpu.ram.write8(0, 0b0010_1000);
    cpu.ram.write8(1, 0b0000_1000);
    cpu.exec();
    assert_eq!(cpu.regs.pc(), 8);

    // JR Z, N increments by 2 if Z
    cpu.regs.set_pc(0);
    cpu.regs.set_flag(ZF, false);
    cpu.ram.write8(0, 0b0010_1000);
    cpu.ram.write8(1, 0b0000_1000);
    cpu.exec();
    assert_eq!(cpu.regs.pc(), 2);

    // JR NC, N increments by N if NC
    cpu.regs.set_pc(0);
    cpu.regs.set_flag(CF, false);
    cpu.ram.write8(0, 0b0011_0000);
    cpu.ram.write8(1, 0b0000_1000);
    cpu.exec();
    assert_eq!(cpu.regs.pc(), 8);

    // JR NC, N increments by 2 if not NC
    cpu.regs.set_pc(0);
    cpu.regs.set_flag(CF, true);
    cpu.ram.write8(0, 0b0011_0000);
    cpu.ram.write8(1, 0b0000_1000);
    cpu.exec();
    assert_eq!(cpu.regs.pc(), 2);

    // JR C, N increments by 2 if not C
    cpu.regs.set_pc(0);
    cpu.regs.set_flag(CF, false);
    cpu.ram.write8(0, 0b0011_1000);
    cpu.ram.write8(1, 0b0000_1000);
    cpu.exec();
    assert_eq!(cpu.regs.pc(), 2);

    // JR C, N increments by N if C
    cpu.regs.set_pc(0);
    cpu.regs.set_flag(CF, true);
    cpu.ram.write8(0, 0b0011_1000);
    cpu.ram.write8(1, 0b0000_1000);
    cpu.exec();
    assert_eq!(cpu.regs.pc(), 8);
  }

  #[test]
  fn opcode_ldi_hl_a() {
    let mut cpu = CPU::new();

    cpu.regs.set_hl(128);
    cpu.regs.set_a(2);
    cpu.ram.write8(0, 0b0010_0010);
    cpu.exec();
    assert_eq!(cpu.ram.read8(128), 2);
    assert_eq!(cpu.regs.hl(), 129);
  }

  #[test]
  fn opcode_ldi_a_hl() {
    let mut cpu = CPU::new();

    cpu.regs.set_hl(128);
    cpu.ram.write8(128, 2);
    cpu.ram.write8(0, 0b0010_1010);
    cpu.exec();
    assert_eq!(cpu.regs.a(), 2);
    assert_eq!(cpu.regs.hl(), 129);
  }

  #[test]
  fn opcode_ldd_hl_a() {
    let mut cpu = CPU::new();

    cpu.regs.set_hl(128);
    cpu.regs.set_a(2);
    cpu.ram.write8(0, 0b0011_0010);
    cpu.exec();
    assert_eq!(cpu.ram.read8(128), 2);
    assert_eq!(cpu.regs.hl(), 127);
  }

  #[test]
  fn opcode_ldd_a_hl() {
    let mut cpu = CPU::new();

    cpu.regs.set_hl(128);
    cpu.ram.write8(128, 2);
    cpu.ram.write8(0, 0b0011_1010);
    cpu.exec();
    assert_eq!(cpu.regs.a(), 2);
    assert_eq!(cpu.regs.hl(), 127);
  }

  #[test]
  fn opcode_daa() {
    let mut cpu = CPU::new();

    // adds 0x06 to A if small digit is greater than 9
    cpu.regs.set_flag(NF, false);
    cpu.regs.set_a(0x0A);
    cpu.ram.write8(0, 0b0010_0111);
    cpu.exec();
    assert_eq!(cpu.regs.a(), 0x10);

    // adds 0x60 to A if big digit is greater than 9 and CF is set
    cpu.regs.set_flag(NF, false);
    cpu.regs.set_flag(CF, true);
    cpu.regs.set_a(0xA0);
    cpu.ram.write8(1, 0b0010_0111);
    cpu.exec();
    assert_eq!(cpu.regs.a(), 0x00);

    // subs 0x06 to A if small digit if C and H flags are set
    cpu.regs.set_flag(NF, true);
    cpu.regs.set_flag(CF, false);
    cpu.regs.set_flag(HF, true);
    cpu.regs.set_a(0x07);
    cpu.ram.write8(2, 0b0010_0111);
    cpu.exec();
    assert_eq!(cpu.regs.a(), 0x01);

    // subs 0x60 to A if small digit if C and C flags are set
    cpu.regs.set_flag(NF, true);
    cpu.regs.set_flag(CF, true);
    cpu.regs.set_flag(HF, false);
    cpu.regs.set_a(0x70);
    cpu.ram.write8(3, 0b0010_0111);
    cpu.exec();
    assert_eq!(cpu.regs.a(), 0x10);
  }

  #[test]
  fn opcode_daa_flags() {
    let mut cpu = CPU::new();

    // HF flag is reset
    cpu.regs.set_flag(NF, false);
    cpu.regs.set_a(0x0A);
    cpu.ram.write8(0, 0b0010_0111);
    cpu.exec();
    assert_eq!(cpu.regs.get_flag(HF), false);

    // ZF flag is set if result is zero
    cpu.regs.set_flag(NF, false);
    cpu.regs.set_flag(CF, true);
    cpu.regs.set_a(0xA0);
    cpu.ram.write8(1, 0b0010_0111);
    cpu.exec();
    assert_eq!(cpu.regs.get_flag(ZF), true);

    // ZF flag is reset if result is not zero
    cpu.regs.set_flag(NF, true);
    cpu.regs.set_flag(CF, false);
    cpu.regs.set_flag(HF, true);
    cpu.regs.set_a(0x07);
    cpu.ram.write8(2, 0b0010_0111);
    cpu.exec();
    assert_eq!(cpu.regs.get_flag(ZF), false);

    // CF flag is set if adjustment is 0x60
    cpu.regs.set_flag(NF, false);
    cpu.regs.set_flag(CF, true);
    cpu.regs.set_a(0x07);
    cpu.ram.write8(3, 0b0010_0111);
    cpu.exec();
    assert_eq!(cpu.regs.get_flag(CF), true);

    // CF flag is reset if adjustment is lower than 0x60
    cpu.regs.set_flag(NF, false);
    cpu.regs.set_flag(CF, false);
    cpu.regs.set_a(0x07);
    cpu.ram.write8(4, 0b0010_0111);
    cpu.exec();
    assert_eq!(cpu.regs.get_flag(CF), false);
  }

  #[test]
  fn opcode_cpl() {
    let mut cpu = CPU::new();

    cpu.regs.set_a(1);
    cpu.ram.write8(0, 0b0010_1111);
    cpu.exec();
    assert_eq!(cpu.regs.a(), 254);
  }

  #[test]
  fn opcode_cpl_flags() {
    let mut cpu = CPU::new();

    cpu.regs.set_a(1);
    cpu.ram.write8(0, 0b0010_1111);
    cpu.exec();
    assert_eq!(cpu.regs.get_flag(NF), true);
    assert_eq!(cpu.regs.get_flag(HF), true);
  }

  #[test]
  fn opcode_scf() {
    let mut cpu = CPU::new();

    cpu.ram.write8(0, 0b0011_0111);
    cpu.exec();
    assert_eq!(cpu.regs.get_flag(NF), false);
    assert_eq!(cpu.regs.get_flag(HF), false);
    assert_eq!(cpu.regs.get_flag(CF), true);
  }

  #[test]
  fn opcode_ccf() {
    let mut cpu = CPU::new();

    cpu.ram.write8(0, 0b0011_1111);
    cpu.exec();
    assert_eq!(cpu.regs.get_flag(NF), false);
    assert_eq!(cpu.regs.get_flag(HF), false);
    assert_eq!(cpu.regs.get_flag(CF), true);

    cpu.ram.write8(1, 0b0011_1111);
    cpu.exec();
    assert_eq!(cpu.regs.get_flag(NF), false);
    assert_eq!(cpu.regs.get_flag(HF), false);
    assert_eq!(cpu.regs.get_flag(CF), false);

    cpu.ram.write8(2, 0b0011_1111);
    cpu.exec();
    assert_eq!(cpu.regs.get_flag(NF), false);
    assert_eq!(cpu.regs.get_flag(HF), false);
    assert_eq!(cpu.regs.get_flag(CF), true);
  }

  #[test]
  fn opcode_ld_b_r8() {
    let mut cpu = CPU::new();

    cpu.regs.set_b(1);
    cpu.ram.write8(0, 0b0100_0000);
    cpu.exec();
    assert_eq!(cpu.regs.b(), 1);

    cpu.regs.set_c(2);
    cpu.ram.write8(1, 0b0100_0001);
    cpu.exec();
    assert_eq!(cpu.regs.b(), 2);

    cpu.regs.set_d(3);
    cpu.ram.write8(2, 0b0100_0010);
    cpu.exec();
    assert_eq!(cpu.regs.b(), 3);

    cpu.regs.set_e(4);
    cpu.ram.write8(3, 0b0100_0011);
    cpu.exec();
    assert_eq!(cpu.regs.b(), 4);

    cpu.regs.set_h(5);
    cpu.ram.write8(4, 0b0100_0100);
    cpu.exec();
    assert_eq!(cpu.regs.b(), 5);

    cpu.regs.set_l(6);
    cpu.ram.write8(5, 0b0100_0101);
    cpu.exec();
    assert_eq!(cpu.regs.b(), 6);

    cpu.ram.write8(128, 7);
    cpu.regs.set_hl(128);
    cpu.ram.write8(6, 0b0100_0110);
    cpu.exec();
    assert_eq!(cpu.regs.b(), 7);

    cpu.regs.set_a(8);
    cpu.ram.write8(7, 0b0100_0111);
    cpu.exec();
    assert_eq!(cpu.regs.b(), 8);
  }

  #[test]
  fn opcode_ld_c_r8() {
    let mut cpu = CPU::new();

    cpu.regs.set_b(1);
    cpu.ram.write8(0, 0b0100_1000);
    cpu.exec();
    assert_eq!(cpu.regs.c(), 1);

    cpu.regs.set_c(2);
    cpu.ram.write8(1, 0b0100_1001);
    cpu.exec();
    assert_eq!(cpu.regs.c(), 2);

    cpu.regs.set_d(3);
    cpu.ram.write8(2, 0b0100_1010);
    cpu.exec();
    assert_eq!(cpu.regs.c(), 3);

    cpu.regs.set_e(4);
    cpu.ram.write8(3, 0b0100_1011);
    cpu.exec();
    assert_eq!(cpu.regs.c(), 4);

    cpu.regs.set_h(5);
    cpu.ram.write8(4, 0b0100_1100);
    cpu.exec();
    assert_eq!(cpu.regs.c(), 5);

    cpu.regs.set_l(6);
    cpu.ram.write8(5, 0b0100_1101);
    cpu.exec();
    assert_eq!(cpu.regs.c(), 6);

    cpu.ram.write8(128, 7);
    cpu.regs.set_hl(128);
    cpu.ram.write8(6, 0b0100_1110);
    cpu.exec();
    assert_eq!(cpu.regs.c(), 7);

    cpu.regs.set_a(8);
    cpu.ram.write8(7, 0b0100_1111);
    cpu.exec();
    assert_eq!(cpu.regs.c(), 8);
  }

  #[test]
  fn opcode_ld_d_r8() {
    let mut cpu = CPU::new();

    cpu.regs.set_b(1);
    cpu.ram.write8(0, 0b0101_0000);
    cpu.exec();
    assert_eq!(cpu.regs.d(), 1);

    cpu.regs.set_c(2);
    cpu.ram.write8(1, 0b0101_0001);
    cpu.exec();
    assert_eq!(cpu.regs.d(), 2);

    cpu.regs.set_d(3);
    cpu.ram.write8(2, 0b0101_0010);
    cpu.exec();
    assert_eq!(cpu.regs.d(), 3);

    cpu.regs.set_e(4);
    cpu.ram.write8(3, 0b0101_0011);
    cpu.exec();
    assert_eq!(cpu.regs.d(), 4);

    cpu.regs.set_h(5);
    cpu.ram.write8(4, 0b0101_0100);
    cpu.exec();
    assert_eq!(cpu.regs.d(), 5);

    cpu.regs.set_l(6);
    cpu.ram.write8(5, 0b0101_0101);
    cpu.exec();
    assert_eq!(cpu.regs.d(), 6);

    cpu.ram.write8(128, 7);
    cpu.regs.set_hl(128);
    cpu.ram.write8(6, 0b0101_0110);
    cpu.exec();
    assert_eq!(cpu.regs.d(), 7);

    cpu.regs.set_a(8);
    cpu.ram.write8(7, 0b0101_0111);
    cpu.exec();
    assert_eq!(cpu.regs.d(), 8);
  }

  #[test]
  fn opcode_ld_e_r8() {
    let mut cpu = CPU::new();

    cpu.regs.set_b(1);
    cpu.ram.write8(0, 0b0101_1000);
    cpu.exec();
    assert_eq!(cpu.regs.e(), 1);

    cpu.regs.set_c(2);
    cpu.ram.write8(1, 0b0101_1001);
    cpu.exec();
    assert_eq!(cpu.regs.e(), 2);

    cpu.regs.set_d(3);
    cpu.ram.write8(2, 0b0101_1010);
    cpu.exec();
    assert_eq!(cpu.regs.e(), 3);

    cpu.regs.set_e(4);
    cpu.ram.write8(3, 0b0101_1011);
    cpu.exec();
    assert_eq!(cpu.regs.e(), 4);

    cpu.regs.set_h(5);
    cpu.ram.write8(4, 0b0101_1100);
    cpu.exec();
    assert_eq!(cpu.regs.e(), 5);

    cpu.regs.set_l(6);
    cpu.ram.write8(5, 0b0101_1101);
    cpu.exec();
    assert_eq!(cpu.regs.e(), 6);

    cpu.ram.write8(128, 7);
    cpu.regs.set_hl(128);
    cpu.ram.write8(6, 0b0101_1110);
    cpu.exec();
    assert_eq!(cpu.regs.e(), 7);

    cpu.regs.set_a(8);
    cpu.ram.write8(7, 0b0101_1111);
    cpu.exec();
    assert_eq!(cpu.regs.e(), 8);
  }

  #[test]
  fn opcode_ld_h_r8() {
    let mut cpu = CPU::new();

    cpu.regs.set_b(1);
    cpu.ram.write8(0, 0b0110_0000);
    cpu.exec();
    assert_eq!(cpu.regs.h(), 1);

    cpu.regs.set_c(2);
    cpu.ram.write8(1, 0b0110_0001);
    cpu.exec();
    assert_eq!(cpu.regs.h(), 2);

    cpu.regs.set_d(3);
    cpu.ram.write8(2, 0b0110_0010);
    cpu.exec();
    assert_eq!(cpu.regs.h(), 3);

    cpu.regs.set_e(4);
    cpu.ram.write8(3, 0b0110_0011);
    cpu.exec();
    assert_eq!(cpu.regs.h(), 4);

    cpu.regs.set_h(5);
    cpu.ram.write8(4, 0b0110_0100);
    cpu.exec();
    assert_eq!(cpu.regs.h(), 5);

    cpu.regs.set_l(6);
    cpu.ram.write8(5, 0b0110_0101);
    cpu.exec();
    assert_eq!(cpu.regs.h(), 6);

    cpu.ram.write8(128, 7);
    cpu.regs.set_hl(128);
    cpu.ram.write8(6, 0b0110_0110);
    cpu.exec();
    assert_eq!(cpu.regs.h(), 7);

    cpu.regs.set_a(8);
    cpu.ram.write8(7, 0b0110_0111);
    cpu.exec();
    assert_eq!(cpu.regs.h(), 8);
  }

  #[test]
  fn opcode_ld_l_r8() {
    let mut cpu = CPU::new();

    cpu.regs.set_b(1);
    cpu.ram.write8(0, 0b0110_1000);
    cpu.exec();
    assert_eq!(cpu.regs.l(), 1);

    cpu.regs.set_c(2);
    cpu.ram.write8(1, 0b0110_1001);
    cpu.exec();
    assert_eq!(cpu.regs.l(), 2);

    cpu.regs.set_d(3);
    cpu.ram.write8(2, 0b0110_1010);
    cpu.exec();
    assert_eq!(cpu.regs.l(), 3);

    cpu.regs.set_e(4);
    cpu.ram.write8(3, 0b0110_1011);
    cpu.exec();
    assert_eq!(cpu.regs.l(), 4);

    cpu.regs.set_h(5);
    cpu.ram.write8(4, 0b0110_1100);
    cpu.exec();
    assert_eq!(cpu.regs.l(), 5);

    cpu.regs.set_l(6);
    cpu.ram.write8(5, 0b0110_1101);
    cpu.exec();
    assert_eq!(cpu.regs.l(), 6);

    cpu.ram.write8(128, 7);
    cpu.regs.set_hl(128);
    cpu.ram.write8(6, 0b0110_1110);
    cpu.exec();
    assert_eq!(cpu.regs.l(), 7);

    cpu.regs.set_a(8);
    cpu.ram.write8(7, 0b0110_1111);
    cpu.exec();
    assert_eq!(cpu.regs.l(), 8);
  }

  #[test]
  fn opcode_ld_hl_r8() {
    let mut cpu = CPU::new();

    cpu.regs.set_hl(128);

    cpu.regs.set_b(1);
    cpu.ram.write8(0, 0b0111_0000);
    cpu.exec();
    assert_eq!(cpu.ram.read8(128), 1);

    cpu.regs.set_c(2);
    cpu.ram.write8(1, 0b0111_0001);
    cpu.exec();
    assert_eq!(cpu.ram.read8(128), 2);

    cpu.regs.set_d(3);
    cpu.ram.write8(2, 0b0111_0010);
    cpu.exec();
    assert_eq!(cpu.ram.read8(128), 3);

    cpu.regs.set_e(4);
    cpu.ram.write8(3, 0b0111_0011);
    cpu.exec();
    assert_eq!(cpu.ram.read8(128), 4);

    cpu.regs.set_h(5);
    cpu.ram.write8(4, 0b0111_0100);
    cpu.exec();
    assert_eq!(cpu.ram.read8(cpu.regs.hl() as usize), 5);

    cpu.regs.set_l(6);
    cpu.ram.write8(5, 0b0111_0101);
    cpu.exec();
    assert_eq!(cpu.ram.read8(cpu.regs.hl() as usize), 6);

    cpu.regs.set_a(7);
    cpu.ram.write8(6, 0b0111_0111);
    cpu.exec();
    assert_eq!(cpu.ram.read8(cpu.regs.hl() as usize), 7);
  }

  #[test]
  fn opcode_ld_a_r8() {
    let mut cpu = CPU::new();

    cpu.regs.set_b(1);
    cpu.ram.write8(0, 0b0111_1000);
    cpu.exec();
    assert_eq!(cpu.regs.a(), 1);

    cpu.regs.set_c(2);
    cpu.ram.write8(1, 0b0111_1001);
    cpu.exec();
    assert_eq!(cpu.regs.a(), 2);

    cpu.regs.set_d(3);
    cpu.ram.write8(2, 0b0111_1010);
    cpu.exec();
    assert_eq!(cpu.regs.a(), 3);

    cpu.regs.set_e(4);
    cpu.ram.write8(3, 0b0111_1011);
    cpu.exec();
    assert_eq!(cpu.regs.a(), 4);

    cpu.regs.set_h(5);
    cpu.ram.write8(4, 0b0111_1100);
    cpu.exec();
    assert_eq!(cpu.regs.a(), 5);

    cpu.regs.set_l(6);
    cpu.ram.write8(5, 0b0111_1101);
    cpu.exec();
    assert_eq!(cpu.regs.a(), 6);

    cpu.ram.write8(128, 7);
    cpu.regs.set_hl(128);
    cpu.ram.write8(6, 0b0111_1110);
    cpu.exec();
    assert_eq!(cpu.regs.a(), 7);

    cpu.regs.set_a(8);
    cpu.ram.write8(7, 0b0111_1111);
    cpu.exec();
    assert_eq!(cpu.regs.a(), 8);
  }
}
