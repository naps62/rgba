pub mod registers;

use super::mmu::MMU;
use registers::{Flag, Register16, Register8, Registers};

use Flag::*;
use Register16::*;
use Register8::*;

pub struct CPU {
  regs: Registers,
  interrupts: u32,
}

impl CPU {
  #[allow(dead_code)]
  pub fn new() -> CPU {
    CPU {
      regs: Registers::new(),
      interrupts: 0,
    }
  }

  // executes the next instruction referenced by PC
  #[allow(dead_code)]
  pub fn exec(&mut self, mmu: &mut MMU) {
    let current_pc = self.regs.read16(PC);

    let byte = mmu.read8(current_pc as usize);
    let new_pc = self.exec_opcode(byte, current_pc, mmu);

    self.regs.set_pc(new_pc);
  }

  // executes the given opcode
  #[allow(unused_macros)]
  fn exec_opcode(&mut self, opcode: u8, pc: u16, mmu: &mut MMU) -> u16 {
    macro_rules! alu_add_a {
      ($d:expr) => {{
        let a = self.regs.a();
        let v = a.wrapping_add($d);

        self.regs.set_flag(ZF, v == 0);
        self.regs.set_flag(HF, (a & 0x0F) + ($d & 0x0F) > 0x0F);
        self.regs.set_flag(NF, false);
        self.regs.set_flag(CF, (a as u16) + ($d as u16) > 0xFF);

        self.regs.set_a(v);

        self.regs.pc() + 1
      }};
    }

    macro_rules! alu_adc_a {
      ($d:expr) => {{
        let a = self.regs.a();
        let c = if self.regs.get_flag(CF) { 1 } else { 0 };
        let v = a.wrapping_add($d).wrapping_add(c);

        self.regs.set_flag(ZF, v == 0);
        self.regs.set_flag(HF, (a & 0x0F) + ($d & 0x0F) + c > 0x0F);
        self.regs.set_flag(NF, false);
        self
          .regs
          .set_flag(CF, (a as u16) + ($d as u16) + (c as u16) > 0xFF);

        self.regs.set_a(v);

        self.regs.pc() + 1
      }};
    }

    macro_rules! alu_sub_a {
      ($d:expr) => {{
        let a = self.regs.a();
        let v = a.wrapping_sub($d);

        self.regs.set_flag(ZF, v == 0);
        self.regs.set_flag(HF, (a & 0x0F) < ($d & 0x0F));
        self.regs.set_flag(NF, true);
        self.regs.set_flag(CF, (a as u16) < ($d as u16));

        self.regs.set_a(v);

        self.regs.pc() + 1
      }};
    }

    macro_rules! alu_sbc_a {
      ($d:expr) => {{
        let a = self.regs.a();
        let c = if self.regs.get_flag(CF) { 1 } else { 0 };
        let v = a.wrapping_sub($d).wrapping_sub(c);

        self.regs.set_flag(ZF, v == 0);
        self.regs.set_flag(HF, (a & 0x0F) < ($d & 0x0F) + c);
        self.regs.set_flag(NF, true);
        self.regs.set_flag(CF, (a as u16) < ($d as u16));

        self.regs.set_a(v);

        self.regs.pc() + 1
      }};
    }

    macro_rules! alu_and_a {
      ($d:expr) => {{
        let a = self.regs.a();
        let v = a & $d;

        self.regs.set_flag(ZF, v == 0);
        self.regs.set_flag(CF, false);
        self.regs.set_flag(HF, true);
        self.regs.set_flag(NF, false);

        self.regs.set_a(v);

        self.regs.pc() + 1
      }};
    }

    macro_rules! alu_xor_a {
      ($d:expr) => {{
        let a = self.regs.a();
        let v = a ^ $d;

        self.regs.set_flag(ZF, v == 0);
        self.regs.set_flag(CF, false);
        self.regs.set_flag(HF, false);
        self.regs.set_flag(NF, false);

        self.regs.set_a(v);

        self.regs.pc() + 1
      }};
    }

    macro_rules! alu_or_a {
      ($d:expr) => {{
        let a = self.regs.a();
        let v = a | $d;

        self.regs.set_flag(ZF, v == 0);
        self.regs.set_flag(CF, false);
        self.regs.set_flag(HF, false);
        self.regs.set_flag(NF, false);

        self.regs.set_a(v);

        self.regs.pc() + 1
      }};
    }

    macro_rules! alu_add_hl {
      ($d:expr) => {{
        let hl = self.regs.hl();

        let v = hl.wrapping_add($d);

        self.regs.set_hl(v);

        self.regs.set_flag(NF, false);
        self.regs.set_flag(HF, self.overflow16(hl, $d, 11));
        self.regs.set_flag(CF, self.overflow16(hl, $d, 15));

        self.regs.pc() + 1
      }};
    }

    match opcode {
      // NOP, do nothing
      0x00 => pc + 1,

      // LD (N), SP
      0x08 => {
        mmu.write16(self.read_arg16(mmu) as usize, self.regs.sp());
        pc + 3
      }

      // // LD R, N
      0b0000_0001 => {
        self.regs.write16(BC, self.read_arg16(mmu));
        pc + 3
      }
      0b0001_0001 => {
        self.regs.write16(DE, self.read_arg16(mmu));
        pc + 3
      }
      0b0010_0001 => {
        self.regs.write16(HL, self.read_arg16(mmu));
        pc + 3
      }
      0b0011_0001 => {
        self.regs.write16(SP, self.read_arg16(mmu));
        pc + 3
      }

      // ADD HL, R
      0b0000_1001 => alu_add_hl!(self.regs.bc()),
      0b0001_1001 => alu_add_hl!(self.regs.de()),
      0b0010_1001 => alu_add_hl!(self.regs.hl()),
      0b0011_1001 => alu_add_hl!(self.regs.sp()),

      // LD (R), A
      0b0000_0010 => {
        mmu.write8(self.regs.bc() as usize, self.regs.a());
        pc + 1
      }
      0b0001_0010 => {
        mmu.write8(self.regs.de() as usize, self.regs.a());
        pc + 1
      }

      // LD A, (R)
      0b0000_1010 => {
        self.regs.write8(A, mmu.read8(self.regs.bc() as usize));
        pc + 1
      }
      0b0001_1010 => {
        self.regs.write8(A, mmu.read8(self.regs.de() as usize));
        pc + 1
      }

      // INC R
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
        let v = self.alu_inc(mmu.read8(ptr));
        mmu.write8(ptr, v);
        pc + 1
      }
      0b0011_1100 => {
        let v = self.alu_inc(self.regs.a());
        self.regs.set_a(v);
        pc + 1
      }

      // DEC D
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
        let v = self.alu_dec(mmu.read8(ptr));
        mmu.write8(ptr, v);
        pc + 1
      }
      0b0011_1101 => {
        let v = self.alu_dec(self.regs.a());
        self.regs.set_a(v);
        pc + 1
      }

      // LD D, N
      0b0000_0110 => {
        self.regs.set_b(self.read_arg8(mmu));
        pc + 2
      }
      0b0000_1110 => {
        self.regs.set_c(self.read_arg8(mmu));
        pc + 2
      }
      0b0001_0110 => {
        self.regs.set_d(self.read_arg8(mmu));
        pc + 2
      }
      0b0001_1110 => {
        self.regs.set_e(self.read_arg8(mmu));
        pc + 2
      }
      0b0010_0110 => {
        self.regs.set_h(self.read_arg8(mmu));
        pc + 2
      }
      0b0010_1110 => {
        self.regs.set_l(self.read_arg8(mmu));
        pc + 2
      }
      0b0011_0110 => {
        let ptr: usize = self.regs.hl() as usize;
        mmu.write8(ptr, self.read_arg8(mmu));
        pc + 2
      }
      0b0011_1110 => {
        self.regs.set_a(self.read_arg8(mmu));
        pc + 2
      }

      // RdCA
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

      // STOP
      0b0001_0000 => {
        // TODO
        1
      }

      // JR N
      0b0001_1000 => pc + self.read_arg8(mmu) as u16,

      // JR F, N
      0b0010_0000 => {
        if !self.regs.get_flag(ZF) {
          pc + self.read_arg8(mmu) as u16
        } else {
          pc + 2
        }
      }
      0b0010_1000 => {
        if self.regs.get_flag(ZF) {
          pc + self.read_arg8(mmu) as u16
        } else {
          pc + 2
        }
      }
      0b0011_0000 => {
        if !self.regs.get_flag(CF) {
          pc + self.read_arg8(mmu) as u16
        } else {
          pc + 2
        }
      }
      0b0011_1000 => {
        if self.regs.get_flag(CF) {
          pc + self.read_arg8(mmu) as u16
        } else {
          pc + 2
        }
      }

      // LDI (HL), A
      0b0010_0010 => {
        mmu.write8(self.regs.hl() as usize, self.regs.a());
        self.regs.set_hl(self.regs.hl().wrapping_add(1));
        pc + 1
      }

      // LDI A, (HL)
      0b0010_1010 => {
        self.regs.set_a(mmu.read8(self.regs.hl() as usize));
        self.regs.set_hl(self.regs.hl().wrapping_add(1));
        pc + 1
      }

      // LDD (HL), A
      0b0011_0010 => {
        mmu.write8(self.regs.hl() as usize, self.regs.a());
        self.regs.set_hl(self.regs.hl().wrapping_sub(1));
        pc + 1
      }

      // LDD A, (HL)
      0b0011_1010 => {
        self.regs.set_a(mmu.read8(self.regs.hl() as usize));
        self.regs.set_hl(self.regs.hl().wrapping_sub(1));
        pc + 1
      }

      // DAA
      0b0010_0111 => {
        self.alu_daa();
        pc + 1
      }

      // CPL
      0b0010_1111 => {
        self.regs.set_a(!self.regs.a());
        self.regs.set_flag(NF, true);
        self.regs.set_flag(HF, true);
        pc + 1
      }

      // SCF
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
        self.regs.set_b(mmu.read8(self.regs.hl() as usize)); // no-op
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
        self.regs.set_c(mmu.read8(self.regs.hl() as usize)); // no-op
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
        self.regs.set_d(mmu.read8(self.regs.hl() as usize)); // no-op
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
        self.regs.set_e(mmu.read8(self.regs.hl() as usize)); // no-op
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
        self.regs.set_h(mmu.read8(self.regs.hl() as usize)); // no-op
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
        self.regs.set_l(mmu.read8(self.regs.hl() as usize)); // no-op
        pc + 1
      }
      0b0110_1111 => {
        self.regs.set_l(self.regs.a());
        pc + 1
      }

      // LD (HL), r8
      0b0111_0000 => {
        mmu.write8(self.regs.hl() as usize, self.regs.b());
        pc + 1
      }
      0b0111_0001 => {
        mmu.write8(self.regs.hl() as usize, self.regs.c());
        pc + 1
      }
      0b0111_0010 => {
        mmu.write8(self.regs.hl() as usize, self.regs.d());
        pc + 1
      }
      0b0111_0011 => {
        mmu.write8(self.regs.hl() as usize, self.regs.e());
        pc + 1
      }
      0b0111_0100 => {
        mmu.write8(self.regs.hl() as usize, self.regs.h());
        pc + 1
      }
      0b0111_0101 => {
        mmu.write8(self.regs.hl() as usize, self.regs.l());
        pc + 1
      }
      0b0111_0111 => {
        mmu.write8(self.regs.hl() as usize, self.regs.a());
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
        self.regs.set_a(mmu.read8(self.regs.hl() as usize)); // no-op
        pc + 1
      }
      0b0111_1111 => {
        self.regs.set_a(self.regs.a());
        pc + 1
      }

      // HALT
      0b0111_0110 => {
        // TODO
        pc + 1
      }

      // ADD A, D
      0b1000_0000 => alu_add_a!(self.regs.b()),
      0b1000_0001 => alu_add_a!(self.regs.c()),
      0b1000_0010 => alu_add_a!(self.regs.d()),
      0b1000_0011 => alu_add_a!(self.regs.e()),
      0b1000_0100 => alu_add_a!(self.regs.h()),
      0b1000_0101 => alu_add_a!(self.regs.l()),
      0b1000_0110 => alu_add_a!(mmu.read8(self.regs.hl() as usize)),
      0b1000_0111 => alu_add_a!(self.regs.a()),
      // ADD A, N
      0b1100_0110 => {
        alu_add_a!(self.read_arg8(mmu));
        2
      }

      // ADC A, D
      0b1000_1000 => alu_adc_a!(self.regs.b()),
      0b1000_1001 => alu_adc_a!(self.regs.c()),
      0b1000_1010 => alu_adc_a!(self.regs.d()),
      0b1000_1011 => alu_adc_a!(self.regs.e()),
      0b1000_1100 => alu_adc_a!(self.regs.h()),
      0b1000_1101 => alu_adc_a!(self.regs.l()),
      0b1000_1110 => alu_adc_a!(mmu.read8(self.regs.hl() as usize)),
      0b1000_1111 => alu_adc_a!(self.regs.a()),
      // ADC A, N
      0b1100_1110 => {
        alu_adc_a!(self.read_arg8(mmu));
        2
      }

      // SUB A, D
      0b1001_0000 => alu_sub_a!(self.regs.b()),
      0b1001_0001 => alu_sub_a!(self.regs.c()),
      0b1001_0010 => alu_sub_a!(self.regs.d()),
      0b1001_0011 => alu_sub_a!(self.regs.e()),
      0b1001_0100 => alu_sub_a!(self.regs.h()),
      0b1001_0101 => alu_sub_a!(self.regs.l()),
      0b1001_0110 => alu_sub_a!(mmu.read8(self.regs.hl() as usize)),
      0b1001_0111 => alu_sub_a!(self.regs.a()),
      // SUB A, N
      0b1101_0110 => {
        alu_sub_a!(self.read_arg8(mmu));
        2
      }

      // SUB A, D
      0b1001_1000 => alu_sbc_a!(self.regs.b()),
      0b1001_1001 => alu_sbc_a!(self.regs.c()),
      0b1001_1010 => alu_sbc_a!(self.regs.d()),
      0b1001_1011 => alu_sbc_a!(self.regs.e()),
      0b1001_1100 => alu_sbc_a!(self.regs.h()),
      0b1001_1101 => alu_sbc_a!(self.regs.l()),
      0b1001_1110 => alu_sbc_a!(mmu.read8(self.regs.hl() as usize)),
      0b1001_1111 => alu_sbc_a!(self.regs.a()),
      // sbc A, N
      0b1101_1110 => {
        alu_sbc_a!(self.read_arg8(mmu));
        2
      }

      // AND A, D
      0b1010_0000 => alu_and_a!(self.regs.b()),
      0b1010_0001 => alu_and_a!(self.regs.c()),
      0b1010_0010 => alu_and_a!(self.regs.d()),
      0b1010_0011 => alu_and_a!(self.regs.e()),
      0b1010_0100 => alu_and_a!(self.regs.h()),
      0b1010_0101 => alu_and_a!(self.regs.l()),
      0b1010_0110 => alu_and_a!(mmu.read8(self.regs.hl() as usize)),
      0b1010_0111 => alu_and_a!(self.regs.a()),
      // ADD A, N
      0b1110_0110 => {
        alu_and_a!(self.read_arg8(mmu));
        2
      }

      // XOR A, D
      0b1010_1000 => alu_xor_a!(self.regs.b()),
      0b1010_1001 => alu_xor_a!(self.regs.c()),
      0b1010_1010 => alu_xor_a!(self.regs.d()),
      0b1010_1011 => alu_xor_a!(self.regs.e()),
      0b1010_1100 => alu_xor_a!(self.regs.h()),
      0b1010_1101 => alu_xor_a!(self.regs.l()),
      0b1010_1110 => alu_xor_a!(mmu.read8(self.regs.hl() as usize)),
      0b1010_1111 => alu_xor_a!(self.regs.a()),
      // ADD A, N
      0b1110_1110 => {
        alu_xor_a!(self.read_arg8(mmu));
        2
      }

      // XOR A, D
      0b1011_0000 => alu_or_a!(self.regs.b()),
      0b1011_0001 => alu_or_a!(self.regs.c()),
      0b1011_0010 => alu_or_a!(self.regs.d()),
      0b1011_0011 => alu_or_a!(self.regs.e()),
      0b1011_0100 => alu_or_a!(self.regs.h()),
      0b1011_0101 => alu_or_a!(self.regs.l()),
      0b1011_0110 => alu_or_a!(mmu.read8(self.regs.hl() as usize)),
      0b1011_0111 => alu_or_a!(self.regs.a()),
      // OR A, N
      0b1111_0110 => {
        alu_or_a!(self.read_arg8(mmu));
        2
      }

      // POP R
      0b1100_0001 => {
        let v = self.pop(mmu);
        self.regs.set_bc(v);
        pc + 1
      }
      0b1101_0001 => {
        let v = self.pop(mmu);
        self.regs.set_de(v);
        pc + 1
      }
      0b1110_0001 => {
        let v = self.pop(mmu);
        self.regs.set_hl(v);
        pc + 1
      }
      0b1111_0001 => {
        let v = self.pop(mmu);
        self.regs.set_af(v);
        pc + 1
      }

      //  R
      0b1100_0101 => {
        self.push(self.regs.bc(), mmu);
        pc + 1
      }
      0b1101_0101 => {
        self.push(self.regs.de(), mmu);
        pc + 1
      }
      0b1110_0101 => {
        self.push(self.regs.hl(), mmu);
        pc + 1
      }
      0b1111_0101 => {
        self.push(self.regs.af(), mmu);
        pc + 1
      }

      // RET NZ
      0b1100_0000 => {
        if !self.regs.get_flag(ZF) {
          self.pop(mmu)
        } else {
          pc + 1
        }
      }
      // RET Z
      0b1100_1000 => {
        if self.regs.get_flag(ZF) {
          self.pop(mmu)
        } else {
          pc + 1
        }
      }
      // RET NC
      0b1101_0000 => {
        if !self.regs.get_flag(CF) {
          self.pop(mmu)
        } else {
          pc + 1
        }
      }
      // RET C
      0b1101_1000 => {
        if self.regs.get_flag(CF) {
          self.pop(mmu)
        } else {
          pc + 1
        }
      }

      // RET
      0b1100_1001 => self.pop(mmu),

      // RETI
      0b1101_1001 => {
        self.interrupts = 1;
        self.pop(mmu)
      }

      // JP NZ, N
      0b1100_0010 => {
        if !self.regs.get_flag(ZF) {
          self.read_arg16(mmu)
        } else {
          pc + 3
        }
      }
      // JP Z, N
      0b1100_1010 => {
        if self.regs.get_flag(ZF) {
          self.read_arg16(mmu)
        } else {
          pc + 3
        }
      }
      // JP NC, N
      0b1101_0010 => {
        if !self.regs.get_flag(CF) {
          self.read_arg16(mmu)
        } else {
          pc + 3
        }
      }
      // JP C N
      0b1101_1010 => {
        if self.regs.get_flag(CF) {
          self.read_arg16(mmu)
        } else {
          pc + 3
        }
      }

      // JP N
      0b1100_0011 => self.read_arg16(mmu),

      // CALL F, N
      // CALL NZ, N
      0b1100_0100 => {
        if !self.regs.get_flag(ZF) {
          self.push(pc + 3, mmu);
          self.read_arg16(mmu)
        } else {
          pc + 3
        }
      }
      // CALL Z, N
      0b1100_1100 => {
        if self.regs.get_flag(ZF) {
          self.push(pc + 3, mmu);
          self.read_arg16(mmu)
        } else {
          pc + 3
        }
      }
      // CALL NC, N
      0b1101_0100 => {
        if !self.regs.get_flag(CF) {
          self.push(pc + 3, mmu);
          self.read_arg16(mmu)
        } else {
          pc + 3
        }
      }
      // CALL C, N
      0b1101_1100 => {
        if self.regs.get_flag(CF) {
          self.push(pc + 3, mmu);
          self.read_arg16(mmu)
        } else {
          pc + 3
        }
      }

      // CALL N
      0b1100_1101 => {
        self.push(pc + 3, mmu);
        self.read_arg16(mmu)
      }

      0b1110_1000 => {
        let v = self.alu_add16imm(self.regs.sp(), mmu);
        self.regs.set_sp(v);
        pc + 2
      }

      // LD HL, SP + N
      0b1111_1000 => {
        let v = self.alu_add16imm(self.regs.sp(), mmu);
        self.regs.set_hl(v);
        pc + 2
      }

      // LD (FF00+N), A
      0b1110_0000 => {
        let ptr = (0xFF00 | self.read_arg8(mmu) as u16) as usize;
        mmu.write8(ptr, self.regs.a());
        pc + 2
      }

      // LD A, (FF00+N)
      0b1111_0000 => {
        let ptr = (0xFF00 | self.read_arg8(mmu) as u16) as usize;
        self.regs.set_a(mmu.read8(ptr));
        pc + 2
      }

      // LD (C), A
      0b1110_0010 => {
        let ptr = (0xFF00 | self.regs.c() as u16) as usize;
        mmu.write8(ptr, self.regs.a());
        pc + 2
      }

      // LD A, (C)
      0b1111_0010 => {
        let ptr = (0xFF00 | self.regs.c() as u16) as usize;
        self.regs.set_a(mmu.read8(ptr));
        pc + 2
      }

      // LD (N), A
      0b1110_1010 => {
        let ptr = self.read_arg16(mmu) as usize;
        mmu.write8(ptr, self.regs.a());
        pc + 2
      }

      // LD A, (N)
      0b1111_1010 => {
        let ptr = self.read_arg16(mmu) as usize;
        self.regs.set_a(mmu.read8(ptr));
        pc + 2
      }

      // JP HL
      0b1110_1001 => self.regs.hl(),

      // LD SP, HL
      0b1111_1001 => {
        self.regs.set_sp(self.regs.hl());
        pc + 1
      }

      // DI
      0b1111_0011 => {
        self.interrupts = 0;
        pc + 1
      }

      // EI
      0b1111_1011 => {
        self.interrupts = 1;
        pc + 1
      }

      // read instr from byte 2
      0xCB => self.exec_cb(self.read_arg8(mmu), pc, mmu),

      _ => self.i_unknown(opcode),
    }
  }

  fn exec_cb(&mut self, opcode: u8, pc: u16, mmu: &mut MMU) -> u16 {
    match opcode {
      // RLC D
      0b0000_0000 => {
        let v = self.alu_rlc(self.regs.b());
        self.regs.set_b(v);
      }
      0b0000_0001 => {
        let v = self.alu_rlc(self.regs.c());
        self.regs.set_c(v);
      }
      0b0000_0010 => {
        let v = self.alu_rlc(self.regs.d());
        self.regs.set_d(v);
      }
      0b0000_0011 => {
        let v = self.alu_rlc(self.regs.e());
        self.regs.set_e(v);
      }
      0b0000_0100 => {
        let v = self.alu_rlc(self.regs.h());
        self.regs.set_h(v);
      }
      0b0000_0101 => {
        let v = self.alu_rlc(self.regs.l());
        self.regs.set_l(v);
      }
      0b0000_0110 => {
        let ptr = self.regs.hl() as usize;
        let v = self.alu_rlc(mmu.read8(ptr));
        mmu.write8(ptr, v);
      }
      0b0000_0111 => {
        let v = self.alu_rlc(self.regs.a());
        self.regs.set_a(v);
      }

      // RRC D
      0b0000_1000 => {
        let v = self.alu_rrc(self.regs.b());
        self.regs.set_b(v);
      }
      0b0000_1001 => {
        let v = self.alu_rrc(self.regs.c());
        self.regs.set_c(v);
      }
      0b0000_1010 => {
        let v = self.alu_rrc(self.regs.d());
        self.regs.set_d(v);
      }
      0b0000_1011 => {
        let v = self.alu_rrc(self.regs.e());
        self.regs.set_e(v);
      }
      0b0000_1100 => {
        let v = self.alu_rrc(self.regs.h());
        self.regs.set_h(v);
      }
      0b0000_1101 => {
        let v = self.alu_rrc(self.regs.l());
        self.regs.set_l(v);
      }
      0b0000_1110 => {
        let ptr = self.regs.hl() as usize;
        let v = self.alu_rrc(mmu.read8(ptr));
        mmu.write8(ptr, v);
      }
      0b0000_1111 => {
        let v = self.alu_rrc(self.regs.a());
        self.regs.set_a(v);
      }

      // RL D
      0b0001_0000 => {
        let v = self.alu_rl(self.regs.b());
        self.regs.set_b(v);
      }
      0b0001_0001 => {
        let v = self.alu_rl(self.regs.c());
        self.regs.set_c(v);
      }
      0b0001_0010 => {
        let v = self.alu_rl(self.regs.d());
        self.regs.set_d(v);
      }
      0b0001_0011 => {
        let v = self.alu_rl(self.regs.e());
        self.regs.set_e(v);
      }
      0b0001_0100 => {
        let v = self.alu_rl(self.regs.h());
        self.regs.set_h(v);
      }
      0b0001_0101 => {
        let v = self.alu_rl(self.regs.l());
        self.regs.set_l(v);
      }
      0b0001_0110 => {
        let ptr = self.regs.hl() as usize;
        let v = self.alu_rl(mmu.read8(ptr));
        mmu.write8(ptr, v);
      }
      0b0001_0111 => {
        let v = self.alu_rl(self.regs.a());
        self.regs.set_a(v);
      }

      // RR D
      0b0001_1000 => {
        let v = self.alu_rr(self.regs.b());
        self.regs.set_b(v);
      }
      0b0001_1001 => {
        let v = self.alu_rr(self.regs.c());
        self.regs.set_c(v);
      }
      0b0001_1010 => {
        let v = self.alu_rr(self.regs.d());
        self.regs.set_d(v);
      }
      0b0001_1011 => {
        let v = self.alu_rr(self.regs.e());
        self.regs.set_e(v);
      }
      0b0001_1100 => {
        let v = self.alu_rr(self.regs.h());
        self.regs.set_h(v);
      }
      0b0001_1101 => {
        let v = self.alu_rr(self.regs.l());
        self.regs.set_l(v);
      }
      0b0001_1110 => {
        let ptr = self.regs.hl() as usize;
        let v = self.alu_rr(mmu.read8(ptr));
        mmu.write8(ptr, v);
      }
      0b0001_1111 => {
        let v = self.alu_rr(self.regs.a());
        self.regs.set_a(v);
      }

      // SLA D
      0b0010_0000 => {
        let v = self.alu_sla(self.regs.b());
        self.regs.set_b(v);
      }
      0b0010_0001 => {
        let v = self.alu_sla(self.regs.c());
        self.regs.set_c(v);
      }
      0b0010_0010 => {
        let v = self.alu_sla(self.regs.d());
        self.regs.set_d(v);
      }
      0b0010_0011 => {
        let v = self.alu_sla(self.regs.e());
        self.regs.set_e(v);
      }
      0b0010_0100 => {
        let v = self.alu_sla(self.regs.h());
        self.regs.set_h(v);
      }
      0b0010_0101 => {
        let v = self.alu_sla(self.regs.l());
        self.regs.set_l(v);
      }
      0b0010_0110 => {
        let ptr = self.regs.hl() as usize;
        let v = self.alu_sla(mmu.read8(ptr));
        mmu.write8(ptr, v);
      }
      0b0010_0111 => {
        let v = self.alu_sla(self.regs.a());
        self.regs.set_a(v);
      }

      // SRA D
      0b0010_1000 => {
        let v = self.alu_sra(self.regs.b());
        self.regs.set_b(v);
      }
      0b0010_1001 => {
        let v = self.alu_sra(self.regs.c());
        self.regs.set_c(v);
      }
      0b0010_1010 => {
        let v = self.alu_sra(self.regs.d());
        self.regs.set_d(v);
      }
      0b0010_1011 => {
        let v = self.alu_sra(self.regs.e());
        self.regs.set_e(v);
      }
      0b0010_1100 => {
        let v = self.alu_sra(self.regs.h());
        self.regs.set_h(v);
      }
      0b0010_1101 => {
        let v = self.alu_sra(self.regs.l());
        self.regs.set_l(v);
      }
      0b0010_1110 => {
        let ptr = self.regs.hl() as usize;
        let v = self.alu_sra(mmu.read8(ptr));
        mmu.write8(ptr, v);
      }
      0b0010_1111 => {
        let v = self.alu_sra(self.regs.a());
        self.regs.set_a(v);
      }

      // SWAP D
      0b0011_0000 => {
        let v = self.alu_swap(self.regs.b());
        self.regs.set_b(v);
      }
      0b0011_0001 => {
        let v = self.alu_swap(self.regs.c());
        self.regs.set_c(v);
      }
      0b0011_0010 => {
        let v = self.alu_swap(self.regs.d());
        self.regs.set_d(v);
      }
      0b0011_0011 => {
        let v = self.alu_swap(self.regs.e());
        self.regs.set_e(v);
      }
      0b0011_0100 => {
        let v = self.alu_swap(self.regs.h());
        self.regs.set_h(v);
      }
      0b0011_0101 => {
        let v = self.alu_swap(self.regs.l());
        self.regs.set_l(v);
      }
      0b0011_0110 => {
        let ptr = self.regs.hl() as usize;
        let v = self.alu_swap(mmu.read8(ptr));
        mmu.write8(ptr, v);
      }
      0b0011_0111 => {
        let v = self.alu_swap(self.regs.a());
        self.regs.set_a(v);
      }

      // SRL D
      0b0011_1000 => {
        let v = self.alu_srl(self.regs.b());
        self.regs.set_b(v);
      }
      0b0011_1001 => {
        let v = self.alu_srl(self.regs.c());
        self.regs.set_c(v);
      }
      0b0011_1010 => {
        let v = self.alu_srl(self.regs.d());
        self.regs.set_d(v);
      }
      0b0011_1011 => {
        let v = self.alu_srl(self.regs.e());
        self.regs.set_e(v);
      }
      0b0011_1100 => {
        let v = self.alu_srl(self.regs.h());
        self.regs.set_h(v);
      }
      0b0011_1101 => {
        let v = self.alu_srl(self.regs.l());
        self.regs.set_l(v);
      }
      0b0011_1110 => {
        let ptr = self.regs.hl() as usize;
        let v = self.alu_srl(mmu.read8(ptr));
        mmu.write8(ptr, v);
      }
      0b0011_1111 => {
        let v = self.alu_srl(self.regs.a());
        self.regs.set_a(v);
      }

      // BIT N, (HL)
      _ if opcode_match(opcode, 0b1100_0111, 0b0100_0110) => {
        let n = self.cb_alu_n(opcode);
        let v = mmu.read8(self.regs.hl() as usize);

        self.alu_bit(n, v);
      }

      // BIT N, D
      _ if opcode_match(opcode, 0b1100_0000, 0b0100_0000) => {
        let n = self.cb_alu_n(opcode);
        let reg = self.cb_alu_reg(opcode);
        let v = self.regs.read8(reg);
        println!("{} {}", n, v);

        self.alu_bit(n, v);
      }

      // RES N, (HL)
      _ if opcode_match(opcode, 0b1100_0111, 0b1000_0110) => {
        let n = self.cb_alu_n(opcode);
        let v = mmu.read8(self.regs.hl() as usize);

        mmu.write8(self.regs.hl() as usize, v & !(1 << n));
      }

      // RES N, D
      _ if opcode_match(opcode, 0b1100_0000, 0b1000_0000) => {
        let n = self.cb_alu_n(opcode);
        let reg = self.cb_alu_reg(opcode);
        let v = self.regs.read8(reg);

        self.regs.write8(reg, v & !(1 << n));
      }

      // SET N, (HL)
      _ if opcode_match(opcode, 0b1100_0111, 0b1100_0110) => {
        let n = self.cb_alu_n(opcode);
        let v = mmu.read8(self.regs.hl() as usize);

        mmu.write8(self.regs.hl() as usize, v | (1 << n));
      }

      // SET N, D
      _ if opcode_match(opcode, 0b1100_0000, 0b1100_0000) => {
        let n = self.cb_alu_n(opcode);
        let reg = self.cb_alu_reg(opcode);
        let v = self.regs.read8(reg);

        self.regs.write8(reg, v | (1 << n));
      }

      _ => {
        self.i_unknown(opcode);
      }
    };

    pc + 2
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
    self.regs.set_flag(ZF, r == 0);
    self.regs.set_flag(CF, c);
    r
  }

  fn alu_rl(&mut self, v: u8) -> u8 {
    let c = v & 0x80 == 0x80;
    let r = (v << 1) | (if self.regs.get_flag(CF) { 1 } else { 0 });

    self.regs.set_flag(HF, false);
    self.regs.set_flag(NF, false);
    self.regs.set_flag(ZF, r == 0);
    self.regs.set_flag(CF, c);
    r
  }

  fn alu_rrc(&mut self, v: u8) -> u8 {
    let c = v & 0x01 == 0x01;
    let r = (v >> 1) | (if c { 0x80 } else { 0 });

    self.regs.set_flag(HF, false);
    self.regs.set_flag(NF, false);
    self.regs.set_flag(ZF, r == 0);
    self.regs.set_flag(CF, c);
    r
  }

  fn alu_rr(&mut self, v: u8) -> u8 {
    let c = v & 0x01 == 0x01;
    let r = (v >> 1) | (if self.regs.get_flag(CF) { 0x80 } else { 0 });

    self.regs.set_flag(HF, false);
    self.regs.set_flag(NF, false);
    self.regs.set_flag(ZF, r == 0);
    self.regs.set_flag(CF, c);
    r
  }

  fn alu_sla(&mut self, a: u8) -> u8 {
    let c = a & 0x80 == 0x80;
    let r = a << 1;

    self.regs.set_flag(HF, false);
    self.regs.set_flag(NF, false);
    self.regs.set_flag(ZF, r == 0);
    self.regs.set_flag(CF, c);

    r
  }

  fn alu_sra(&mut self, a: u8) -> u8 {
    let c = a & 0x80 == 0x80;
    let r = a >> 1 | (a & 0x80);

    self.regs.set_flag(HF, false);
    self.regs.set_flag(NF, false);
    self.regs.set_flag(ZF, r == 0);
    self.regs.set_flag(CF, c);

    r
  }

  fn alu_swap(&mut self, a: u8) -> u8 {
    self.regs.set_flag(ZF, a == 0);
    self.regs.set_flag(CF, false);
    self.regs.set_flag(HF, false);
    self.regs.set_flag(NF, false);

    (a >> 4) | (a << 4)
  }

  fn alu_srl(&mut self, a: u8) -> u8 {
    let c = a & 0x01 == 0x01;
    let r = a >> 1;

    self.regs.set_flag(NF, false);
    self.regs.set_flag(HF, false);
    self.regs.set_flag(ZF, r == 0);
    self.regs.set_flag(CF, c);

    r
  }

  fn alu_bit(&mut self, n: u8, v: u8) {
    let r = v & (1 << (n as u32)) == 0;

    self.regs.set_flag(NF, false);
    self.regs.set_flag(HF, true);
    self.regs.set_flag(ZF, r);
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

  fn alu_add16imm(&mut self, r: u16, mmu: &MMU) -> u16 {
    let d = self.read_arg8(mmu) as u16;

    let v = r.wrapping_add(d);

    self.regs.set_flag(ZF, v == 0);
    self.regs.set_flag(HF, (r & 0x000F) + (d & 0x000F) > 0x000F);
    self.regs.set_flag(NF, false);
    self.regs.set_flag(CF, (r & 0x00FF) + (d & 0x00FF) > 0x00FF);

    v
  }

  fn push(&mut self, value: u16, mmu: &mut MMU) {
    self.regs.set_sp(self.regs.sp() - 2);
    mmu.write16(self.regs.sp() as usize, value);
  }

  fn pop(&mut self, mmu: &mut MMU) -> u16 {
    let v = mmu.read16(self.regs.sp() as usize);
    self.regs.set_sp(self.regs.sp() + 2);

    v
  }

  fn i_unknown(&self, opcode: u8) -> u16 {
    panic!(
      "Failed to execute unknown opcode: 0x{:02x} (0b{0:b})",
      opcode
    );
  }

  fn read_arg8(&self, mmu: &MMU) -> u8 {
    let pc = self.regs.read16(PC);

    mmu.read8((pc + 1) as usize)
  }

  fn read_arg16(&self, mmu: &MMU) -> u16 {
    let pc = self.regs.read16(PC);

    mmu.read16((pc + 1) as usize)
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

  fn cb_alu_reg(&self, reg: u8) -> Register8 {
    match reg & 0x07 {
      0x0 => B,
      0x1 => C,
      0x2 => D,
      0x3 => E,
      0x4 => H,
      0x5 => L,
      0x7 => A,

      _ => panic!("Unkonwn alu_val register code: 0x{:x}", reg),
    }
  }

  fn cb_alu_n(&self, reg: u8) -> u8 {
    (reg & 0b0011_1000) >> 3
  }
}

fn opcode_match(opcode: u8, mask: u8, expectation: u8) -> bool {
  opcode & mask == expectation
}

#[cfg(test)]
mod tests {
  use super::*;

  macro_rules! exec {
    ($cpu:expr, $mmu:expr, $instr:expr) => {{
      let pc = $cpu.regs.pc() as usize;
      $mmu._load8(pc, $instr);
      $cpu.exec(&mut $mmu);
    }};

    ($cpu:expr, $mmu:expr,$instr:expr, arg8 => $arg8:expr) => {{
      let pc = $cpu.regs.pc() as usize;
      $mmu._load8(pc, $instr);
      $mmu._load8(pc + 1, $arg8);
      $cpu.exec(&mut $mmu);
    }};

    ($cpu:expr,$mmu:expr, $instr:expr, arg16 => $arg16:expr) => {{
      let pc = $cpu.regs.pc() as usize;
      $mmu._load8(pc, $instr);
      $mmu._load16(pc + 1, $arg16);
      $cpu.exec(&mut $mmu);
    }};
  }

  macro_rules! exec_cb {
    ($cpu:expr, $mmu:expr,$instr:expr) => {
      exec!($cpu, $mmu, 0xCB, arg8 => $instr)
    };
  }

  fn new_test_cpu() -> (CPU, MMU) {
    (CPU::new(), MMU::new(false))
  }

  #[test]
  fn new_cpu() {
    let (cpu, mmu) = new_test_cpu();

    assert_eq!(cpu.regs.read16(Register16::BC), 0);
  }

  #[test]
  fn opcode_nop() {
    let (mut cpu, mut mmu) = new_test_cpu();

    exec!(cpu, mmu, 0b0000_0000);

    assert_eq!(cpu.regs.read16(Register16::PC), 1);
  }

  #[test]
  fn opcode_ld_ptr16_sp() {
    let (mut cpu, mut mmu) = new_test_cpu();
    cpu.regs.set_sp(2047);

    exec!(cpu, mmu, 0b0000_1000, arg16 => 0xff90);

    assert_eq!(mmu.read16(0xff90), 2047);
  }

  #[test]
  fn opcode_ld_r16_n16() {
    let (mut cpu, mut mmu) = new_test_cpu();

    exec!(cpu, mmu, 0x01, arg16 => 511);
    exec!(cpu, mmu, 0x11, arg16 => 1023);
    exec!(cpu, mmu, 0x21, arg16 => 2047);
    exec!(cpu, mmu, 0x31, arg16 => 4095);

    assert_eq!(cpu.regs.read16(BC), 511);
    assert_eq!(cpu.regs.read16(DE), 1023);
    assert_eq!(cpu.regs.read16(HL), 2047);
    assert_eq!(cpu.regs.read16(SP), 4095);
  }

  #[test]
  fn opcode_add_hl_r16() {
    let (mut cpu, mut mmu) = new_test_cpu();

    // ADD HL, BC
    cpu.regs.write16(HL, 128);
    cpu.regs.write16(BC, 127);
    exec!(cpu, mmu, 0x09);
    assert_eq!(cpu.regs.read16(HL), 255);

    // ADD HL, DE
    cpu.regs.write16(HL, 256);
    cpu.regs.write16(DE, 255);
    exec!(cpu, mmu, 0x19);
    assert_eq!(cpu.regs.read16(HL), 511);

    // ADD HL, HL
    cpu.regs.write16(HL, 511);
    exec!(cpu, mmu, 0x29);
    assert_eq!(cpu.regs.read16(HL), 1022);

    // ADD HL, SP
    cpu.regs.write16(HL, 1024);
    cpu.regs.write16(SP, 1023);
    exec!(cpu, mmu, 0x39);
    assert_eq!(cpu.regs.read16(HL), 2047);
  }

  #[test]
  fn opcode_add_hl_r16_flags() {
    let (mut cpu, mut mmu) = new_test_cpu();

    // carry from bit 11
    cpu.regs.write16(HL, 0b0000_1000_0000_0000);
    cpu.regs.write16(BC, 0b0000_1000_0000_0000);
    exec!(cpu, mmu, 0x09);

    assert_eq!(cpu.regs.get_flag(NF), false);
    assert_eq!(cpu.regs.get_flag(HF), true);
    assert_eq!(cpu.regs.get_flag(CF), false);

    // carry from bit 15
    cpu.regs.write16(HL, 0b1000_0000_0000_0000);
    cpu.regs.write16(BC, 0b1000_0000_0000_0000);
    exec!(cpu, mmu, 0x09);

    assert_eq!(cpu.regs.get_flag(NF), false);
    assert_eq!(cpu.regs.get_flag(HF), false);
    assert_eq!(cpu.regs.get_flag(CF), true);

    // carry from bit 11 and 15
    cpu.regs.write16(HL, 0b1000_1000_0000_0000);
    cpu.regs.write16(BC, 0b1000_1000_0000_0000);
    exec!(cpu, mmu, 0x09);

    assert_eq!(cpu.regs.get_flag(NF), false);
    assert_eq!(cpu.regs.get_flag(HF), true);
    assert_eq!(cpu.regs.get_flag(CF), true);

    // carry from bit 11 and 15 indirectly
    cpu.regs.write16(HL, 0b1100_0100_0000_0000);
    cpu.regs.write16(BC, 0b0100_1100_0000_0000);
    exec!(cpu, mmu, 0x09);

    assert_eq!(cpu.regs.get_flag(NF), false);
    assert_eq!(cpu.regs.get_flag(HF), true);
    assert_eq!(cpu.regs.get_flag(CF), true);
  }

  #[test]
  fn opcode_ld_r16_a() {
    let (mut cpu, mut mmu) = new_test_cpu();

    // LD BC, A
    cpu.regs.write8(A, 127);
    cpu.regs.write16(BC, 0xff90);
    exec!(cpu, mmu, 0x02);
    assert_eq!(mmu.read8(0xff90), 127);

    // LD DE, A
    cpu.regs.write8(A, 63);
    cpu.regs.write16(DE, 0xff90);
    exec!(cpu, mmu, 0x12);
    assert_eq!(mmu.read8(0xff90), 63);
  }

  #[test]
  fn opcode_ld_a_r16() {
    let (mut cpu, mut mmu) = new_test_cpu();

    // LD BC, A
    mmu.write8(0xff90, 127);
    cpu.regs.write16(BC, 0xff90);
    exec!(cpu, mmu, 0x0a);
    assert_eq!(cpu.regs.read8(A), 127);

    // LD DE, A
    mmu.write8(0xff90, 63);
    cpu.regs.write16(DE, 0xff90);
    exec!(cpu, mmu, 0x1a);
    assert_eq!(cpu.regs.read8(A), 63);
  }

  #[test]
  fn opcode_inc_r16() {
    let (mut cpu, mut mmu) = new_test_cpu();

    // INC BC
    cpu.regs.write16(BC, 257);
    exec!(cpu, mmu, 0x03);
    assert_eq!(cpu.regs.read16(BC), 258);

    // INC DE
    cpu.regs.write16(DE, 511);
    exec!(cpu, mmu, 0x13);
    assert_eq!(cpu.regs.read16(DE), 512);

    // INC HL
    cpu.regs.write16(HL, 1023);
    exec!(cpu, mmu, 0x23);
    assert_eq!(cpu.regs.read16(HL), 1024);

    // INC SP
    cpu.regs.write16(SP, 2047);
    exec!(cpu, mmu, 0x33);
    assert_eq!(cpu.regs.read16(SP), 2048);
  }

  #[test]
  fn opcode_dec_r16() {
    let (mut cpu, mut mmu) = new_test_cpu();

    // INC BC
    cpu.regs.write16(BC, 257);
    exec!(cpu, mmu, 0x0b);
    assert_eq!(cpu.regs.read16(BC), 256);

    // INC DE
    cpu.regs.write16(DE, 511);
    exec!(cpu, mmu, 0x1b);
    assert_eq!(cpu.regs.read16(DE), 510);

    // INC HL
    cpu.regs.write16(HL, 1023);
    exec!(cpu, mmu, 0x2b);
    assert_eq!(cpu.regs.read16(HL), 1022);

    // INC SP
    cpu.regs.write16(SP, 2047);
    exec!(cpu, mmu, 0x3b);
    assert_eq!(cpu.regs.read16(SP), 2046);
  }

  #[test]
  fn opcode_inc_r8() {
    let (mut cpu, mut mmu) = new_test_cpu();

    // INC B
    cpu.regs.set_b(1);
    exec!(cpu, mmu, 0x04);
    assert_eq!(cpu.regs.read8(B), 2);

    // INC C
    cpu.regs.set_c(2);
    exec!(cpu, mmu, 0x0c);
    assert_eq!(cpu.regs.read8(C), 3);

    // INC D
    cpu.regs.set_d(3);
    exec!(cpu, mmu, 0x14);
    assert_eq!(cpu.regs.read8(D), 4);
    // INC E
    cpu.regs.set_e(4);
    exec!(cpu, mmu, 0x1c);
    assert_eq!(cpu.regs.read8(E), 5);
    // INC H
    cpu.regs.set_h(5);
    exec!(cpu, mmu, 0x24);
    assert_eq!(cpu.regs.read8(H), 6);
    // INC L
    cpu.regs.set_l(6);
    exec!(cpu, mmu, 0x2c);
    assert_eq!(cpu.regs.read8(L), 7);

    // INC (HL)
    cpu.regs.set_hl(0xff90);
    mmu.write8(0xff90, 7);
    exec!(cpu, mmu, 0x34);
    assert_eq!(mmu.read8(0xff90), 8);

    // INC A
    cpu.regs.set_a(8);
    exec!(cpu, mmu, 0x3c);
    assert_eq!(cpu.regs.read8(A), 9);
  }

  #[test]
  fn opcode_inc_r8_flags() {
    let (mut cpu, mut mmu) = new_test_cpu();

    // NF is set to false
    cpu.regs.set_a(8);
    exec!(cpu, mmu, 0x3c);
    assert_eq!(cpu.regs.get_flag(NF), false);

    // ZF is set to true if result is 0
    cpu.regs.set_a(0xFF);
    exec!(cpu, mmu, 0x3c);
    assert_eq!(cpu.regs.get_flag(ZF), true);

    // ZF is set to false if result is not 0
    cpu.regs.set_a(0xFE);
    exec!(cpu, mmu, 0x3c);
    assert_eq!(cpu.regs.get_flag(ZF), false);

    // HF is set to true if overflows from bit 3
    cpu.regs.set_a(0b0000_1111);
    exec!(cpu, mmu, 0x3c);
    assert_eq!(cpu.regs.get_flag(HF), true);

    // HF is set to false if does not overflow from bit 3
    cpu.regs.set_a(0b0000_0111);
    exec!(cpu, mmu, 0x3c);
    assert_eq!(cpu.regs.get_flag(HF), false);
  }

  #[test]
  fn opcode_dec_r8() {
    let (mut cpu, mut mmu) = new_test_cpu();

    // DEC B
    cpu.regs.set_b(1);
    exec!(cpu, mmu, 0x05);
    assert_eq!(cpu.regs.b(), 0);

    // DEC C
    cpu.regs.set_c(2);
    exec!(cpu, mmu, 0x0d);
    assert_eq!(cpu.regs.c(), 1);

    // DEC D
    cpu.regs.set_d(3);
    exec!(cpu, mmu, 0x15);
    assert_eq!(cpu.regs.d(), 2);

    // DEC E
    cpu.regs.set_e(4);
    exec!(cpu, mmu, 0x1d);
    assert_eq!(cpu.regs.e(), 3);

    // DEC H
    cpu.regs.set_h(5);
    exec!(cpu, mmu, 0x25);
    assert_eq!(cpu.regs.h(), 4);

    // DEC L
    cpu.regs.set_l(6);
    exec!(cpu, mmu, 0x2d);
    assert_eq!(cpu.regs.l(), 5);

    // DEC (HL)
    cpu.regs.set_hl(0xff90);
    mmu.write8(0xff90, 7);
    exec!(cpu, mmu, 0x35);
    assert_eq!(mmu.read8(0xff90), 6);

    // DEC A
    cpu.regs.set_a(8);
    exec!(cpu, mmu, 0x3d);
    assert_eq!(cpu.regs.a(), 7);
  }

  #[test]
  fn opcode_dec_r8_flags() {
    let (mut cpu, mut mmu) = new_test_cpu();

    // NF is set to true
    cpu.regs.set_a(8);
    exec!(cpu, mmu, 0x3d);
    assert_eq!(cpu.regs.get_flag(NF), true);

    // ZF is set to true if result is 0
    cpu.regs.set_a(0x01);
    exec!(cpu, mmu, 0x3d);
    assert_eq!(cpu.regs.get_flag(ZF), true);

    // ZF is set to false if result is not 0
    cpu.regs.set_a(0x02);
    exec!(cpu, mmu, 0x3d);
    assert_eq!(cpu.regs.get_flag(ZF), false);

    // HF is set to true if overflows from bit 3
    cpu.regs.set_a(0b0000_0000);
    exec!(cpu, mmu, 0x3d);
    assert_eq!(cpu.regs.get_flag(HF), true);

    // HF is set to false if does not overflow from bit 3
    cpu.regs.set_a(0b0000_1000);
    exec!(cpu, mmu, 0x3d);
    assert_eq!(cpu.regs.get_flag(HF), false);
  }

  #[test]
  fn opcode_ld_r8_n8() {
    let (mut cpu, mut mmu) = new_test_cpu();

    // LD B, 1
    exec!(cpu, mmu, 0b00_000_110, arg8 => 1);
    assert_eq!(cpu.regs.read8(B), 1);

    // LD C, 2
    exec!(cpu, mmu, 0b00_001_110, arg8 => 2);
    assert_eq!(cpu.regs.read8(C), 2);

    // LD D, 3
    exec!(cpu, mmu, 0b00_010_110, arg8 => 3);
    assert_eq!(cpu.regs.read8(D), 3);

    // LD E, 4
    exec!(cpu, mmu, 0b00_011_110, arg8 => 4);
    assert_eq!(cpu.regs.read8(E), 4);

    // LD H, 5
    exec!(cpu, mmu, 0b00_100_110, arg8 => 5);
    assert_eq!(cpu.regs.read8(H), 5);

    // LD L, 6
    exec!(cpu, mmu, 0b00_101_110, arg8 => 6);
    assert_eq!(cpu.regs.read8(L), 6);

    // LD (HL), 7
    cpu.regs.set_hl(0xff90);
    exec!(cpu, mmu, 0b00_110_110, arg8 => 7);
    assert_eq!(mmu.read16(0xff90), 7);

    // LD A, 8
    exec!(cpu, mmu, 0b00_111_110, arg8 => 8);
    assert_eq!(cpu.regs.read8(A), 8);
  }

  #[test]
  fn opcode_rdca() {
    let (mut cpu, mut mmu) = new_test_cpu();

    // RLCA
    cpu.regs.set_a(0b0000_0010);
    exec!(cpu, mmu, 0b0000_0111);
    assert_eq!(cpu.regs.a(), 0b0000_0100);

    // RRCA
    cpu.regs.set_a(0b0000_0010);
    exec!(cpu, mmu, 0b0000_1111);
    assert_eq!(cpu.regs.a(), 0b0000_0001);
  }

  #[test]
  fn opcode_rdca_flags() {
    let (mut cpu, mut mmu) = new_test_cpu();

    // ZH, HF and NF flags set to false
    cpu.regs.set_a(0b0000_0010);
    exec!(cpu, mmu, 0b0000_0111);
    assert_eq!(cpu.regs.get_flag(HF), false);
    assert_eq!(cpu.regs.get_flag(NF), false);
    assert_eq!(cpu.regs.get_flag(ZF), false);

    // CF flag set to false if carry not used
    cpu.regs.set_a(0b0000_0010);
    exec!(cpu, mmu, 0b0000_0111);
    assert_eq!(cpu.regs.get_flag(CF), false);

    // CF flag set to false if carry used
    cpu.regs.set_a(0b1000_0000);
    exec!(cpu, mmu, 0b0000_0111);
    assert_eq!(cpu.regs.get_flag(CF), true);
  }

  #[test]
  fn opcode_rda() {
    let (mut cpu, mut mmu) = new_test_cpu();

    // RLA
    cpu.regs.set_a(0b0000_0010);
    cpu.regs.set_flag(CF, false);
    exec!(cpu, mmu, 0b0001_0111);
    assert_eq!(cpu.regs.a(), 0b0000_0100);

    // RLA without carry flag
    cpu.regs.set_a(0b1000_0000);
    cpu.regs.set_flag(CF, false);
    exec!(cpu, mmu, 0b0001_0111);
    assert_eq!(cpu.regs.a(), 0b0000_0000);

    // RLA with carry flag
    cpu.regs.set_a(0b1000_0000);
    cpu.regs.set_flag(CF, true);
    exec!(cpu, mmu, 0b0001_0111);
    assert_eq!(cpu.regs.a(), 0b0000_0001);

    // RRA
    cpu.regs.set_a(0b0000_0010);
    cpu.regs.set_flag(CF, false);
    exec!(cpu, mmu, 0b0001_1111);
    assert_eq!(cpu.regs.a(), 0b0000_0001);

    // RRA without carry flag
    cpu.regs.set_a(0b0000_0001);
    cpu.regs.set_flag(CF, false);
    exec!(cpu, mmu, 0b0001_1111);
    assert_eq!(cpu.regs.a(), 0b0000_0000);

    // RRA with carry flag
    cpu.regs.set_a(0b0000_0001);
    cpu.regs.set_flag(CF, true);
    exec!(cpu, mmu, 0b0001_1111);
    assert_eq!(cpu.regs.a(), 0b1000_0000);
  }

  #[test]
  fn opcode_rda_flags() {
    let (mut cpu, mut mmu) = new_test_cpu();

    // ZH, HF and NF flags set to false
    cpu.regs.set_a(0b0000_0010);
    exec!(cpu, mmu, 0b0001_0111);
    assert_eq!(cpu.regs.get_flag(HF), false);
    assert_eq!(cpu.regs.get_flag(NF), false);
    assert_eq!(cpu.regs.get_flag(ZF), false);

    // CF flag set to false if carry not used
    cpu.regs.set_a(0b0000_0010);
    exec!(cpu, mmu, 0b0001_0111);
    assert_eq!(cpu.regs.get_flag(CF), false);

    // CF flag set to false if carry used
    cpu.regs.set_a(0b1000_0000);
    exec!(cpu, mmu, 0b0001_0111);
    assert_eq!(cpu.regs.get_flag(CF), true);
  }

  #[test]
  fn opcode_jr_n() {
    let (mut cpu, mut mmu) = new_test_cpu();

    // new PC is incremented by N
    exec!(cpu, mmu, 0b0001_1000, arg8 => 0b0000_0011);
    assert_eq!(cpu.regs.pc(), 3);
  }

  #[test]
  fn opcode_jr_f_n() {
    let (mut cpu, mut mmu) = new_test_cpu();

    // JR NZ, N increments by N if NZ
    cpu.regs.set_pc(0);
    cpu.regs.set_flag(ZF, false);
    exec!(cpu, mmu, 0b0010_0000, arg8 => 0b0000_1000);
    assert_eq!(cpu.regs.pc(), 8);

    // JR NZ, N increments by 2 if not NZ
    cpu.regs.set_pc(0);
    cpu.regs.set_flag(ZF, true);
    exec!(cpu, mmu, 0b0010_0000, arg8 => 0b0000_1000);
    assert_eq!(cpu.regs.pc(), 2);

    // JR Z, N increments by N if not Z
    cpu.regs.set_pc(0);
    cpu.regs.set_flag(ZF, true);
    exec!(cpu, mmu, 0b0010_1000, arg8 => 0b0000_1000);
    assert_eq!(cpu.regs.pc(), 8);

    // JR Z, N increments by 2 if Z
    cpu.regs.set_pc(0);
    cpu.regs.set_flag(ZF, false);
    exec!(cpu, mmu, 0b0010_1000, arg8 => 0b0000_1000);
    assert_eq!(cpu.regs.pc(), 2);

    // JR NC, N increments by N if NC
    cpu.regs.set_pc(0);
    cpu.regs.set_flag(CF, false);
    exec!(cpu, mmu, 0b0011_0000, arg8 => 0b0000_1000);
    assert_eq!(cpu.regs.pc(), 8);

    // JR NC, N increments by 2 if not NC
    cpu.regs.set_pc(0);
    cpu.regs.set_flag(CF, true);
    exec!(cpu, mmu, 0b0011_0000, arg8 => 0b0000_1000);
    assert_eq!(cpu.regs.pc(), 2);

    // JR C, N increments by 2 if not C
    cpu.regs.set_pc(0);
    cpu.regs.set_flag(CF, false);
    exec!(cpu, mmu, 0b0011_1000, arg8 => 0b0000_1000);
    assert_eq!(cpu.regs.pc(), 2);

    // JR C, N increments by N if C
    cpu.regs.set_pc(0);
    cpu.regs.set_flag(CF, true);
    exec!(cpu, mmu, 0b0011_1000, arg8 => 0b0000_1000);
    assert_eq!(cpu.regs.pc(), 8);
  }

  #[test]
  fn opcode_ldi_hl_a() {
    let (mut cpu, mut mmu) = new_test_cpu();

    cpu.regs.set_hl(0xff90);
    cpu.regs.set_a(2);
    exec!(cpu, mmu, 0b0010_0010);
    assert_eq!(mmu.read8(0xff90), 2);
    assert_eq!(cpu.regs.hl(), 0xff91);
  }

  #[test]
  fn opcode_ldi_a_hl() {
    let (mut cpu, mut mmu) = new_test_cpu();

    cpu.regs.set_hl(128);
    mmu._load8(128, 2);
    exec!(cpu, mmu, 0b0010_1010);
    assert_eq!(cpu.regs.a(), 2);
    assert_eq!(cpu.regs.hl(), 129);
  }

  #[test]
  fn opcode_ldd_hl_a() {
    let (mut cpu, mut mmu) = new_test_cpu();

    cpu.regs.set_hl(0xff90);
    cpu.regs.set_a(2);
    exec!(cpu, mmu, 0b0011_0010);
    assert_eq!(mmu.read8(0xff90), 2);
    assert_eq!(cpu.regs.hl(), 0xff8f);
  }

  #[test]
  fn opcode_ldd_a_hl() {
    let (mut cpu, mut mmu) = new_test_cpu();

    cpu.regs.set_hl(128);
    mmu._load8(128, 2);
    exec!(cpu, mmu, 0b0011_1010);
    assert_eq!(cpu.regs.a(), 2);
    assert_eq!(cpu.regs.hl(), 127);
  }

  #[test]
  fn opcode_daa() {
    let (mut cpu, mut mmu) = new_test_cpu();

    // adds 0x06 to A if small digit is greater than 9
    cpu.regs.set_flag(NF, false);
    cpu.regs.set_a(0x0A);
    exec!(cpu, mmu, 0b0010_0111);
    assert_eq!(cpu.regs.a(), 0x10);

    // adds 0x60 to A if big digit is greater than 9 and CF is set
    cpu.regs.set_flag(NF, false);
    cpu.regs.set_flag(CF, true);
    cpu.regs.set_a(0xA0);
    exec!(cpu, mmu, 0b0010_0111);
    assert_eq!(cpu.regs.a(), 0x00);

    // subs 0x06 to A if small digit if C and H flags are set
    cpu.regs.set_flag(NF, true);
    cpu.regs.set_flag(CF, false);
    cpu.regs.set_flag(HF, true);
    cpu.regs.set_a(0x07);
    exec!(cpu, mmu, 0b0010_0111);
    assert_eq!(cpu.regs.a(), 0x01);

    // subs 0x60 to A if small digit if C and C flags are set
    cpu.regs.set_flag(NF, true);
    cpu.regs.set_flag(CF, true);
    cpu.regs.set_flag(HF, false);
    cpu.regs.set_a(0x70);
    exec!(cpu, mmu, 0b0010_0111);
    assert_eq!(cpu.regs.a(), 0x10);
  }

  #[test]
  fn opcode_daa_flags() {
    let (mut cpu, mut mmu) = new_test_cpu();

    // HF flag is reset
    cpu.regs.set_flag(NF, false);
    cpu.regs.set_a(0x0A);
    exec!(cpu, mmu, 0b0010_0111);
    assert_eq!(cpu.regs.get_flag(HF), false);

    // ZF flag is set if result is zero
    cpu.regs.set_flag(NF, false);
    cpu.regs.set_flag(CF, true);
    cpu.regs.set_a(0xA0);
    exec!(cpu, mmu, 0b0010_0111, arg8 => 0b0010_0111);
    assert_eq!(cpu.regs.get_flag(ZF), true);

    // ZF flag is reset if result is not zero
    cpu.regs.set_flag(NF, true);
    cpu.regs.set_flag(CF, false);
    cpu.regs.set_flag(HF, true);
    cpu.regs.set_a(0x07);
    exec!(cpu, mmu, 0b0010_0111);
    assert_eq!(cpu.regs.get_flag(ZF), false);

    // CF flag is set if adjustment is 0x60
    cpu.regs.set_flag(NF, false);
    cpu.regs.set_flag(CF, true);
    cpu.regs.set_a(0x07);
    exec!(cpu, mmu, 0b0010_0111);
    assert_eq!(cpu.regs.get_flag(CF), true);

    // CF flag is reset if adjustment is lower than 0x60
    cpu.regs.set_flag(NF, false);
    cpu.regs.set_flag(CF, false);
    cpu.regs.set_a(0x07);
    exec!(cpu, mmu, 0b0010_0111);
    assert_eq!(cpu.regs.get_flag(CF), false);
  }

  #[test]
  fn opcode_cpl() {
    let (mut cpu, mut mmu) = new_test_cpu();

    cpu.regs.set_a(1);
    exec!(cpu, mmu, 0b0010_1111);
    assert_eq!(cpu.regs.a(), 254);
  }

  #[test]
  fn opcode_cpl_flags() {
    let (mut cpu, mut mmu) = new_test_cpu();

    cpu.regs.set_a(1);
    exec!(cpu, mmu, 0b0010_1111);
    assert_eq!(cpu.regs.get_flag(NF), true);
    assert_eq!(cpu.regs.get_flag(HF), true);
  }

  #[test]
  fn opcode_scf() {
    let (mut cpu, mut mmu) = new_test_cpu();

    exec!(cpu, mmu, 0b0011_0111);
    assert_eq!(cpu.regs.get_flag(NF), false);
    assert_eq!(cpu.regs.get_flag(HF), false);
    assert_eq!(cpu.regs.get_flag(CF), true);
  }

  #[test]
  fn opcode_ccf() {
    let (mut cpu, mut mmu) = new_test_cpu();

    exec!(cpu, mmu, 0b0011_1111);
    assert_eq!(cpu.regs.get_flag(NF), false);
    assert_eq!(cpu.regs.get_flag(HF), false);
    assert_eq!(cpu.regs.get_flag(CF), true);

    exec!(cpu, mmu, 0b0011_1111);
    assert_eq!(cpu.regs.get_flag(NF), false);
    assert_eq!(cpu.regs.get_flag(HF), false);
    assert_eq!(cpu.regs.get_flag(CF), false);

    exec!(cpu, mmu, 0b0011_1111);
    assert_eq!(cpu.regs.get_flag(NF), false);
    assert_eq!(cpu.regs.get_flag(HF), false);
    assert_eq!(cpu.regs.get_flag(CF), true);
  }

  #[test]
  fn opcode_ld_b_r8() {
    let (mut cpu, mut mmu) = new_test_cpu();

    cpu.regs.set_b(1);
    exec!(cpu, mmu, 0b0100_0000);
    assert_eq!(cpu.regs.b(), 1);

    cpu.regs.set_c(2);
    exec!(cpu, mmu, 0b0100_0001);
    assert_eq!(cpu.regs.b(), 2);

    cpu.regs.set_d(3);
    exec!(cpu, mmu, 0b0100_0010);
    assert_eq!(cpu.regs.b(), 3);

    cpu.regs.set_e(4);
    exec!(cpu, mmu, 0b0100_0011);
    assert_eq!(cpu.regs.b(), 4);

    cpu.regs.set_h(5);
    exec!(cpu, mmu, 0b0100_0100);
    assert_eq!(cpu.regs.b(), 5);

    cpu.regs.set_l(6);
    exec!(cpu, mmu, 0b0100_0101);
    assert_eq!(cpu.regs.b(), 6);

    mmu.write8(0xff90, 7);
    cpu.regs.set_hl(0xff90);
    exec!(cpu, mmu, 0b0100_0110);
    assert_eq!(cpu.regs.b(), 7);

    cpu.regs.set_a(8);
    exec!(cpu, mmu, 0b0100_0111);
    assert_eq!(cpu.regs.b(), 8);
  }

  #[test]
  fn opcode_ld_c_r8() {
    let (mut cpu, mut mmu) = new_test_cpu();

    cpu.regs.set_b(1);
    exec!(cpu, mmu, 0b0100_1000);
    assert_eq!(cpu.regs.c(), 1);

    cpu.regs.set_c(2);
    exec!(cpu, mmu, 0b0100_1001);
    assert_eq!(cpu.regs.c(), 2);

    cpu.regs.set_d(3);
    exec!(cpu, mmu, 0b0100_1010);
    assert_eq!(cpu.regs.c(), 3);

    cpu.regs.set_e(4);
    exec!(cpu, mmu, 0b0100_1011);
    assert_eq!(cpu.regs.c(), 4);

    cpu.regs.set_h(5);
    exec!(cpu, mmu, 0b0100_1100);
    assert_eq!(cpu.regs.c(), 5);

    cpu.regs.set_l(6);
    exec!(cpu, mmu, 0b0100_1101);
    assert_eq!(cpu.regs.c(), 6);

    mmu.write8(0xff90, 7);
    cpu.regs.set_hl(0xff90);
    exec!(cpu, mmu, 0b0100_1110);
    assert_eq!(cpu.regs.c(), 7);

    cpu.regs.set_a(8);
    exec!(cpu, mmu, 0b0100_1111);
    assert_eq!(cpu.regs.c(), 8);
  }

  #[test]
  fn opcode_ld_d_r8() {
    let (mut cpu, mut mmu) = new_test_cpu();

    cpu.regs.set_b(1);
    exec!(cpu, mmu, 0b0101_0000);
    assert_eq!(cpu.regs.d(), 1);

    cpu.regs.set_c(2);
    exec!(cpu, mmu, 0b0101_0001);
    assert_eq!(cpu.regs.d(), 2);

    cpu.regs.set_d(3);
    exec!(cpu, mmu, 0b0101_0010);
    assert_eq!(cpu.regs.d(), 3);

    cpu.regs.set_e(4);
    exec!(cpu, mmu, 0b0101_0011);
    assert_eq!(cpu.regs.d(), 4);

    cpu.regs.set_h(5);
    exec!(cpu, mmu, 0b0101_0100);
    assert_eq!(cpu.regs.d(), 5);

    cpu.regs.set_l(6);
    exec!(cpu, mmu, 0b0101_0101);
    assert_eq!(cpu.regs.d(), 6);

    mmu.write8(0xff90, 7);
    cpu.regs.set_hl(0xff90);
    exec!(cpu, mmu, 0b0101_0110);
    assert_eq!(cpu.regs.d(), 7);

    cpu.regs.set_a(8);
    exec!(cpu, mmu, 0b0101_0111);
    assert_eq!(cpu.regs.d(), 8);
  }

  #[test]
  fn opcode_ld_e_r8() {
    let (mut cpu, mut mmu) = new_test_cpu();

    cpu.regs.set_b(1);
    exec!(cpu, mmu, 0b0101_1000);
    assert_eq!(cpu.regs.e(), 1);

    cpu.regs.set_c(2);
    exec!(cpu, mmu, 0b0101_1001);
    assert_eq!(cpu.regs.e(), 2);

    cpu.regs.set_d(3);
    exec!(cpu, mmu, 0b0101_1010);
    assert_eq!(cpu.regs.e(), 3);

    cpu.regs.set_e(4);
    exec!(cpu, mmu, 0b0101_1011);
    assert_eq!(cpu.regs.e(), 4);

    cpu.regs.set_h(5);
    exec!(cpu, mmu, 0b0101_1100);
    assert_eq!(cpu.regs.e(), 5);

    cpu.regs.set_l(6);
    exec!(cpu, mmu, 0b0101_1101);
    assert_eq!(cpu.regs.e(), 6);

    mmu.write8(0xff90, 7);
    cpu.regs.set_hl(0xff90);
    exec!(cpu, mmu, 0b0101_1110);
    assert_eq!(cpu.regs.e(), 7);

    cpu.regs.set_a(8);
    exec!(cpu, mmu, 0b0101_1111);
    assert_eq!(cpu.regs.e(), 8);
  }

  #[test]
  fn opcode_ld_h_r8() {
    let (mut cpu, mut mmu) = new_test_cpu();

    cpu.regs.set_b(1);
    exec!(cpu, mmu, 0b0110_0000);
    assert_eq!(cpu.regs.h(), 1);

    cpu.regs.set_c(2);
    exec!(cpu, mmu, 0b0110_0001);
    assert_eq!(cpu.regs.h(), 2);

    cpu.regs.set_d(3);
    exec!(cpu, mmu, 0b0110_0010);
    assert_eq!(cpu.regs.h(), 3);

    cpu.regs.set_e(4);
    exec!(cpu, mmu, 0b0110_0011);
    assert_eq!(cpu.regs.h(), 4);

    cpu.regs.set_h(5);
    exec!(cpu, mmu, 0b0110_0100);
    assert_eq!(cpu.regs.h(), 5);

    cpu.regs.set_l(6);
    exec!(cpu, mmu, 0b0110_0101);
    assert_eq!(cpu.regs.h(), 6);

    mmu.write8(0xff90, 7);
    cpu.regs.set_hl(0xff90);
    exec!(cpu, mmu, 0b0110_0110);
    assert_eq!(cpu.regs.h(), 7);

    cpu.regs.set_a(8);
    exec!(cpu, mmu, 0b0110_0111);
    assert_eq!(cpu.regs.h(), 8);
  }

  #[test]
  fn opcode_ld_l_r8() {
    let (mut cpu, mut mmu) = new_test_cpu();

    cpu.regs.set_b(1);
    exec!(cpu, mmu, 0b0110_1000);
    assert_eq!(cpu.regs.l(), 1);

    cpu.regs.set_c(2);
    exec!(cpu, mmu, 0b0110_1001);
    assert_eq!(cpu.regs.l(), 2);

    cpu.regs.set_d(3);
    exec!(cpu, mmu, 0b0110_1010);
    assert_eq!(cpu.regs.l(), 3);

    cpu.regs.set_e(4);
    exec!(cpu, mmu, 0b0110_1011);
    assert_eq!(cpu.regs.l(), 4);

    cpu.regs.set_h(5);
    exec!(cpu, mmu, 0b0110_1100);
    assert_eq!(cpu.regs.l(), 5);

    cpu.regs.set_l(6);
    exec!(cpu, mmu, 0b0110_1101);
    assert_eq!(cpu.regs.l(), 6);

    mmu.write8(0xff90, 7);
    cpu.regs.set_hl(0xff90);
    exec!(cpu, mmu, 0b0110_1110);
    assert_eq!(cpu.regs.l(), 7);

    cpu.regs.set_a(8);
    exec!(cpu, mmu, 0b0110_1111);
    assert_eq!(cpu.regs.l(), 8);
  }

  #[test]
  fn opcode_ld_hl_r8() {
    let (mut cpu, mut mmu) = new_test_cpu();

    cpu.regs.set_hl(0xff90);

    cpu.regs.set_b(1);
    exec!(cpu, mmu, 0b0111_0000);
    assert_eq!(mmu.read8(0xff90), 1);

    cpu.regs.set_c(2);
    exec!(cpu, mmu, 0b0111_0001);
    assert_eq!(mmu.read8(0xff90), 2);

    cpu.regs.set_d(3);
    exec!(cpu, mmu, 0b0111_0010);
    assert_eq!(mmu.read8(0xff90), 3);

    cpu.regs.set_e(4);
    exec!(cpu, mmu, 0b0111_0011);
    assert_eq!(mmu.read8(0xff90), 4);

    cpu.regs.set_h(0xff);
    exec!(cpu, mmu, 0b0111_0100);
    assert_eq!(mmu.read8(cpu.regs.hl() as usize), 0xff);

    cpu.regs.set_l(0x90);
    exec!(cpu, mmu, 0b0111_0101);
    assert_eq!(mmu.read8(cpu.regs.hl() as usize), 0x90);

    cpu.regs.set_a(7);
    exec!(cpu, mmu, 0b0111_0111);
    assert_eq!(mmu.read8(cpu.regs.hl() as usize), 7);
  }

  #[test]
  fn opcode_ld_a_r8() {
    let (mut cpu, mut mmu) = new_test_cpu();

    cpu.regs.set_b(1);
    exec!(cpu, mmu, 0b0111_1000);
    assert_eq!(cpu.regs.a(), 1);

    cpu.regs.set_c(2);
    exec!(cpu, mmu, 0b0111_1001);
    assert_eq!(cpu.regs.a(), 2);

    cpu.regs.set_d(3);
    exec!(cpu, mmu, 0b0111_1010);
    assert_eq!(cpu.regs.a(), 3);

    cpu.regs.set_e(4);
    exec!(cpu, mmu, 0b0111_1011);
    assert_eq!(cpu.regs.a(), 4);

    cpu.regs.set_h(5);
    exec!(cpu, mmu, 0b0111_1100);
    assert_eq!(cpu.regs.a(), 5);

    cpu.regs.set_l(6);
    exec!(cpu, mmu, 0b0111_1101);
    assert_eq!(cpu.regs.a(), 6);

    mmu.write8(0xff90, 7);
    cpu.regs.set_hl(0xff90);
    exec!(cpu, mmu, 0b0111_1110);
    assert_eq!(cpu.regs.a(), 7);

    cpu.regs.set_a(8);
    exec!(cpu, mmu, 0b0111_1111);
    assert_eq!(cpu.regs.a(), 8);
  }

  #[test]
  fn opcode_add() {
    let (mut cpu, mut mmu) = new_test_cpu();

    // ADD A, B
    cpu.regs.set_a(1);
    cpu.regs.set_b(2);
    exec!(cpu, mmu, 0b1000_0000);
    assert_eq!(cpu.regs.a(), 3);

    // ADD A, C
    cpu.regs.set_a(1);
    cpu.regs.set_c(2);
    exec!(cpu, mmu, 0b1000_0001);
    assert_eq!(cpu.regs.a(), 3);

    // ADD A, D
    cpu.regs.set_a(1);
    cpu.regs.set_d(2);
    exec!(cpu, mmu, 0b1000_0010);
    assert_eq!(cpu.regs.a(), 3);

    // ADD A, E
    cpu.regs.set_a(1);
    cpu.regs.set_e(2);
    exec!(cpu, mmu, 0b1000_0011);
    assert_eq!(cpu.regs.a(), 3);

    // ADD A, H
    cpu.regs.set_a(1);
    cpu.regs.set_h(2);
    exec!(cpu, mmu, 0b1000_0100);
    assert_eq!(cpu.regs.a(), 3);

    // ADD A, L
    cpu.regs.set_a(1);
    cpu.regs.set_l(2);
    exec!(cpu, mmu, 0b1000_0101);
    assert_eq!(cpu.regs.a(), 3);

    // ADD A, (HL)
    cpu.regs.set_a(1);
    cpu.regs.set_hl(0xff90);
    mmu.write8(0xff90, 2);
    exec!(cpu, mmu, 0b1000_0110);
    assert_eq!(cpu.regs.a(), 3);

    // ADD A, A
    cpu.regs.set_a(1);
    println!("{}", cpu.regs.pc());
    exec!(cpu, mmu, 0b1000_0111);
    assert_eq!(cpu.regs.a(), 2);

    // ADD A, N
    cpu.regs.set_a(1);
    exec!(cpu, mmu, 0b1100_0110, arg8 => 2);
    assert_eq!(cpu.regs.a(), 3);
  }

  #[test]
  fn opcode_add_flags() {
    let (mut cpu, mut mmu) = new_test_cpu();

    // ZF flag set if result is zero
    cpu.regs.set_a(0);
    cpu.regs.set_b(0);
    exec!(cpu, mmu, 0b1000_0000);
    assert_eq!(cpu.regs.get_flag(ZF), true);

    // HF flag set if overflow from bit 3
    cpu.regs.set_a(0x0A);
    cpu.regs.set_b(0x0A);
    exec!(cpu, mmu, 0b1000_0000);
    assert_eq!(cpu.regs.get_flag(HF), true);

    // NF flag reset
    cpu.regs.set_a(7);
    cpu.regs.set_b(7);
    exec!(cpu, mmu, 0b1000_0000);
    assert_eq!(cpu.regs.get_flag(NF), false);

    // CF flag reset
    cpu.regs.set_a(0xA0);
    cpu.regs.set_b(0xA0);
    exec!(cpu, mmu, 0b1000_0000);
    assert_eq!(cpu.regs.get_flag(CF), true);
  }

  #[test]
  fn opcode_adc() {
    let (mut cpu, mut mmu) = new_test_cpu();

    // ADC A, B
    cpu.regs.set_a(1);
    cpu.regs.set_b(2);
    exec!(cpu, mmu, 0b1000_1000);
    assert_eq!(cpu.regs.a(), 3);

    // ADC A, C
    cpu.regs.set_a(1);
    cpu.regs.set_c(2);
    exec!(cpu, mmu, 0b1000_1001);
    assert_eq!(cpu.regs.a(), 3);

    // ADC A, D
    cpu.regs.set_a(1);
    cpu.regs.set_d(2);
    exec!(cpu, mmu, 0b1000_1010);
    assert_eq!(cpu.regs.a(), 3);

    // ADC A, E
    cpu.regs.set_a(1);
    cpu.regs.set_e(2);
    exec!(cpu, mmu, 0b1000_1011);
    assert_eq!(cpu.regs.a(), 3);

    // ADC A, H
    cpu.regs.set_a(1);
    cpu.regs.set_h(2);
    exec!(cpu, mmu, 0b1000_1100);
    assert_eq!(cpu.regs.a(), 3);

    // ADC A, L
    cpu.regs.set_a(1);
    cpu.regs.set_l(2);
    exec!(cpu, mmu, 0b1000_1101);
    assert_eq!(cpu.regs.a(), 3);

    // ADC A, (HL)
    cpu.regs.set_a(1);
    cpu.regs.set_hl(0xff90);
    mmu.write8(0xff90, 2);
    exec!(cpu, mmu, 0b1000_1110);
    assert_eq!(cpu.regs.a(), 3);

    // ADC A, A
    cpu.regs.set_a(1);
    exec!(cpu, mmu, 0b1000_1111);
    assert_eq!(cpu.regs.a(), 2);

    // ADD A, N
    cpu.regs.set_a(1);
    exec!(cpu, mmu, 0b1100_1110, arg8 => 2);
    assert_eq!(cpu.regs.a(), 3);
  }

  #[test]
  fn opcode_adc_flags() {
    let (mut cpu, mut mmu) = new_test_cpu();

    // ZF flag set if result is zero
    cpu.regs.set_a(0);
    cpu.regs.set_b(0);
    exec!(cpu, mmu, 0b1000_1000);
    assert_eq!(cpu.regs.get_flag(ZF), true);

    // HF flag set if overflow from bit 3
    cpu.regs.set_a(0x0A);
    cpu.regs.set_b(0x0A);
    exec!(cpu, mmu, 0b1000_1000);
    assert_eq!(cpu.regs.get_flag(HF), true);

    // NF flag reset
    cpu.regs.set_a(7);
    cpu.regs.set_b(7);
    exec!(cpu, mmu, 0b1000_1000);
    assert_eq!(cpu.regs.get_flag(NF), false);

    // CF flag reset
    cpu.regs.set_a(0xA0);
    cpu.regs.set_b(0xA0);
    exec!(cpu, mmu, 0b1000_1000);
    assert_eq!(cpu.regs.get_flag(CF), true);
  }

  #[test]
  fn opcode_sub() {
    let (mut cpu, mut mmu) = new_test_cpu();

    // SUB A, B
    cpu.regs.set_a(5);
    cpu.regs.set_b(2);
    exec!(cpu, mmu, 0b1001_0000);
    assert_eq!(cpu.regs.a(), 3);

    // SUB A, C
    cpu.regs.set_a(5);
    cpu.regs.set_c(2);
    exec!(cpu, mmu, 0b1001_0001);
    assert_eq!(cpu.regs.a(), 3);

    // SUB A, D
    cpu.regs.set_a(5);
    cpu.regs.set_d(2);
    exec!(cpu, mmu, 0b1001_0010);
    assert_eq!(cpu.regs.a(), 3);

    // SUB A, E
    cpu.regs.set_a(5);
    cpu.regs.set_e(2);
    exec!(cpu, mmu, 0b1001_0011);
    assert_eq!(cpu.regs.a(), 3);

    // SUB A, H
    cpu.regs.set_a(5);
    cpu.regs.set_h(2);
    exec!(cpu, mmu, 0b1001_0100);
    assert_eq!(cpu.regs.a(), 3);

    // SUB A, L
    cpu.regs.set_a(5);
    cpu.regs.set_l(2);
    exec!(cpu, mmu, 0b1001_0101);
    assert_eq!(cpu.regs.a(), 3);

    // SUB A, (HL)
    cpu.regs.set_a(5);
    cpu.regs.set_hl(1024);
    mmu._load8(1024, 2);
    exec!(cpu, mmu, 0b1001_0110);
    assert_eq!(cpu.regs.a(), 3);

    // SUB A, A
    cpu.regs.set_a(5);
    exec!(cpu, mmu, 0b1001_0111);
    assert_eq!(cpu.regs.a(), 0);

    // SUB A, N
    cpu.regs.set_a(5);
    exec!(cpu, mmu, 0b1101_0110, arg8 => 2);
    assert_eq!(cpu.regs.a(), 3);
  }

  #[test]
  fn opcode_sub_flags() {
    let (mut cpu, mut mmu) = new_test_cpu();

    // ZF flag set if result is zero
    cpu.regs.set_a(0);
    cpu.regs.set_b(0);
    exec!(cpu, mmu, 0b1001_0000);
    assert_eq!(cpu.regs.get_flag(ZF), true);

    // HF flag set if borrow from bit 4
    cpu.regs.set_a(0x10);
    cpu.regs.set_b(0x01);
    exec!(cpu, mmu, 0b1001_0000);
    assert_eq!(cpu.regs.get_flag(HF), true);

    // NF flag set
    cpu.regs.set_a(7);
    cpu.regs.set_b(7);
    exec!(cpu, mmu, 0b1001_0000);
    assert_eq!(cpu.regs.get_flag(NF), true);

    // CF flag set if r8 > A
    cpu.regs.set_a(0xA0);
    cpu.regs.set_b(0xB0);
    exec!(cpu, mmu, 0b1001_0000);
    assert_eq!(cpu.regs.get_flag(CF), true);
  }

  #[test]
  fn opcode_sbc() {
    let (mut cpu, mut mmu) = new_test_cpu();

    // SBC A, B
    cpu.regs.set_a(5);
    cpu.regs.set_b(2);
    exec!(cpu, mmu, 0b1001_1000);
    assert_eq!(cpu.regs.a(), 3);

    // SBC A, C
    cpu.regs.set_a(5);
    cpu.regs.set_c(2);
    exec!(cpu, mmu, 0b1001_1001);
    assert_eq!(cpu.regs.a(), 3);

    // SBC A, D
    cpu.regs.set_a(5);
    cpu.regs.set_d(2);
    exec!(cpu, mmu, 0b1001_1010);
    assert_eq!(cpu.regs.a(), 3);

    // SBC A, E
    cpu.regs.set_a(5);
    cpu.regs.set_e(2);
    exec!(cpu, mmu, 0b1001_1011);
    assert_eq!(cpu.regs.a(), 3);

    // SBC A, H
    cpu.regs.set_a(5);
    cpu.regs.set_h(2);
    exec!(cpu, mmu, 0b1001_1100);
    assert_eq!(cpu.regs.a(), 3);

    // SBC A, L
    cpu.regs.set_a(5);
    cpu.regs.set_l(2);
    exec!(cpu, mmu, 0b1001_1101);
    assert_eq!(cpu.regs.a(), 3);

    // SBC A, (HL)
    cpu.regs.set_a(5);
    cpu.regs.set_hl(1024);
    mmu._load8(1024, 2);
    exec!(cpu, mmu, 0b1001_1110);
    assert_eq!(cpu.regs.a(), 3);

    // SBC A, A
    cpu.regs.set_a(5);
    exec!(cpu, mmu, 0b1001_1111);
    assert_eq!(cpu.regs.a(), 0);

    // SBC A, N
    cpu.regs.set_a(5);
    exec!(cpu, mmu, 0b1101_1110, arg8 => 2);
    assert_eq!(cpu.regs.a(), 3);
  }

  #[test]
  fn opcode_sbc_flags() {
    let (mut cpu, mut mmu) = new_test_cpu();

    // ZF flag set if result is zero
    cpu.regs.set_a(0);
    cpu.regs.set_b(0);
    exec!(cpu, mmu, 0b1001_1000);
    assert_eq!(cpu.regs.get_flag(ZF), true);

    // HF flag set if borrow from bit 4
    cpu.regs.set_a(0x10);
    cpu.regs.set_b(0x01);
    exec!(cpu, mmu, 0b1001_1000);
    assert_eq!(cpu.regs.get_flag(HF), true);

    // NF flag set
    cpu.regs.set_a(7);
    cpu.regs.set_b(7);
    exec!(cpu, mmu, 0b1001_1000);
    assert_eq!(cpu.regs.get_flag(NF), true);

    // CF flag set if r8 > A
    cpu.regs.set_a(0xA0);
    cpu.regs.set_b(0xB0);
    exec!(cpu, mmu, 0b1001_1000);
    assert_eq!(cpu.regs.get_flag(CF), true);
  }

  #[test]
  fn opcode_and() {
    let (mut cpu, mut mmu) = new_test_cpu();

    // AND A, B
    cpu.regs.set_a(5);
    cpu.regs.set_b(3);
    exec!(cpu, mmu, 0b1010_0000);
    assert_eq!(cpu.regs.a(), 1);

    // AND A, C
    cpu.regs.set_a(5);
    cpu.regs.set_c(3);
    exec!(cpu, mmu, 0b1010_0001);
    assert_eq!(cpu.regs.a(), 1);

    // AND A, D
    cpu.regs.set_a(5);
    cpu.regs.set_d(3);
    exec!(cpu, mmu, 0b1010_0010);
    assert_eq!(cpu.regs.a(), 1);

    // AND A, E
    cpu.regs.set_a(5);
    cpu.regs.set_e(3);
    exec!(cpu, mmu, 0b1010_0011);
    assert_eq!(cpu.regs.a(), 1);

    // AND A, H
    cpu.regs.set_a(5);
    cpu.regs.set_h(3);
    exec!(cpu, mmu, 0b1010_0100);
    assert_eq!(cpu.regs.a(), 1);

    // AND A, L
    cpu.regs.set_a(5);
    cpu.regs.set_l(3);
    exec!(cpu, mmu, 0b1010_0101);
    assert_eq!(cpu.regs.a(), 1);

    // AND A, (HL)
    cpu.regs.set_a(5);
    cpu.regs.set_hl(1024);
    mmu._load8(1024, 3);
    exec!(cpu, mmu, 0b1010_0110);
    assert_eq!(cpu.regs.a(), 1);

    // AND A, A
    cpu.regs.set_a(5);
    exec!(cpu, mmu, 0b1010_0111);
    assert_eq!(cpu.regs.a(), 5);

    // AND A, N
    cpu.regs.set_a(5);
    exec!(cpu, mmu, 0b1110_0110, arg8 => 9);
    assert_eq!(cpu.regs.a(), 1);
  }

  #[test]
  fn opcode_and_flags() {
    let (mut cpu, mut mmu) = new_test_cpu();

    // ZF flag set if result is zero
    cpu.regs.set_a(0);
    cpu.regs.set_b(0);
    exec!(cpu, mmu, 0b1010_0000);
    assert_eq!(cpu.regs.get_flag(ZF), true);

    // HF flag always set
    cpu.regs.set_a(0x10);
    cpu.regs.set_b(0x01);
    exec!(cpu, mmu, 0b1010_0000);
    assert_eq!(cpu.regs.get_flag(HF), true);

    // NF flag reset
    cpu.regs.set_a(7);
    cpu.regs.set_b(7);
    exec!(cpu, mmu, 0b1010_0000);
    assert_eq!(cpu.regs.get_flag(NF), false);

    // CF flag reset
    cpu.regs.set_a(0xA0);
    cpu.regs.set_b(0xB0);
    exec!(cpu, mmu, 0b1010_0000);
    assert_eq!(cpu.regs.get_flag(CF), false);
  }

  #[test]
  fn opcode_xor() {
    let (mut cpu, mut mmu) = new_test_cpu();

    // XOR A, B
    cpu.regs.set_a(5);
    cpu.regs.set_b(3);
    exec!(cpu, mmu, 0b1010_1000);
    assert_eq!(cpu.regs.a(), 6);

    // XOR A, C
    cpu.regs.set_a(5);
    cpu.regs.set_c(3);
    exec!(cpu, mmu, 0b1010_1001);
    assert_eq!(cpu.regs.a(), 6);

    // XOR A, D
    cpu.regs.set_a(5);
    cpu.regs.set_d(3);
    exec!(cpu, mmu, 0b1010_1010);
    assert_eq!(cpu.regs.a(), 6);

    // XOR A, E
    cpu.regs.set_a(5);
    cpu.regs.set_e(3);
    exec!(cpu, mmu, 0b1010_1011);
    assert_eq!(cpu.regs.a(), 6);

    // XOR A, H
    cpu.regs.set_a(5);
    cpu.regs.set_h(3);
    exec!(cpu, mmu, 0b1010_1100);
    assert_eq!(cpu.regs.a(), 6);

    // XOR A, L
    cpu.regs.set_a(5);
    cpu.regs.set_l(3);
    exec!(cpu, mmu, 0b1010_1101);
    assert_eq!(cpu.regs.a(), 6);

    // XOR A, (HL)
    cpu.regs.set_a(5);
    cpu.regs.set_hl(1024);
    mmu._load8(1024, 3);
    exec!(cpu, mmu, 0b1010_1110);
    assert_eq!(cpu.regs.a(), 6);

    // XOR A, A
    cpu.regs.set_a(5);
    exec!(cpu, mmu, 0b1010_1111);
    assert_eq!(cpu.regs.a(), 0);

    // XOR A, N
    cpu.regs.set_a(5);
    exec!(cpu, mmu, 0b1110_1110, arg8 => 3);
    assert_eq!(cpu.regs.a(), 6);
  }

  #[test]
  fn opcode_xor_flags() {
    let (mut cpu, mut mmu) = new_test_cpu();

    // ZF flag set if result is zero
    cpu.regs.set_a(0);
    cpu.regs.set_b(0);
    exec!(cpu, mmu, 0b1010_1000);
    assert_eq!(cpu.regs.get_flag(ZF), true);

    // HF flag always reset
    cpu.regs.set_a(0x10);
    cpu.regs.set_b(0x01);
    exec!(cpu, mmu, 0b1010_1000);
    assert_eq!(cpu.regs.get_flag(HF), false);

    // NF flag reset
    cpu.regs.set_a(7);
    cpu.regs.set_b(7);
    exec!(cpu, mmu, 0b1010_1000);
    assert_eq!(cpu.regs.get_flag(NF), false);

    // CF flag reset
    cpu.regs.set_a(0xA0);
    cpu.regs.set_b(0xB0);
    exec!(cpu, mmu, 0b1010_1000);
    assert_eq!(cpu.regs.get_flag(CF), false);
  }

  #[test]
  fn opcode_or() {
    let (mut cpu, mut mmu) = new_test_cpu();

    // OR A, B
    cpu.regs.set_a(5);
    cpu.regs.set_b(3);
    exec!(cpu, mmu, 0b1011_0000);
    assert_eq!(cpu.regs.a(), 7);

    // OR A, C
    cpu.regs.set_a(5);
    cpu.regs.set_c(3);
    exec!(cpu, mmu, 0b1011_0001);
    assert_eq!(cpu.regs.a(), 7);

    // OR A, D
    cpu.regs.set_a(5);
    cpu.regs.set_d(3);
    exec!(cpu, mmu, 0b1011_0010);
    assert_eq!(cpu.regs.a(), 7);

    // OR A, E
    cpu.regs.set_a(5);
    cpu.regs.set_e(3);
    exec!(cpu, mmu, 0b1011_0011);
    assert_eq!(cpu.regs.a(), 7);

    // OR A, H
    cpu.regs.set_a(5);
    cpu.regs.set_h(3);
    exec!(cpu, mmu, 0b1011_0100);
    assert_eq!(cpu.regs.a(), 7);

    // OR A, L
    cpu.regs.set_a(5);
    cpu.regs.set_l(3);
    exec!(cpu, mmu, 0b1011_0101);
    assert_eq!(cpu.regs.a(), 7);

    // OR A, (HL)
    cpu.regs.set_a(5);
    mmu._load8(1024, 3);
    cpu.regs.set_hl(1024);
    exec!(cpu, mmu, 0b1011_0110);
    assert_eq!(cpu.regs.a(), 7);

    // OR A, A
    cpu.regs.set_a(5);
    exec!(cpu, mmu, 0b1011_0111);
    assert_eq!(cpu.regs.a(), 5);

    // OR A, N
    cpu.regs.set_a(5);
    exec!(cpu, mmu, 0b1111_0110, arg8 => 3);
    assert_eq!(cpu.regs.a(), 7);
  }

  #[test]
  fn opcode_or_flags() {
    let (mut cpu, mut mmu) = new_test_cpu();

    // ZF flag set if result is zero
    cpu.regs.set_a(0);
    cpu.regs.set_b(0);
    exec!(cpu, mmu, 0b1011_0000);
    assert_eq!(cpu.regs.get_flag(ZF), true);

    // HF flag always reset
    cpu.regs.set_a(0x10);
    cpu.regs.set_b(0x01);
    exec!(cpu, mmu, 0b1011_0000);
    assert_eq!(cpu.regs.get_flag(HF), false);

    // NF flag reset
    cpu.regs.set_a(7);
    cpu.regs.set_b(7);
    exec!(cpu, mmu, 0b1011_0000);
    assert_eq!(cpu.regs.get_flag(NF), false);

    // CF flag reset
    cpu.regs.set_a(0xA0);
    cpu.regs.set_b(0xB0);
    exec!(cpu, mmu, 0b1011_0000);
    assert_eq!(cpu.regs.get_flag(CF), false);
  }

  #[test]
  fn opcode_pop() {
    let (mut cpu, mut mmu) = new_test_cpu();

    // POP BC
    cpu.regs.set_sp(1024);
    mmu._load16(1024, 0xAF);
    exec!(cpu, mmu, 0b1100_0001);
    assert_eq!(cpu.regs.bc(), 0xAF);
    assert_eq!(cpu.regs.sp(), 1026);

    // POP DE
    cpu.regs.set_sp(1024);
    mmu._load16(1024, 0xAF);
    exec!(cpu, mmu, 0b1101_0001);
    assert_eq!(cpu.regs.de(), 0xAF);
    assert_eq!(cpu.regs.sp(), 1026);

    // POP HL
    cpu.regs.set_sp(1024);
    mmu._load16(1024, 0xAF);
    exec!(cpu, mmu, 0b1110_0001);
    assert_eq!(cpu.regs.hl(), 0xAF);
    assert_eq!(cpu.regs.sp(), 1026);

    // POP AF
    cpu.regs.set_sp(1024);
    mmu._load16(1024, 0xAF);
    exec!(cpu, mmu, 0b1111_0001);
    assert_eq!(cpu.regs.af(), 0xAF);
    assert_eq!(cpu.regs.sp(), 1026);
  }

  #[test]
  fn opcode_push() {
    let (mut cpu, mut mmu) = new_test_cpu();

    // PUSH BC
    cpu.regs.set_sp(0xff90);
    cpu.regs.set_bc(0xAF);
    exec!(cpu, mmu, 0b1100_0101);
    assert_eq!(cpu.regs.sp(), 0xff8e);
    assert_eq!(mmu.read16(0xff8e), 0xAF);

    // PUSH DE
    cpu.regs.set_sp(0xff90);
    cpu.regs.set_de(0xAF);
    exec!(cpu, mmu, 0b1100_0101);
    assert_eq!(cpu.regs.sp(), 0xff8e);
    assert_eq!(mmu.read16(0xff8e), 0xAF);

    // PUSH HL
    cpu.regs.set_sp(0xff90);
    cpu.regs.set_hl(0xAF);
    exec!(cpu, mmu, 0b1110_0101);
    assert_eq!(cpu.regs.sp(), 0xff8e);
    assert_eq!(mmu.read16(0xff8e), 0xAF);

    // PUSH AF
    cpu.regs.set_sp(0xff90);
    cpu.regs.set_af(0xAF);
    exec!(cpu, mmu, 0b1111_0101);
    assert_eq!(cpu.regs.sp(), 0xff8e);
    assert_eq!(mmu.read16(0xff8e), 0xAF);
  }

  #[test]
  fn opcode_ret_f() {
    let (mut cpu, mut mmu) = new_test_cpu();

    // RET NZ if Z flag is not set
    cpu.regs.set_pc(0);
    cpu.regs.set_sp(0xff90);
    cpu.regs.set_flag(ZF, false);
    cpu.push(666, &mut mmu);
    exec!(cpu, mmu, 0b1100_0000);
    assert_eq!(cpu.regs.sp(), 0xff90);
    assert_eq!(cpu.regs.pc(), 666);

    // RET NZ if Z flag is set
    cpu.regs.set_pc(0);
    cpu.regs.set_sp(0xff90);
    cpu.regs.set_flag(ZF, true);
    cpu.push(666, &mut mmu);
    exec!(cpu, mmu, 0b1100_0000);
    assert_eq!(cpu.regs.pc(), 1);
    assert_eq!(cpu.regs.sp(), 0xff8e);

    // RET Z if Z flag is not set
    cpu.regs.set_pc(0);
    cpu.regs.set_sp(0xff90);
    cpu.regs.set_flag(ZF, false);
    cpu.push(666, &mut mmu);
    exec!(cpu, mmu, 0b1100_1000);
    assert_eq!(cpu.regs.pc(), 1);
    assert_eq!(cpu.regs.sp(), 0xff8e);

    // RET Z if Z flag is set
    cpu.regs.set_pc(0);
    cpu.regs.set_sp(0xff90);
    cpu.regs.set_flag(ZF, true);
    cpu.push(666, &mut mmu);
    exec!(cpu, mmu, 0b1100_1000);
    assert_eq!(cpu.regs.sp(), 0xff90);
    assert_eq!(cpu.regs.pc(), 666);

    // RET NC if C flag is not set
    cpu.regs.set_pc(0);
    cpu.regs.set_sp(0xff90);
    cpu.regs.set_flag(CF, false);
    cpu.push(666, &mut mmu);
    exec!(cpu, mmu, 0b1101_0000);
    assert_eq!(cpu.regs.sp(), 0xff90);
    assert_eq!(cpu.regs.pc(), 666);

    // RET NC if C flag is set
    cpu.regs.set_pc(0);
    cpu.regs.set_sp(0xff90);
    cpu.regs.set_flag(CF, true);
    cpu.push(666, &mut mmu);
    exec!(cpu, mmu, 0b1101_0000);
    assert_eq!(cpu.regs.pc(), 1);
    assert_eq!(cpu.regs.sp(), 0xff8e);

    // RET C if C flag is not set
    cpu.regs.set_pc(0);
    cpu.regs.set_sp(0xff90);
    cpu.regs.set_flag(CF, false);
    cpu.push(666, &mut mmu);
    exec!(cpu, mmu, 0b1101_1000);
    assert_eq!(cpu.regs.pc(), 1);
    assert_eq!(cpu.regs.sp(), 0xff8e);

    // RET C if C flag is set
    cpu.regs.set_pc(0);
    cpu.regs.set_sp(0xff90);
    cpu.regs.set_flag(CF, true);
    cpu.push(666, &mut mmu);
    exec!(cpu, mmu, 0b1101_1000);
    assert_eq!(cpu.regs.sp(), 0xff90);
    assert_eq!(cpu.regs.pc(), 666);
  }

  #[test]
  fn opcode_ret() {
    let (mut cpu, mut mmu) = new_test_cpu();

    cpu.regs.set_sp(0xff90);
    cpu.push(666, &mut mmu);
    exec!(cpu, mmu, 0b1100_1001);
    assert_eq!(cpu.regs.sp(), 0xff90);
    assert_eq!(cpu.regs.pc(), 666);
  }

  #[test]
  fn opcode_reti() {
    let (mut cpu, mut mmu) = new_test_cpu();

    cpu.regs.set_sp(0xff90);
    cpu.push(666, &mut mmu);
    exec!(cpu, mmu, 0b1101_1001);
    assert_eq!(cpu.regs.sp(), 0xff90);
    assert_eq!(cpu.regs.pc(), 666);
    assert_eq!(cpu.interrupts, 1);
  }

  #[test]
  fn opcode_jp_f_n() {
    let (mut cpu, mut mmu) = new_test_cpu();

    // JP NZ, N when ZF is not set
    cpu.regs.set_pc(0);
    cpu.regs.set_flag(ZF, false);
    exec!(cpu, mmu, 0b1100_0010, arg8 => 123);
    assert_eq!(cpu.regs.pc(), 123);

    // JP NZ, N when ZF is set
    cpu.regs.set_pc(0);
    cpu.regs.set_flag(ZF, true);
    exec!(cpu, mmu, 0b1100_0010, arg8 => 123);
    assert_eq!(cpu.regs.pc(), 3);

    // JP Z, N when ZF is not set
    cpu.regs.set_pc(0);
    cpu.regs.set_flag(ZF, false);
    exec!(cpu, mmu, 0b1100_1010, arg8 => 123);
    assert_eq!(cpu.regs.pc(), 3);

    // JP Z, N when ZF is set
    cpu.regs.set_pc(0);
    cpu.regs.set_flag(ZF, true);
    exec!(cpu, mmu, 0b1100_1010, arg8 => 123);
    assert_eq!(cpu.regs.pc(), 123);

    // JP NC, N when CF is not set
    cpu.regs.set_pc(0);
    cpu.regs.set_flag(CF, false);
    exec!(cpu, mmu, 0b1101_0010, arg8 => 123);
    assert_eq!(cpu.regs.pc(), 123);

    // JP NC, N when CF is set
    cpu.regs.set_pc(0);
    cpu.regs.set_flag(CF, true);
    exec!(cpu, mmu, 0b1101_0010, arg8 => 123);
    assert_eq!(cpu.regs.pc(), 3);

    // JP C, N when CF is not set
    cpu.regs.set_pc(0);
    cpu.regs.set_flag(CF, false);
    exec!(cpu, mmu, 0b1101_1010, arg8 => 123);
    assert_eq!(cpu.regs.pc(), 3);

    // JP C, N when CF is set
    cpu.regs.set_pc(0);
    cpu.regs.set_flag(CF, true);
    exec!(cpu, mmu, 0b1101_1010, arg8 => 123);
    assert_eq!(cpu.regs.pc(), 123);
  }

  #[test]
  fn opcode_jp_n() {
    let (mut cpu, mut mmu) = new_test_cpu();

    exec!(cpu, mmu, 0b1100_0011, arg8 => 123);
    assert_eq!(cpu.regs.pc(), 123);
  }

  #[test]
  fn opcode_call_f_n() {
    let (mut cpu, mut mmu) = new_test_cpu();

    // CALL NZ, N when ZF is not set
    cpu.regs.set_pc(0);
    cpu.regs.set_flag(ZF, false);
    cpu.regs.set_sp(0xff90);
    exec!(cpu, mmu, 0b1100_0100, arg16 => 123);
    assert_eq!(cpu.regs.sp(), 0xff8e);
    assert_eq!(mmu.read16(0xff8e), 3);
    assert_eq!(cpu.regs.pc(), 123);

    // CALL NZ, N when ZF is set
    cpu.regs.set_pc(0);
    cpu.regs.set_flag(ZF, true);
    cpu.regs.set_sp(0xff90);
    exec!(cpu, mmu, 0b1100_0100, arg16 => 123);
    assert_eq!(cpu.regs.sp(), 0xff90);
    assert_eq!(cpu.regs.pc(), 3);

    // CALL Z, N when ZF is not set
    cpu.regs.set_pc(0);
    cpu.regs.set_flag(ZF, false);
    cpu.regs.set_sp(0xff90);
    exec!(cpu, mmu, 0b1100_1100, arg16 => 123);
    assert_eq!(cpu.regs.sp(), 0xff90);
    assert_eq!(cpu.regs.pc(), 3);

    // CALL Z, N when ZF is set
    cpu.regs.set_pc(0);
    cpu.regs.set_flag(ZF, true);
    cpu.regs.set_sp(0xff90);
    exec!(cpu, mmu, 0b1100_1100, arg16 => 123);
    assert_eq!(cpu.regs.sp(), 0xff8e);
    assert_eq!(mmu.read16(0xff8e), 3);
    assert_eq!(cpu.regs.pc(), 123);

    // CALL NC, N when CF is not set
    cpu.regs.set_pc(0);
    cpu.regs.set_flag(CF, false);
    cpu.regs.set_sp(0xff90);
    exec!(cpu, mmu, 0b1101_0100, arg16 => 123);
    assert_eq!(cpu.regs.sp(), 0xff8e);
    assert_eq!(mmu.read16(0xff8e), 3);
    assert_eq!(cpu.regs.pc(), 123);

    // CALL NC, N when CF is set
    cpu.regs.set_pc(0);
    cpu.regs.set_flag(CF, true);
    cpu.regs.set_sp(0xff90);
    exec!(cpu, mmu, 0b1101_0100, arg16 => 123);
    assert_eq!(cpu.regs.sp(), 0xff90);
    assert_eq!(cpu.regs.pc(), 3);

    // CALL C, N when CF is not set
    cpu.regs.set_pc(0);
    cpu.regs.set_flag(CF, false);
    cpu.regs.set_sp(0xff90);
    exec!(cpu, mmu, 0b1101_1100, arg16 => 123);
    assert_eq!(cpu.regs.sp(), 0xff90);
    assert_eq!(cpu.regs.pc(), 3);

    // CALL C, N when CF is set
    cpu.regs.set_pc(0);
    cpu.regs.set_flag(CF, true);
    cpu.regs.set_sp(0xff90);
    exec!(cpu, mmu, 0b1101_1100, arg16 => 123);
    assert_eq!(cpu.regs.sp(), 0xff8e);
    assert_eq!(mmu.read16(0xff8e), 3);
    assert_eq!(cpu.regs.pc(), 123);
  }

  #[test]
  fn opcode_call_n() {
    let (mut cpu, mut mmu) = new_test_cpu();

    // CALL C, N when CF is set
    cpu.regs.set_sp(0xff90);
    exec!(cpu, mmu, 0b1100_1101, arg16 => 123);
    assert_eq!(cpu.regs.sp(), 0xff8e);
    assert_eq!(mmu.read16(0xff8e), 3);
    assert_eq!(cpu.regs.pc(), 123);
  }

  #[test]
  fn opcode_add_sp_n() {
    let (mut cpu, mut mmu) = new_test_cpu();

    cpu.regs.set_sp(1);
    exec!(cpu, mmu, 0b1110_1000, arg8 => 3);
    assert_eq!(cpu.regs.sp(), 4);
  }

  #[test]
  fn opcode_ld_hl_sp_n() {
    let (mut cpu, mut mmu) = new_test_cpu();

    cpu.regs.set_sp(1);
    exec!(cpu, mmu, 0b1111_1000, arg8 => 3);
    assert_eq!(cpu.regs.hl(), 4);
  }

  #[test]
  fn opcode_ld_ff00_n_a() {
    let (mut cpu, mut mmu) = new_test_cpu();

    cpu.regs.set_a(1);
    exec!(cpu, mmu, 0b1110_0000, arg8 => 0x80);
    assert_eq!(mmu.read8(0xFF80), 1);
  }

  #[test]
  fn opcode_ld_a_ff00_n() {
    let (mut cpu, mut mmu) = new_test_cpu();

    mmu.write8(0xFF80, 1);
    exec!(cpu, mmu, 0b1111_0000, arg8 => 0x80);
    assert_eq!(cpu.regs.a(), 1);
  }

  #[test]
  fn opcode_ld_c_a() {
    let (mut cpu, mut mmu) = new_test_cpu();

    cpu.regs.set_a(1);
    cpu.regs.set_c(0x80);
    exec!(cpu, mmu, 0b1110_0010);
    assert_eq!(mmu.read8(0xFF80), 1);
  }

  #[test]
  fn opcode_ld_a_c() {
    let (mut cpu, mut mmu) = new_test_cpu();

    cpu.regs.set_c(0x80);
    mmu.write8(0xFF80, 1);
    exec!(cpu, mmu, 0b1111_0010);
    assert_eq!(cpu.regs.a(), 1);
  }

  #[test]
  fn opcode_ld_n_a() {
    let (mut cpu, mut mmu) = new_test_cpu();

    cpu.regs.set_a(1);
    exec!(cpu, mmu, 0b1110_1010, arg16 => 0xff90);
    assert_eq!(mmu.read8(0xff90), 1);
  }

  #[test]
  fn opcode_ld_a_n() {
    let (mut cpu, mut mmu) = new_test_cpu();

    mmu.write16(0xff90, 1);
    exec!(cpu, mmu, 0b1111_1010, arg16 => 0xff90);
    assert_eq!(cpu.regs.a(), 1);
  }

  #[test]
  fn opcode_jp_hl() {
    let (mut cpu, mut mmu) = new_test_cpu();

    cpu.regs.set_hl(123);
    exec!(cpu, mmu, 0b1110_1001);
    assert_eq!(cpu.regs.pc(), 123);
  }

  #[test]
  fn opcode_ld_sp_hl() {
    let (mut cpu, mut mmu) = new_test_cpu();

    cpu.regs.set_hl(123);
    exec!(cpu, mmu, 0b1111_1001);
    assert_eq!(cpu.regs.sp(), 123);
  }

  #[test]
  fn opcode_di() {
    let (mut cpu, mut mmu) = new_test_cpu();

    exec!(cpu, mmu, 0b1111_0011);
    assert_eq!(cpu.interrupts, 0);
  }

  #[test]
  fn opcode_ei() {
    let (mut cpu, mut mmu) = new_test_cpu();

    exec!(cpu, mmu, 0b1111_1011);
    assert_eq!(cpu.interrupts, 1);
  }

  #[test]
  fn opcode_cb_rlc() {
    let (mut cpu, mut mmu) = new_test_cpu();

    // RLC B
    cpu.regs.set_b(0b0000_0010);
    exec_cb!(cpu, mmu, 0b0000_0000);
    assert_eq!(cpu.regs.b(), 0b0000_0100);

    // RLC C
    cpu.regs.set_c(0b0000_0010);
    exec_cb!(cpu, mmu, 0b0000_0001);
    assert_eq!(cpu.regs.c(), 0b0000_0100);

    // RLC D
    cpu.regs.set_d(0b0000_0010);
    exec_cb!(cpu, mmu, 0b0000_0010);
    assert_eq!(cpu.regs.d(), 0b0000_0100);

    // RLC C
    cpu.regs.set_e(0b0000_0010);
    exec_cb!(cpu, mmu, 0b0000_0011);
    assert_eq!(cpu.regs.e(), 0b0000_0100);

    // RLC H
    cpu.regs.set_h(0b0000_0010);
    exec_cb!(cpu, mmu, 0b0000_0100);
    assert_eq!(cpu.regs.h(), 0b0000_0100);

    // RLC L
    cpu.regs.set_l(0b0000_0010);
    exec_cb!(cpu, mmu, 0b0000_0101);
    assert_eq!(cpu.regs.l(), 0b0000_0100);

    // RLC (HL)
    cpu.regs.set_hl(0xff90);
    mmu.write8(0xff90, 0b0000_0010);
    exec_cb!(cpu, mmu, 0b0000_0110);
    assert_eq!(mmu.read8(0xff90), 0b0000_0100);

    // RLC A
    cpu.regs.set_a(0b0000_0010);
    exec_cb!(cpu, mmu, 0b0000_0111);
    assert_eq!(cpu.regs.a(), 0b0000_0100);
  }

  #[test]
  fn opcode_cb_rrc() {
    let (mut cpu, mut mmu) = new_test_cpu();

    // RRC B
    cpu.regs.set_b(0b0000_1010);
    mmu._load8(0, 0xCB);
    exec_cb!(cpu, mmu, 0b0000_1000);
    assert_eq!(cpu.regs.b(), 0b0000_0101);

    // RRC C
    cpu.regs.set_c(0b0000_1010);
    mmu._load8(0, 0xCB);
    exec_cb!(cpu, mmu, 0b0000_1001);
    assert_eq!(cpu.regs.c(), 0b0000_0101);

    // RRC D
    cpu.regs.set_d(0b0000_1010);
    mmu._load8(0, 0xCB);
    exec_cb!(cpu, mmu, 0b0000_1010);
    assert_eq!(cpu.regs.d(), 0b0000_0101);

    // RRC C
    cpu.regs.set_e(0b0000_1010);
    mmu._load8(0, 0xCB);
    exec_cb!(cpu, mmu, 0b0000_1011);
    assert_eq!(cpu.regs.e(), 0b0000_0101);

    // RRC H
    cpu.regs.set_h(0b0000_1010);
    mmu._load8(0, 0xCB);
    exec_cb!(cpu, mmu, 0b0000_1100);
    assert_eq!(cpu.regs.h(), 0b0000_0101);

    // RRC L
    cpu.regs.set_l(0b0000_1010);
    mmu._load8(0, 0xCB);
    exec_cb!(cpu, mmu, 0b0000_1101);
    assert_eq!(cpu.regs.l(), 0b0000_0101);

    // RRC (HL)
    cpu.regs.set_hl(0xff90);
    mmu.write8(0xff90, 0b0000_1010);
    mmu._load8(0, 0xCB);
    exec_cb!(cpu, mmu, 0b0000_1110);
    assert_eq!(mmu.read8(0xff90), 0b0000_0101);

    // RRC A
    cpu.regs.set_a(0b0000_1010);
    mmu._load8(0, 0xCB);
    exec_cb!(cpu, mmu, 0b0000_1111);
    assert_eq!(cpu.regs.a(), 0b0000_0101);
  }

  #[test]
  fn opcode_cb_rl() {
    let (mut cpu, mut mmu) = new_test_cpu();

    // RL B
    cpu.regs.set_b(0b0000_0010);
    mmu._load8(0, 0xCB);
    exec_cb!(cpu, mmu, 0b0001_0000);
    assert_eq!(cpu.regs.b(), 0b0000_0100);

    // RL C
    cpu.regs.set_c(0b0000_0010);
    exec_cb!(cpu, mmu, 0b0001_0001);
    assert_eq!(cpu.regs.c(), 0b0000_0100);

    // // RL D
    cpu.regs.set_d(0b0000_0010);
    exec_cb!(cpu, mmu, 0b0001_0010);
    assert_eq!(cpu.regs.d(), 0b0000_0100);

    // // RL C
    cpu.regs.set_e(0b0000_0010);
    exec_cb!(cpu, mmu, 0b0001_0011);
    assert_eq!(cpu.regs.e(), 0b0000_0100);

    // // RL H
    cpu.regs.set_h(0b0000_0010);
    exec_cb!(cpu, mmu, 0b0001_0100);
    assert_eq!(cpu.regs.h(), 0b0000_0100);

    // RL L
    cpu.regs.set_l(0b0000_0010);
    exec_cb!(cpu, mmu, 0b0001_0101);
    assert_eq!(cpu.regs.l(), 0b0000_0100);

    // RL (HL)
    cpu.regs.set_hl(0xff90);
    mmu.write8(0xff90, 0b0000_0010);
    exec_cb!(cpu, mmu, 0b0001_0110);
    assert_eq!(mmu.read8(0xff90), 0b0000_0100);

    // RL A
    cpu.regs.set_a(0b0000_0010);
    exec_cb!(cpu, mmu, 0b0001_0111);
    assert_eq!(cpu.regs.a(), 0b0000_0100);
  }

  #[test]
  fn opcode_cb_rr() {
    let (mut cpu, mut mmu) = new_test_cpu();

    // RR B
    cpu.regs.set_b(0b0000_1010);
    exec_cb!(cpu, mmu, 0b0001_1000);
    assert_eq!(cpu.regs.b(), 0b0000_0101);

    // RR C
    cpu.regs.set_c(0b0000_1010);
    exec_cb!(cpu, mmu, 0b0001_1001);
    assert_eq!(cpu.regs.c(), 0b0000_0101);

    // RR D
    cpu.regs.set_d(0b0000_1010);
    exec_cb!(cpu, mmu, 0b0001_1010);
    assert_eq!(cpu.regs.d(), 0b0000_0101);

    // RR C
    cpu.regs.set_e(0b0000_1010);
    exec_cb!(cpu, mmu, 0b0001_1011);
    assert_eq!(cpu.regs.e(), 0b0000_0101);

    // RR H
    cpu.regs.set_h(0b0000_1010);
    exec_cb!(cpu, mmu, 0b0001_1100);
    assert_eq!(cpu.regs.h(), 0b0000_0101);

    // RR L
    cpu.regs.set_l(0b0000_1010);
    exec_cb!(cpu, mmu, 0b0001_1101);
    assert_eq!(cpu.regs.l(), 0b0000_0101);

    // RR (HL)
    cpu.regs.set_hl(0xff90);
    mmu.write8(0xff90, 0b0000_1010);
    exec_cb!(cpu, mmu, 0b0001_1110);
    assert_eq!(mmu.read8(0xff90), 0b0000_0101);

    // RR A
    cpu.regs.set_a(0b0000_1010);
    exec_cb!(cpu, mmu, 0b0001_1111);
    assert_eq!(cpu.regs.a(), 0b0000_0101);
  }

  #[test]
  fn opcode_cb_sla_d() {
    let (mut cpu, mut mmu) = new_test_cpu();

    // SLA B
    cpu.regs.set_b(0b0000_0001);
    exec_cb!(cpu, mmu, 0b0010_0000);
    assert_eq!(cpu.regs.b(), 0b0000_0010);

    // SLA C
    cpu.regs.set_c(0b0000_0001);
    exec_cb!(cpu, mmu, 0b0010_0001);
    assert_eq!(cpu.regs.c(), 0b0000_0010);

    // SLA D
    cpu.regs.set_d(0b0000_0001);
    exec_cb!(cpu, mmu, 0b0010_0010);
    assert_eq!(cpu.regs.d(), 0b0000_0010);

    // SLA E
    cpu.regs.set_e(0b0000_0001);
    exec_cb!(cpu, mmu, 0b0010_0011);
    assert_eq!(cpu.regs.e(), 0b0000_0010);

    // SLA H
    cpu.regs.set_h(0b0000_0001);
    exec_cb!(cpu, mmu, 0b0010_0100);
    assert_eq!(cpu.regs.h(), 0b0000_0010);

    // SLA L
    cpu.regs.set_l(0b0000_0001);
    exec_cb!(cpu, mmu, 0b0010_0101);
    assert_eq!(cpu.regs.l(), 0b0000_0010);

    // SLA (HL)
    cpu.regs.set_hl(0xff90);
    mmu.write8(0xff90, 0b0000_0001);
    exec_cb!(cpu, mmu, 0b0010_0110);
    assert_eq!(mmu.read8(0xff90), 0b0000_0010);

    // SLA A
    cpu.regs.set_a(0b0000_0001);
    exec_cb!(cpu, mmu, 0b0010_0111);
    assert_eq!(cpu.regs.a(), 0b0000_0010);
  }

  #[test]
  fn opcode_cb_sla_d_flags() {
    let (mut cpu, mut mmu) = new_test_cpu();

    // Sets ZF flag if result is 0
    cpu.regs.set_b(0b1000_0000);
    exec_cb!(cpu, mmu, 0b0010_0000);
    assert_eq!(cpu.regs.get_flag(ZF), true);

    // Does not set ZF flag if result is 0
    cpu.regs.set_b(0b0100_0000);
    exec_cb!(cpu, mmu, 0b0010_0000);
    assert_eq!(cpu.regs.get_flag(ZF), false);
  }

  #[test]
  fn opcode_cb_sra_d() {
    let (mut cpu, mut mmu) = new_test_cpu();

    // SRA B
    cpu.regs.set_b(0b0000_0010);
    exec_cb!(cpu, mmu, 0b0010_1000);
    assert_eq!(cpu.regs.b(), 0b0000_0001);

    // SRA C
    cpu.regs.set_c(0b0000_0010);
    exec_cb!(cpu, mmu, 0b0010_1001);
    assert_eq!(cpu.regs.c(), 0b0000_0001);

    // SRA D
    cpu.regs.set_d(0b0000_0010);
    exec_cb!(cpu, mmu, 0b0010_1010);
    assert_eq!(cpu.regs.d(), 0b0000_0001);

    // SRA E
    cpu.regs.set_e(0b0000_0010);
    exec_cb!(cpu, mmu, 0b0010_1011);
    assert_eq!(cpu.regs.e(), 0b0000_0001);

    // SRA H
    cpu.regs.set_h(0b0000_0010);
    exec_cb!(cpu, mmu, 0b0010_1100);
    assert_eq!(cpu.regs.h(), 0b0000_0001);

    // SRA L
    cpu.regs.set_l(0b0000_0010);
    exec_cb!(cpu, mmu, 0b0010_1101);
    assert_eq!(cpu.regs.l(), 0b0000_0001);

    // SRA (HL)
    cpu.regs.set_hl(0xff90);
    mmu.write8(0xff90, 0b0000_0010);
    exec_cb!(cpu, mmu, 0b0010_1110);
    assert_eq!(mmu.read8(0xff90), 0b0000_0001);

    // SRA A
    cpu.regs.set_a(0b0000_0010);
    exec_cb!(cpu, mmu, 0b0010_1111);
    assert_eq!(cpu.regs.a(), 0b0000_0001);

    // SRA A with carry
    cpu.regs.set_a(0b1000_0000);
    exec_cb!(cpu, mmu, 0b0010_1111);
    assert_eq!(cpu.regs.a(), 0b1100_0000);
  }

  #[test]
  fn opcode_cb_sra_d_flags() {
    let (mut cpu, mut mmu) = new_test_cpu();

    // Sets ZF flag if result is 0
    cpu.regs.set_b(0b0000_0001);
    exec_cb!(cpu, mmu, 0b0010_1000);
    assert_eq!(cpu.regs.get_flag(ZF), true);

    // Does not set ZF flag if result is 0
    cpu.regs.set_b(0b0000_0010);
    exec_cb!(cpu, mmu, 0b0010_1000);
    assert_eq!(cpu.regs.get_flag(ZF), false);
  }

  #[test]
  fn opcode_cb_swap_d() {
    let (mut cpu, mut mmu) = new_test_cpu();

    // SWAP B
    cpu.regs.set_b(0x12);
    exec_cb!(cpu, mmu, 0b0011_0000);
    assert_eq!(cpu.regs.b(), 0x21);

    // SWAP C
    cpu.regs.set_c(0x12);
    exec_cb!(cpu, mmu, 0b0011_0001);
    assert_eq!(cpu.regs.c(), 0x21);

    // SWAP D
    cpu.regs.set_d(0x12);
    exec_cb!(cpu, mmu, 0b0011_0010);
    assert_eq!(cpu.regs.d(), 0x21);

    // SWAP E
    cpu.regs.set_e(0x12);
    exec_cb!(cpu, mmu, 0b0011_0011);
    assert_eq!(cpu.regs.e(), 0x21);

    // SWAP H
    cpu.regs.set_h(0x12);
    exec_cb!(cpu, mmu, 0b0011_0100);
    assert_eq!(cpu.regs.h(), 0x21);

    // SWAP L
    cpu.regs.set_l(0x12);
    exec_cb!(cpu, mmu, 0b0011_0101);
    assert_eq!(cpu.regs.l(), 0x21);

    // SWAP (HL)
    cpu.regs.set_hl(0xff90);
    mmu.write8(0xff90, 0x12);
    exec_cb!(cpu, mmu, 0b0011_0110);
    assert_eq!(mmu.read8(0xff90), 0x21);

    // SWAP A
    cpu.regs.set_a(0x12);
    exec_cb!(cpu, mmu, 0b0011_0111);
    assert_eq!(cpu.regs.a(), 0x21);
  }

  #[test]
  fn opcode_cb_srl_d() {
    let (mut cpu, mut mmu) = new_test_cpu();

    // SRL B
    cpu.regs.set_b(0b0000_0010);
    exec_cb!(cpu, mmu, 0b0011_1000);
    assert_eq!(cpu.regs.b(), 0b0000_0001);

    // SRL C
    cpu.regs.set_c(0b0000_0010);
    exec_cb!(cpu, mmu, 0b0011_1001);
    assert_eq!(cpu.regs.c(), 0b0000_0001);

    // SRL D
    cpu.regs.set_d(0b0000_0010);
    exec_cb!(cpu, mmu, 0b0011_1010);
    assert_eq!(cpu.regs.d(), 0b0000_0001);

    // SRL E
    cpu.regs.set_e(0b0000_0010);
    exec_cb!(cpu, mmu, 0b0011_1011);
    assert_eq!(cpu.regs.e(), 0b0000_0001);

    // SRL H
    cpu.regs.set_h(0b0000_0010);
    exec_cb!(cpu, mmu, 0b0011_1100);
    assert_eq!(cpu.regs.h(), 0b0000_0001);

    // SRL L
    cpu.regs.set_l(0b0000_0010);
    exec_cb!(cpu, mmu, 0b0011_1101);
    assert_eq!(cpu.regs.l(), 0b0000_0001);

    // SRL (HL)
    cpu.regs.set_hl(0xff90);
    mmu.write8(0xff90, 0b0000_0010);
    exec_cb!(cpu, mmu, 0b0011_1110);
    assert_eq!(mmu.read8(0xff90), 0b0000_0001);

    // SRL A
    cpu.regs.set_a(0b0000_0010);
    exec_cb!(cpu, mmu, 0b0011_1111);
    assert_eq!(cpu.regs.a(), 0b0000_0001);

    // SRL A with carry
    cpu.regs.set_a(0b1000_0000);
    exec_cb!(cpu, mmu, 0b0011_1111);
    assert_eq!(cpu.regs.a(), 0b0100_0000);
  }

  #[test]
  fn opcode_cb_srl_d_flags() {
    let (mut cpu, mut mmu) = new_test_cpu();

    // Sets ZF flag if result is 0
    cpu.regs.set_b(0b0000_0001);
    exec_cb!(cpu, mmu, 0b0011_1000);
    assert_eq!(cpu.regs.get_flag(ZF), true);

    // Does not set ZF flag if result is 0
    cpu.regs.set_b(0b0000_0010);
    exec_cb!(cpu, mmu, 0b0011_1000);
    assert_eq!(cpu.regs.get_flag(ZF), false);
  }

  #[test]
  fn opcode_cb_bit_n_d_flags() {
    let (mut cpu, mut mmu) = new_test_cpu();

    // BIT N, B sets ZF if bit N is zero
    cpu.regs.set_b(0b0000_0000);
    exec_cb!(cpu, mmu, 0b0101_0000);
    assert_eq!(cpu.regs.get_flag(ZF), true);

    // BIT N, C resets ZF if bit N is 1
    cpu.regs.set_c(0b0000_0100);
    exec_cb!(cpu, mmu, 0b0101_0001);
    assert_eq!(cpu.regs.get_flag(ZF), false);

    // BIT N, (HL) sets ZF if bit N is zero
    cpu.regs.set_hl(123);
    mmu._load8(123, 0b0000_0000);
    exec_cb!(cpu, mmu, 0b0111_1110);
    assert_eq!(cpu.regs.get_flag(ZF), true);

    // BIT N, A resets ZF if bit N is 1
    cpu.regs.set_a(0b1000_0000);
    exec_cb!(cpu, mmu, 0b0111_1111);
    assert_eq!(cpu.regs.get_flag(ZF), false);
  }

  #[test]
  fn opcode_cb_res_n_d() {
    let (mut cpu, mut mmu) = new_test_cpu();

    // RES N, B
    cpu.regs.set_b(0xFF);
    exec_cb!(cpu, mmu, 0b1001_0000);
    assert_eq!(cpu.regs.b(), 0b1111_1011);

    // RES N, (HL)
    cpu.regs.set_hl(0xff90);
    mmu.write8(0xff90, 0xFF);
    exec_cb!(cpu, mmu, 0b1001_0110);
    assert_eq!(mmu.read8(0xff90), 0b1111_1011);
  }

  #[test]
  fn opcode_cb_set_n_d() {
    let (mut cpu, mut mmu) = new_test_cpu();

    // SET N, B
    cpu.regs.set_b(0x00);
    exec_cb!(cpu, mmu, 0b1101_0000);
    assert_eq!(cpu.regs.b(), 0b0000_0100);

    // SET N, (HL)
    cpu.regs.set_hl(0xff90);
    mmu.write8(0xff90, 0x00);
    exec_cb!(cpu, mmu, 0b1101_0110);
    assert_eq!(mmu.read8(0xff90), 0b0000_0100);
  }
}
