pub mod opcodes;
pub mod registers;

use super::mmu::MMU;
use opcodes::{AluOp, ExtendedOpcode, JumpCondition, Opcode};
use registers::{Flag, Register16, Register8, Registers};

use Flag::*;
use Register16::*;
use Register8::*;
pub struct CPU {
  regs: Registers,
  interrupts: u32,
  cycles: u32,
  pub last_instr_cycles: u8,
}

type ExecResult = (Option<u16>, u8);

impl CPU {
  #[allow(dead_code)]
  pub fn new() -> CPU {
    CPU {
      regs: Registers::new(),
      interrupts: 0,
      cycles: 0,
      last_instr_cycles: 0,
    }
  }

  // executes the next instruction referenced by PC
  #[allow(dead_code)]
  pub fn exec<M: MMU>(&mut self, mmu: &mut M) {
    let current_pc = self.regs.read16(PC);

    let byte = mmu.read8(current_pc as usize);
    let opcode = opcodes::decode(byte);

    // println!("{:#04x}: {:?}", current_pc, opcode);

    let (jump_to, cycles) = self.exec_opcode(opcode, current_pc, mmu);

    let new_pc = match jump_to {
      Some(new_pc) => {
        // println!("   JUMP: {:#02x}", new_pc);
        new_pc
      }
      None => current_pc + opcodes::op_size(opcode),
    };

    self.regs.set_pc(new_pc);
    self.last_instr_cycles = cycles;
    self.cycles += cycles as u32;
  }

  // executes the given opcode
  #[allow(unused_macros)]
  fn exec_opcode<M: MMU>(&mut self, opcode: Opcode, pc: u16, mmu: &mut M) -> ExecResult {
    use opcodes::{Arg::*, JumpCondition::*, Opcode::*};

    match opcode {
      NOP => (None, 4),

      LD(Addr16, Reg16(reg16)) => {
        mmu.write16(self.read_arg16(mmu) as usize, self.regs.read16(reg16));

        (None, 16)
      }

      LD(Reg16(reg16), Imm16) => {
        self.regs.write16(reg16, self.read_arg16(mmu));

        (None, 12)
      }

      ADD(Reg16(HL), Reg16(reg16)) => {
        self.alu_add_hl(self.regs.read16(reg16));

        (None, 8)
      }

      INC(Reg16(reg16)) => {
        self
          .regs
          .write16(reg16, self.regs.read16(reg16).wrapping_add(1));

        (None, 8)
      }

      DEC(Reg16(reg16)) => {
        self
          .regs
          .write16(reg16, self.regs.read16(reg16).wrapping_sub(1));

        (None, 8)
      }

      INC(Reg8(reg8)) => {
        let v = self.alu_inc(self.regs.read8(reg8));
        self.regs.write8(reg8, v);

        (None, 4)
      }

      INC(PtrReg16(reg16)) => {
        let ptr: usize = self.regs.read16(reg16) as usize;
        let v = self.alu_inc(mmu.read8(ptr));
        mmu.write8(ptr, v);

        (None, 12)
      }

      DEC(Reg8(reg8)) => {
        let v = self.alu_dec(self.regs.read8(reg8));
        self.regs.write8(reg8, v);

        (None, 8)
      }

      DEC(PtrReg16(reg16)) => {
        let ptr: usize = self.regs.read16(reg16) as usize;
        let v = self.alu_dec(mmu.read8(ptr));
        mmu.write8(ptr, v);

        (None, 12)
      }

      LD(Reg8(reg8), Imm8) => {
        self.regs.write8(reg8, self.read_arg8(mmu));

        (None, 8)
      }

      LD(PtrReg16(reg16), Imm8) => {
        let ptr: usize = self.regs.read16(reg16) as usize;
        mmu.write8(ptr, self.read_arg8(mmu));

        (None, 12)
      }

      // RdCA
      RLCA => {
        let v = self.alu_rlc(self.regs.a());
        self.regs.set_a(v);

        (None, 4)
      }

      RRCA => {
        let v = self.alu_rrc(self.regs.a());
        self.regs.set_a(v);

        (None, 4)
      }

      RLA => {
        let v = self.alu_rl(self.regs.a());
        self.regs.set_a(v);

        (None, 4)
      }

      RRA => {
        let v = self.alu_rr(self.regs.a());
        self.regs.set_a(v);

        (None, 4)
      }

      STOP => {
        panic!("not done yet");

        // (None, 4)
      }

      JUMP(condition, Imm8) => {
        if self.check_jump_condition(condition) {
          let arg = self.read_arg8(mmu);
          let displacement: i8 = unsafe { std::mem::transmute::<u8, i8>(arg) };
          let abs_displacement: u16 = i8::abs(displacement) as u16;

          if displacement < 0 {
            (Some(pc - abs_displacement + 2), 12)
          } else {
            (Some(pc + abs_displacement + 2), 12)
          }
        } else {
          (None, 8)
        }
      }

      LDI(PtrReg16(reg16), Reg8(reg8)) => {
        mmu.write8(self.regs.read16(reg16) as usize, self.regs.read8(reg8));
        self.regs.set_hl(self.regs.read16(reg16).wrapping_add(1));

        (None, 8)
      }

      LDI(Reg8(reg8), PtrReg16(reg16)) => {
        self
          .regs
          .write8(reg8, mmu.read8(self.regs.read16(reg16) as usize));
        self.regs.set_hl(self.regs.read16(reg16).wrapping_add(1));

        (None, 8)
      }

      LDD(PtrReg16(reg16), Reg8(reg8)) => {
        mmu.write8(self.regs.read16(reg16) as usize, self.regs.read8(reg8));
        self.regs.set_hl(self.regs.read16(reg16).wrapping_sub(1));

        (None, 8)
      }

      LDD(Reg8(reg8), PtrReg16(reg16)) => {
        self
          .regs
          .write8(reg8, mmu.read8(self.regs.read16(reg16) as usize));
        self.regs.set_hl(self.regs.read16(reg16).wrapping_sub(1));

        (None, 8)
      }

      DAA => {
        self.alu_daa();

        (None, 4)
      }

      CPL => {
        self.regs.set_a(!self.regs.a());
        self.regs.set_flag(NF, true);
        self.regs.set_flag(HF, true);

        (None, 4)
      }

      SCF => {
        self.regs.set_flag(CF, true);
        self.regs.set_flag(NF, false);
        self.regs.set_flag(HF, false);

        (None, 4)
      }

      CCF => {
        self.regs.set_flag(CF, !self.regs.get_flag(CF));
        self.regs.set_flag(NF, false);
        self.regs.set_flag(HF, false);

        (None, 4)
      }

      LD(Reg8(reg8_dest), Reg8(reg8_orig)) => {
        self.regs.write8(reg8_dest, self.regs.read8(reg8_orig));

        (None, 4)
      }

      LD(Reg8(reg8_dest), PtrReg16(reg16)) => {
        self
          .regs
          .write8(reg8_dest, mmu.read8(self.regs.read16(reg16) as usize));

        (None, 8)
      }

      LD(PtrReg16(reg16), Reg8(reg8_orig)) => {
        mmu.write8(self.regs.read16(reg16) as usize, self.regs.read8(reg8_orig));

        (None, 8)
      }

      HALT => {
        panic!("not done yet");

        // (None, 4)
      }

      ALU(op, Reg8(A), from) => {
        let (d, cycles) = match from {
          Reg8(r) => (self.regs.read8(r), 4),
          PtrReg16(r) => (mmu.read8(self.regs.read16(r) as usize), 8),
          Imm8 => (self.read_arg8(mmu), 8),
          _ => unreachable!(),
        };

        self.alu_op(op, d);

        (None, cycles)
      }

      POP(reg16) => {
        let v = self.pop(mmu);
        self.regs.write16(reg16, v);

        (None, 12)
      }

      PUSH(reg16) => {
        self.push(self.regs.read16(reg16), mmu);

        (None, 16)
      }

      RST(n) => {
        self.push(self.regs.pc(), mmu);

        (Some((n as u16) << 3), 16)
      }

      RET(NotZero) => {
        if !self.regs.get_flag(ZF) {
          (Some(self.pop(mmu)), 20)
        } else {
          (None, 8)
        }
      }

      RET(Zero) => {
        if self.regs.get_flag(ZF) {
          (Some(self.pop(mmu)), 20)
        } else {
          (None, 8)
        }
      }

      RET(NotCarry) => {
        if !self.regs.get_flag(CF) {
          (Some(self.pop(mmu)), 20)
        } else {
          (None, 8)
        }
      }

      RET(Carry) => {
        if self.regs.get_flag(CF) {
          (Some(self.pop(mmu)), 20)
        } else {
          (None, 8)
        }
      }

      RET(Always) => (Some(self.pop(mmu)), 16),

      RETI => {
        self.interrupts = 1;

        (Some(self.pop(mmu)), 16)
      }

      JUMP(condition, Addr16) => {
        if self.check_jump_condition(condition) {
          (Some(self.read_arg16(mmu)), 12)
        } else {
          (None, 8)
        }
      }

      CALL(NotZero, Addr16) => {
        if !self.regs.get_flag(ZF) {
          self.push(pc + 3, mmu);
          (Some(self.read_arg16(mmu)), 24)
        } else {
          (None, 12)
        }
      }

      CALL(Zero, Addr16) => {
        if self.regs.get_flag(ZF) {
          self.push(pc + 3, mmu);
          (Some(self.read_arg16(mmu)), 24)
        } else {
          (None, 12)
        }
      }

      CALL(NotCarry, Addr16) => {
        if !self.regs.get_flag(CF) {
          self.push(pc + 3, mmu);
          (Some(self.read_arg16(mmu)), 24)
        } else {
          (None, 12)
        }
      }

      CALL(Carry, Addr16) => {
        if self.regs.get_flag(CF) {
          self.push(pc + 3, mmu);
          (Some(self.read_arg16(mmu)), 24)
        } else {
          (None, 12)
        }
      }

      CALL(Always, Addr16) => {
        self.push(pc + 3, mmu);
        (Some(self.read_arg16(mmu)), 24)
      }

      ADD(Reg16(reg16), Imm8) => {
        let v = self.alu_add16imm(self.regs.read16(reg16), mmu);
        self.regs.write16(reg16, v);

        (None, 16)
      }

      LD(Reg16(reg16), SPPlusImm8) => {
        let v = self.alu_add16imm(self.regs.sp(), mmu);
        self.regs.write16(reg16, v);

        (None, 12)
      }

      LD(HighMemImm8, Reg8(reg8)) => {
        let ptr = (0xFF00 | self.read_arg8(mmu) as u16) as usize;
        mmu.write8(ptr, self.regs.read8(reg8));

        (None, 12)
      }

      LD(Reg8(reg8), HighMemImm8) => {
        let ptr = (0xFF00 | self.read_arg8(mmu) as u16) as usize;
        self.regs.write8(reg8, mmu.read8(ptr));

        (None, 12)
      }

      LD(HighMemReg8(reg8_dest), Reg8(reg8_orig)) => {
        let ptr = (0xFF00 | self.regs.read8(reg8_dest) as u16) as usize;
        mmu.write8(ptr, self.regs.read8(reg8_orig));

        (None, 8)
      }

      LD(Reg8(reg8_dest), HighMemReg8(reg8_orig)) => {
        let ptr = (0xFF00 | self.regs.read8(reg8_orig) as u16) as usize;
        self.regs.write8(reg8_dest, mmu.read8(ptr));

        (None, 8)
      }

      LD(Addr16, Reg8(reg8)) => {
        let ptr = self.read_arg16(mmu) as usize;
        mmu.write8(ptr, self.regs.read8(reg8));

        (None, 16)
      }

      LD(Reg8(reg8), Addr16) => {
        let ptr = self.read_arg16(mmu) as usize;
        self.regs.write8(reg8, mmu.read8(ptr));

        (None, 16)
      }

      JUMP(Always, Reg16(reg16)) => (Some(self.regs.read16(reg16)), 4),

      LD(Reg16(reg16_dest), Reg16(reg16_orig)) => {
        self.regs.write16(reg16_dest, self.regs.read16(reg16_orig));

        (None, 8)
      }

      DI => {
        self.interrupts = 0;

        (None, 4)
      }

      EI => {
        self.interrupts = 1;

        (None, 4)
      }

      CALLBACK => {
        let extended_opcode = opcodes::decode_extended(self.read_arg8(mmu));
        (None, self.exec_cb(extended_opcode, mmu))
      }

      _ => unreachable!("Unexpected opcode: {:?}", opcode),
    }
  }

  fn exec_cb<M: MMU>(&mut self, decoded_opcode: ExtendedOpcode, mmu: &mut M) -> u8 {
    use opcodes::{Arg::*, ExtendedOpcode::*};

    // println!("   {:?}", decoded_opcode);

    match decoded_opcode {
      RLC(Reg8(reg8)) => {
        let v = self.alu_rlc(self.regs.read8(reg8));
        self.regs.write8(reg8, v);

        8
      }

      RLC(PtrReg16(reg16)) => {
        let ptr = self.regs.read16(reg16) as usize;
        let v = self.alu_rlc(mmu.read8(ptr));
        mmu.write8(ptr, v);

        16
      }

      RRC(Reg8(reg8)) => {
        let v = self.alu_rrc(self.regs.read8(reg8));
        self.regs.write8(reg8, v);

        8
      }

      RRC(PtrReg16(reg16)) => {
        let ptr = self.regs.read16(reg16) as usize;
        let v = self.alu_rrc(mmu.read8(ptr));
        mmu.write8(ptr, v);

        16
      }

      RL(Reg8(reg8)) => {
        let v = self.alu_rl(self.regs.read8(reg8));
        self.regs.write8(reg8, v);

        8
      }

      RL(PtrReg16(reg16)) => {
        let ptr = self.regs.read16(reg16) as usize;
        let v = self.alu_rl(mmu.read8(ptr));
        mmu.write8(ptr, v);

        16
      }

      RR(Reg8(reg8)) => {
        let v = self.alu_rr(self.regs.read8(reg8));
        self.regs.write8(reg8, v);

        8
      }

      RR(PtrReg16(reg16)) => {
        let ptr = self.regs.read16(reg16) as usize;
        let v = self.alu_rr(mmu.read8(ptr));
        mmu.write8(ptr, v);

        16
      }

      SLA(Reg8(reg8)) => {
        let v = self.alu_sla(self.regs.read8(reg8));
        self.regs.write8(reg8, v);

        8
      }

      SLA(PtrReg16(reg16)) => {
        let ptr = self.regs.read16(reg16) as usize;
        let v = self.alu_sla(mmu.read8(ptr));
        mmu.write8(ptr, v);

        16
      }

      SRA(Reg8(reg8)) => {
        let v = self.alu_sra(self.regs.read8(reg8));
        self.regs.write8(reg8, v);

        8
      }

      SRA(PtrReg16(reg16)) => {
        let ptr = self.regs.read16(reg16) as usize;
        let v = self.alu_sra(mmu.read8(ptr));
        mmu.write8(ptr, v);

        16
      }

      SWAP(Reg8(reg8)) => {
        let v = self.alu_swap(self.regs.read8(reg8));
        self.regs.write8(reg8, v);

        8
      }

      SWAP(PtrReg16(reg16)) => {
        let ptr = self.regs.read16(reg16) as usize;
        let v = self.alu_swap(mmu.read8(ptr));
        mmu.write8(ptr, v);

        16
      }

      SRL(Reg8(reg8)) => {
        let v = self.alu_srl(self.regs.read8(reg8));
        self.regs.write8(reg8, v);

        8
      }

      SRL(PtrReg16(reg16)) => {
        let ptr = self.regs.read16(reg16) as usize;
        let v = self.alu_srl(mmu.read8(ptr));
        mmu.write8(ptr, v);

        16
      }

      BIT(n, Reg8(reg8)) => {
        let v = self.regs.read8(reg8);

        self.alu_bit(n, v);

        8
      }

      BIT(n, PtrReg16(reg16)) => {
        let v = mmu.read8(self.regs.read16(reg16) as usize);

        self.alu_bit(n, v);

        12
      }

      RES(n, Reg8(reg8)) => {
        let v = self.regs.read8(reg8);

        self.regs.write8(reg8, v & !(1 << n));

        8
      }

      RES(n, PtrReg16(reg16)) => {
        let v = mmu.read8(self.regs.read16(reg16) as usize);

        mmu.write8(self.regs.read16(reg16) as usize, v & !(1 << n));

        16
      }

      SET(n, Reg8(reg8)) => {
        let v = self.regs.read8(reg8);

        self.regs.write8(reg8, v | (1 << n));

        8
      }

      SET(n, PtrReg16(reg16)) => {
        let v = mmu.read8(self.regs.read16(reg16) as usize);

        mmu.write8(self.regs.read16(reg16) as usize, v | (1 << n));

        16
      }

      opcode => unreachable!("Unknown extended opcode #{:?}", opcode),
    }
  }

  fn alu_add_hl(&mut self, d: u16) {
    let hl = self.regs.hl();

    let v = hl.wrapping_add(d);

    self.regs.set_hl(v);

    self.regs.set_flag(NF, false);
    self.regs.set_flag(HF, self.overflow16(hl, d, 11));
    self.regs.set_flag(CF, self.overflow16(hl, d, 15));
  }

  fn alu_op(&mut self, op: AluOp, d: u8) {
    match op {
      AluOp::Add => {
        let a = self.regs.a();
        let v = a.wrapping_add(d);

        self.regs.set_flag(ZF, v == 0);
        self.regs.set_flag(HF, (a & 0x0F) + (d & 0x0F) > 0x0F);
        self.regs.set_flag(NF, false);
        self.regs.set_flag(CF, (a as u16) + (d as u16) > 0xFF);

        self.regs.set_a(v);
      }
      AluOp::Adc => {
        let a = self.regs.a();
        let c = if self.regs.get_flag(CF) { 1 } else { 0 };
        let v = a.wrapping_add(d).wrapping_add(c);

        self.regs.set_flag(ZF, v == 0);
        self.regs.set_flag(HF, (a & 0x0F) + (d & 0x0F) + c > 0x0F);
        self.regs.set_flag(NF, false);
        self
          .regs
          .set_flag(CF, (a as u16) + (d as u16) + (c as u16) > 0xFF);

        self.regs.set_a(v);
      }
      AluOp::Sub => {
        let a = self.regs.a();
        let v = a.wrapping_sub(d);

        self.regs.set_flag(ZF, v == 0);
        self.regs.set_flag(HF, (a & 0x0F) < (d & 0x0F));
        self.regs.set_flag(NF, true);
        self.regs.set_flag(CF, (a as u16) < (d as u16));

        self.regs.set_a(v);
      }
      AluOp::Sbc => {
        let a = self.regs.a();
        let c = if self.regs.get_flag(CF) { 1 } else { 0 };
        let v = a.wrapping_sub(d).wrapping_sub(c);

        self.regs.set_flag(ZF, v == 0);
        self.regs.set_flag(HF, (a & 0x0F) < (d & 0x0F) + c);
        self.regs.set_flag(NF, true);
        self.regs.set_flag(CF, (a as u16) < (d as u16));

        self.regs.set_a(v);
      }
      AluOp::And => {
        let a = self.regs.a();
        let v = a & d;

        self.regs.set_flag(ZF, v == 0);
        self.regs.set_flag(CF, false);
        self.regs.set_flag(HF, true);
        self.regs.set_flag(NF, false);

        self.regs.set_a(v);
      }
      AluOp::Xor => {
        let a = self.regs.a();
        let v = a ^ d;

        self.regs.set_flag(ZF, v == 0);
        self.regs.set_flag(CF, false);
        self.regs.set_flag(HF, false);
        self.regs.set_flag(NF, false);

        self.regs.set_a(v);
      }
      AluOp::Or => {
        let a = self.regs.a();
        let v = a | d;

        self.regs.set_flag(ZF, v == 0);
        self.regs.set_flag(CF, false);
        self.regs.set_flag(HF, false);
        self.regs.set_flag(NF, false);

        self.regs.set_a(v);
      }

      AluOp::Cp => {
        let a = self.regs.a();
        let v = a.wrapping_sub(d);

        self.regs.set_flag(ZF, v == 0);
        self.regs.set_flag(HF, (a & 0x0F) < (d & 0x0F));
        self.regs.set_flag(NF, true);
        self.regs.set_flag(CF, (a as u16) < (d as u16));
      }
    }
  }

  fn check_jump_condition(&self, condition: JumpCondition) -> bool {
    use JumpCondition::*;

    match condition {
      Always => true,
      NotZero => !self.regs.get_flag(ZF),
      Zero => self.regs.get_flag(ZF),
      NotCarry => !self.regs.get_flag(CF),
      Carry => self.regs.get_flag(CF),
    }
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
    if r {
      self.regs.set_flag(ZF, true)
    };
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

  fn alu_add16imm<M: MMU>(&mut self, r: u16, mmu: &M) -> u16 {
    let d = self.read_arg8(mmu) as u16;

    let v = r.wrapping_add(d);

    self.regs.set_flag(ZF, v == 0);
    self.regs.set_flag(HF, (r & 0x000F) + (d & 0x000F) > 0x000F);
    self.regs.set_flag(NF, false);
    self.regs.set_flag(CF, (r & 0x00FF) + (d & 0x00FF) > 0x00FF);

    v
  }

  fn push<M: MMU>(&mut self, value: u16, mmu: &mut M) {
    self.regs.set_sp(self.regs.sp() - 2);
    mmu.write16(self.regs.sp() as usize, value);
  }

  fn pop<M: MMU>(&mut self, mmu: &mut M) -> u16 {
    let v = mmu.read16(self.regs.sp() as usize);
    self.regs.set_sp(self.regs.sp() + 2);

    v
  }

  fn read_arg8<M: MMU>(&self, mmu: &M) -> u8 {
    let pc = self.regs.read16(PC);

    mmu.read8((pc + 1) as usize)
  }

  fn read_arg16<M: MMU>(&self, mmu: &M) -> u16 {
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
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::mmu::test_mmu::TestMMU;
  use opcodes::{ExtendedOpcode::*, JumpCondition::*};

  macro_rules! exec {
    ($cpu:expr, $mmu:expr, $opcode:expr) => {{
      let pc = $cpu.regs.pc();
      $mmu.write8(pc as usize, 0x0);
      $cpu.exec_opcode($opcode, pc, &mut $mmu)
    }};

    ($cpu:expr, $mmu:expr,$opcode:expr, arg8 => $arg8:expr) => {{
      let pc = $cpu.regs.pc();
      $mmu.write8(pc as usize, 0x0);
      $mmu.write8((pc + 1) as usize, $arg8);
      $cpu.exec_opcode($opcode, pc, &mut $mmu)
    }};

    ($cpu:expr,$mmu:expr, $opcode:expr, arg16 => $arg16:expr) => {{
      let pc = $cpu.regs.pc();
      $mmu.write8(pc as usize, 0x0);
      $mmu.write16((pc + 1) as usize, $arg16);
      $cpu.exec_opcode($opcode, pc, &mut $mmu)
    }};
  }

  macro_rules! exec_cb {
    ($cpu:expr, $mmu:expr, $opcode:expr) => {{
      $cpu.exec_cb($opcode, &mut $mmu)
    }};
  }

  fn new_test_cpu() -> (CPU, TestMMU) {
    (CPU::new(), TestMMU::new())
  }

  use opcodes::{AluOp::*, Arg::*, Opcode::*};

  #[test]
  fn new_cpu() {
    let (cpu, _mmu) = new_test_cpu();

    assert_eq!(cpu.regs.read16(Register16::BC), 0);
  }

  #[test]
  fn opcode_nop() {
    let (mut cpu, mut mmu) = new_test_cpu();

    exec!(cpu, mmu, NOP);
  }

  #[test]
  fn opcode_ld_ptr16_sp() {
    let (mut cpu, mut mmu) = new_test_cpu();
    cpu.regs.set_sp(2047);

    exec!(cpu, mmu, LD(Addr16, Reg16(SP)), arg16 => 0xff90);

    assert_eq!(mmu.read16(0xff90u16), 2047);
  }

  #[test]
  fn opcode_add_hl_r16() {
    let (mut cpu, mut mmu) = new_test_cpu();

    cpu.regs.write16(HL, 128);
    cpu.regs.write16(BC, 127);
    exec!(cpu, mmu, ADD(Reg16(HL), Reg16(BC)));
    assert_eq!(cpu.regs.read16(HL), 255);
  }

  #[test]
  fn opcode_add_hl_r16_flags() {
    let (mut cpu, mut mmu) = new_test_cpu();

    // carry from bit 11
    cpu.regs.write16(HL, 0b0000_1000_0000_0000);
    cpu.regs.write16(BC, 0b0000_1000_0000_0000);
    exec!(cpu, mmu, ADD(Reg16(HL), Reg16(BC)));

    assert_eq!(cpu.regs.get_flag(NF), false);
    assert_eq!(cpu.regs.get_flag(HF), true);
    assert_eq!(cpu.regs.get_flag(CF), false);

    // carry from bit 15
    cpu.regs.write16(HL, 0b1000_0000_0000_0000);
    cpu.regs.write16(BC, 0b1000_0000_0000_0000);
    exec!(cpu, mmu, ADD(Reg16(HL), Reg16(BC)));

    assert_eq!(cpu.regs.get_flag(NF), false);
    assert_eq!(cpu.regs.get_flag(HF), false);
    assert_eq!(cpu.regs.get_flag(CF), true);

    // carry from bit 11 and 15
    cpu.regs.write16(HL, 0b1000_1000_0000_0000);
    cpu.regs.write16(BC, 0b1000_1000_0000_0000);
    exec!(cpu, mmu, ADD(Reg16(HL), Reg16(BC)));

    assert_eq!(cpu.regs.get_flag(NF), false);
    assert_eq!(cpu.regs.get_flag(HF), true);
    assert_eq!(cpu.regs.get_flag(CF), true);

    // carry from bit 11 and 15 indirectly
    cpu.regs.write16(HL, 0b1100_0100_0000_0000);
    cpu.regs.write16(BC, 0b0100_1100_0000_0000);
    exec!(cpu, mmu, ADD(Reg16(HL), Reg16(BC)));

    assert_eq!(cpu.regs.get_flag(NF), false);
    assert_eq!(cpu.regs.get_flag(HF), true);
    assert_eq!(cpu.regs.get_flag(CF), true);
  }

  #[test]
  fn opcode_ld_r16_a() {
    let (mut cpu, mut mmu) = new_test_cpu();
    cpu.regs.write8(A, 127);
    cpu.regs.write16(BC, 0xff90);

    exec!(cpu, mmu, LD(PtrReg16(BC), Reg8(A)));

    assert_eq!(mmu.read8(0xff90u16), 127);
  }

  #[test]
  fn opcode_ld_a_r16() {
    let (mut cpu, mut mmu) = new_test_cpu();

    mmu.write8(0xff90u16, 127);
    cpu.regs.write16(BC, 0xff90);
    exec!(cpu, mmu, LD(Reg8(A), PtrReg16(BC)));
    assert_eq!(cpu.regs.read8(A), 127);
  }

  #[test]
  fn opcode_inc_r16() {
    let (mut cpu, mut mmu) = new_test_cpu();
    cpu.regs.write16(BC, 257);
    exec!(cpu, mmu, INC(Reg16(BC)));

    assert_eq!(cpu.regs.read16(BC), 258);
  }

  #[test]
  fn opcode_dec_r16() {
    let (mut cpu, mut mmu) = new_test_cpu();
    cpu.regs.write16(HL, 1023);

    exec!(cpu, mmu, DEC(Reg16(HL)));

    assert_eq!(cpu.regs.read16(HL), 1022);
  }

  #[test]
  fn opcode_inc_r8() {
    let (mut cpu, mut mmu) = new_test_cpu();
    cpu.regs.set_b(1);

    exec!(cpu, mmu, INC(Reg8(B)));

    assert_eq!(cpu.regs.read8(B), 2);
  }

  #[test]
  fn opcode_inc_r8_flags() {
    let (mut cpu, mut mmu) = new_test_cpu();

    // NF is set to false
    cpu.regs.set_a(8);
    exec!(cpu, mmu, INC(Reg8(A)));
    assert_eq!(cpu.regs.get_flag(NF), false);

    // ZF is set to true if result is 0
    cpu.regs.set_a(0xFF);
    exec!(cpu, mmu, INC(Reg8(A)));
    assert_eq!(cpu.regs.get_flag(ZF), true);

    // ZF is set to false if result is not 0
    cpu.regs.set_a(0xFE);
    exec!(cpu, mmu, INC(Reg8(A)));
    assert_eq!(cpu.regs.get_flag(ZF), false);

    // HF is set to true if overflows from bit 3
    cpu.regs.set_a(0b0000_1111);
    exec!(cpu, mmu, INC(Reg8(A)));
    assert_eq!(cpu.regs.get_flag(HF), true);

    // HF is set to false if does not overflow from bit 3
    cpu.regs.set_a(0b0000_0111);
    exec!(cpu, mmu, INC(Reg8(A)));
    assert_eq!(cpu.regs.get_flag(HF), false);
  }

  #[test]
  fn opcode_dec_r8() {
    let (mut cpu, mut mmu) = new_test_cpu();
    cpu.regs.set_b(1);

    exec!(cpu, mmu, DEC(Reg8(B)));

    assert_eq!(cpu.regs.b(), 0);
  }

  #[test]
  fn opcode_dec_ptr_hl() {
    let (mut cpu, mut mmu) = new_test_cpu();
    cpu.regs.set_hl(0xff90);
    mmu.write8(0xff90u16, 7);

    exec!(cpu, mmu, DEC(PtrReg16(HL)));

    assert_eq!(mmu.read8(0xff90u16), 6);
  }

  #[test]
  fn opcode_dec_r8_flags() {
    let (mut cpu, mut mmu) = new_test_cpu();

    // NF is set to true
    cpu.regs.set_a(8);
    exec!(cpu, mmu, DEC(Reg8(A)));
    assert_eq!(cpu.regs.get_flag(NF), true);

    // ZF is set to true if result is 0
    cpu.regs.set_a(0x01);
    exec!(cpu, mmu, DEC(Reg8(A)));
    assert_eq!(cpu.regs.get_flag(ZF), true);

    // ZF is set to false if result is not 0
    cpu.regs.set_a(0x02);
    exec!(cpu, mmu, DEC(Reg8(A)));
    assert_eq!(cpu.regs.get_flag(ZF), false);

    // HF is set to true if overflows from bit 3
    cpu.regs.set_a(0b0000_0000);
    exec!(cpu, mmu, DEC(Reg8(A)));
    assert_eq!(cpu.regs.get_flag(HF), true);

    // HF is set to false if does not overflow from bit 3
    cpu.regs.set_a(0b0000_1000);
    exec!(cpu, mmu, DEC(Reg8(A)));
    assert_eq!(cpu.regs.get_flag(HF), false);
  }

  #[test]
  fn opcode_ld_r8_n8() {
    let (mut cpu, mut mmu) = new_test_cpu();

    exec!(cpu, mmu, LD(Reg8(B), Imm8), arg8 => 1);

    assert_eq!(cpu.regs.read8(B), 1);
  }

  #[test]
  fn opcode_rlca() {
    let (mut cpu, mut mmu) = new_test_cpu();
    cpu.regs.set_a(0b0000_0010);

    exec!(cpu, mmu, RLCA);

    assert_eq!(cpu.regs.a(), 0b0000_0100);
  }

  #[test]
  fn opcode_rlca_flags() {
    let (mut cpu, mut mmu) = new_test_cpu();

    // ZH, HF and NF flags set to false
    cpu.regs.set_a(0b0000_0010);
    exec!(cpu, mmu, RLCA);
    assert_eq!(cpu.regs.get_flag(HF), false);
    assert_eq!(cpu.regs.get_flag(NF), false);
    assert_eq!(cpu.regs.get_flag(ZF), false);

    // CF flag set to false if carry not used
    cpu.regs.set_a(0b0000_0010);
    exec!(cpu, mmu, RLCA);
    assert_eq!(cpu.regs.get_flag(CF), false);

    // CF flag set to false if carry used
    cpu.regs.set_a(0b1000_0000);
    exec!(cpu, mmu, RLCA);
    assert_eq!(cpu.regs.get_flag(CF), true);
  }

  #[test]
  fn opcode_rrca() {
    let (mut cpu, mut mmu) = new_test_cpu();
    cpu.regs.set_a(0b0000_0010);

    exec!(cpu, mmu, RRCA);

    assert_eq!(cpu.regs.a(), 0b0000_0001);
  }

  #[test]
  fn opcode_rrca_flags() {
    let (mut cpu, mut mmu) = new_test_cpu();

    // ZH, HF and NF flags set to false
    cpu.regs.set_a(0b0000_0010);
    exec!(cpu, mmu, RRCA);
    assert_eq!(cpu.regs.get_flag(HF), false);
    assert_eq!(cpu.regs.get_flag(NF), false);
    assert_eq!(cpu.regs.get_flag(ZF), false);

    // CF flag set to false if carry not used
    cpu.regs.set_a(0b0000_0010);
    exec!(cpu, mmu, RRCA);
    assert_eq!(cpu.regs.get_flag(CF), false);

    // CF flag set to false if carry used
    cpu.regs.set_a(0b0000_0001);
    exec!(cpu, mmu, RRCA);
    assert_eq!(cpu.regs.get_flag(CF), true);
  }

  #[test]
  fn opcode_rla() {
    let (mut cpu, mut mmu) = new_test_cpu();

    // RLA
    cpu.regs.set_a(0b0000_0010);
    cpu.regs.set_flag(CF, false);
    exec!(cpu, mmu, RLA);
    assert_eq!(cpu.regs.a(), 0b0000_0100);

    // RLA without carry flag
    cpu.regs.set_a(0b1000_0000);
    cpu.regs.set_flag(CF, false);
    exec!(cpu, mmu, RLA);
    assert_eq!(cpu.regs.a(), 0b0000_0000);

    // RLA with carry flag
    cpu.regs.set_a(0b1000_0000);
    cpu.regs.set_flag(CF, true);
    exec!(cpu, mmu, RLA);
    assert_eq!(cpu.regs.a(), 0b0000_0001);
  }

  #[test]
  fn opcode_rla_flags() {
    let (mut cpu, mut mmu) = new_test_cpu();

    // ZH, HF and NF flags set to false
    cpu.regs.set_a(0b0000_0010);
    exec!(cpu, mmu, RLA);
    assert_eq!(cpu.regs.get_flag(HF), false);
    assert_eq!(cpu.regs.get_flag(NF), false);
    assert_eq!(cpu.regs.get_flag(ZF), false);

    // CF flag set to false if carry not used
    cpu.regs.set_a(0b0000_0010);
    exec!(cpu, mmu, RLA);
    assert_eq!(cpu.regs.get_flag(CF), false);

    // CF flag set to false if carry used
    cpu.regs.set_a(0b1000_0000);
    exec!(cpu, mmu, RLA);
    assert_eq!(cpu.regs.get_flag(CF), true);
  }

  #[test]
  fn opcode_rra() {
    let (mut cpu, mut mmu) = new_test_cpu();

    // RRA
    cpu.regs.set_a(0b0000_0010);
    cpu.regs.set_flag(CF, false);
    exec!(cpu, mmu, RRA);
    assert_eq!(cpu.regs.a(), 0b0000_0001);

    // RRA without carry flag
    cpu.regs.set_a(0b0000_0001);
    cpu.regs.set_flag(CF, false);
    exec!(cpu, mmu, RRA);
    assert_eq!(cpu.regs.a(), 0b0000_0000);

    // RRA with carry flag
    cpu.regs.set_a(0b0000_0001);
    cpu.regs.set_flag(CF, true);
    exec!(cpu, mmu, RRA);
    assert_eq!(cpu.regs.a(), 0b1000_0000);
  }

  #[test]
  fn opcode_rra_flags() {
    let (mut cpu, mut mmu) = new_test_cpu();

    // ZH, HF and NF flags set to false
    cpu.regs.set_a(0b0000_0010);
    exec!(cpu, mmu, RRA);
    assert_eq!(cpu.regs.get_flag(HF), false);
    assert_eq!(cpu.regs.get_flag(NF), false);
    assert_eq!(cpu.regs.get_flag(ZF), false);

    // CF flag set to false if carry not used
    cpu.regs.set_a(0b0000_0010);
    exec!(cpu, mmu, RRA);
    assert_eq!(cpu.regs.get_flag(CF), false);

    // CF flag set to false if carry used
    cpu.regs.set_a(0b0000_0001);
    exec!(cpu, mmu, RRA);
    assert_eq!(cpu.regs.get_flag(CF), true);
  }

  #[test]
  fn opcode_jr_n() {
    let (mut cpu, mut mmu) = new_test_cpu();

    let (jump, _) = exec!(cpu, mmu, JUMP(Always, Imm8), arg8 => 0b0000_0011);

    assert_eq!(jump, Some(5));
  }

  #[test]
  fn opcode_jr_f_n() {
    let (mut cpu, mut mmu) = new_test_cpu();

    // JR NZ, N increments by N if NZ
    cpu.regs.set_pc(0);
    cpu.regs.set_flag(ZF, false);
    let (jump, _) = exec!(cpu, mmu, JUMP(NotZero, Imm8), arg8 => 0b0000_1000);
    assert_eq!(jump, Some(10));

    // JR NZ, N increments by 2 if not NZ
    cpu.regs.set_pc(0);
    cpu.regs.set_flag(ZF, true);
    let (jump, _) = exec!(cpu, mmu, JUMP(NotZero, Imm8), arg8 => 0b0000_1000);
    assert_eq!(jump, None);
  }

  #[test]
  fn opcode_ldi_hl_a() {
    let (mut cpu, mut mmu) = new_test_cpu();
    cpu.regs.set_hl(0xff90);
    cpu.regs.set_a(2);

    exec!(cpu, mmu, LDI(PtrReg16(HL), Reg8(A)));

    assert_eq!(mmu.read8(0xff90u16), 2);
    assert_eq!(cpu.regs.hl(), 0xff91);
  }

  #[test]
  fn opcode_ldi_a_hl() {
    let (mut cpu, mut mmu) = new_test_cpu();
    cpu.regs.set_hl(128);
    mmu.write8(128u16, 2);

    exec!(cpu, mmu, LDI(Reg8(A), PtrReg16(HL)));

    assert_eq!(cpu.regs.a(), 2);
    assert_eq!(cpu.regs.hl(), 129);
  }

  #[test]
  fn opcode_ldd_hl_a() {
    let (mut cpu, mut mmu) = new_test_cpu();
    cpu.regs.set_hl(0xff90);
    cpu.regs.set_a(2);

    exec!(cpu, mmu, LDD(PtrReg16(HL), Reg8(A)));

    assert_eq!(mmu.read8(0xff90u16), 2);
    assert_eq!(cpu.regs.hl(), 0xff8f);
  }

  #[test]
  fn opcode_ldd_a_hl() {
    let (mut cpu, mut mmu) = new_test_cpu();
    cpu.regs.set_hl(128);
    mmu.write8(128u16, 2);

    exec!(cpu, mmu, LDD(Reg8(A), PtrReg16(HL)));

    assert_eq!(cpu.regs.a(), 2);
    assert_eq!(cpu.regs.hl(), 127);
  }

  #[test]
  fn opcode_daa() {
    let (mut cpu, mut mmu) = new_test_cpu();

    // adds 0x06 to A if small digit is greater than 9
    cpu.regs.set_flag(NF, false);
    cpu.regs.set_a(0x0A);
    exec!(cpu, mmu, DAA);
    assert_eq!(cpu.regs.a(), 0x10);

    // adds 0x60 to A if big digit is greater than 9 and CF is set
    cpu.regs.set_flag(NF, false);
    cpu.regs.set_flag(CF, true);
    cpu.regs.set_a(0xA0);
    exec!(cpu, mmu, DAA);
    assert_eq!(cpu.regs.a(), 0x00);

    // subs 0x06 to A if small digit if C and H flags are set
    cpu.regs.set_flag(NF, true);
    cpu.regs.set_flag(CF, false);
    cpu.regs.set_flag(HF, true);
    cpu.regs.set_a(0x07);
    exec!(cpu, mmu, DAA);
    assert_eq!(cpu.regs.a(), 0x01);

    // subs 0x60 to A if small digit if C and C flags are set
    cpu.regs.set_flag(NF, true);
    cpu.regs.set_flag(CF, true);
    cpu.regs.set_flag(HF, false);
    cpu.regs.set_a(0x70);
    exec!(cpu, mmu, DAA);
    assert_eq!(cpu.regs.a(), 0x10);
  }

  #[test]
  fn opcode_daa_flags() {
    let (mut cpu, mut mmu) = new_test_cpu();

    // HF flag is reset
    cpu.regs.set_flag(NF, false);
    cpu.regs.set_a(0x0A);
    exec!(cpu, mmu, DAA);
    assert_eq!(cpu.regs.get_flag(HF), false);

    // ZF flag is set if result is zero
    cpu.regs.set_flag(NF, false);
    cpu.regs.set_flag(CF, true);
    cpu.regs.set_a(0xA0);
    exec!(cpu, mmu, DAA);
    assert_eq!(cpu.regs.get_flag(ZF), true);

    // ZF flag is reset if result is not zero
    cpu.regs.set_flag(NF, true);
    cpu.regs.set_flag(CF, false);
    cpu.regs.set_flag(HF, true);
    cpu.regs.set_a(0x07);
    exec!(cpu, mmu, DAA);
    assert_eq!(cpu.regs.get_flag(ZF), false);

    // CF flag is set if adjustment is 0x60
    cpu.regs.set_flag(NF, false);
    cpu.regs.set_flag(CF, true);
    cpu.regs.set_a(0x07);
    exec!(cpu, mmu, DAA);
    assert_eq!(cpu.regs.get_flag(CF), true);

    // CF flag is reset if adjustment is lower than 0x60
    cpu.regs.set_flag(NF, false);
    cpu.regs.set_flag(CF, false);
    cpu.regs.set_a(0x07);
    exec!(cpu, mmu, DAA);
    assert_eq!(cpu.regs.get_flag(CF), false);
  }

  #[test]
  fn opcode_cpl() {
    let (mut cpu, mut mmu) = new_test_cpu();
    cpu.regs.set_a(1);

    exec!(cpu, mmu, CPL);

    assert_eq!(cpu.regs.a(), 254);
  }

  #[test]
  fn opcode_cpl_flags() {
    let (mut cpu, mut mmu) = new_test_cpu();
    cpu.regs.set_a(1);

    exec!(cpu, mmu, CPL);

    assert_eq!(cpu.regs.get_flag(NF), true);
    assert_eq!(cpu.regs.get_flag(HF), true);
  }

  #[test]
  fn opcode_scf() {
    let (mut cpu, mut mmu) = new_test_cpu();

    exec!(cpu, mmu, SCF);

    assert_eq!(cpu.regs.get_flag(NF), false);
    assert_eq!(cpu.regs.get_flag(HF), false);
    assert_eq!(cpu.regs.get_flag(CF), true);
  }

  #[test]
  fn opcode_ccf() {
    let (mut cpu, mut mmu) = new_test_cpu();

    exec!(cpu, mmu, CCF);
    assert_eq!(cpu.regs.get_flag(NF), false);
    assert_eq!(cpu.regs.get_flag(HF), false);
    assert_eq!(cpu.regs.get_flag(CF), true);

    exec!(cpu, mmu, CCF);
    assert_eq!(cpu.regs.get_flag(NF), false);
    assert_eq!(cpu.regs.get_flag(HF), false);
    assert_eq!(cpu.regs.get_flag(CF), false);

    exec!(cpu, mmu, CCF);
    assert_eq!(cpu.regs.get_flag(NF), false);
    assert_eq!(cpu.regs.get_flag(HF), false);
    assert_eq!(cpu.regs.get_flag(CF), true);
  }

  #[test]
  fn opcode_ld_b_r8() {
    let (mut cpu, mut mmu) = new_test_cpu();
    cpu.regs.set_c(2);

    exec!(cpu, mmu, LD(Reg8(B), Reg8(C)));

    assert_eq!(cpu.regs.b(), 2);
  }

  #[test]
  fn opcode_ld_hl_r8() {
    let (mut cpu, mut mmu) = new_test_cpu();
    cpu.regs.set_hl(0xff90);
    cpu.regs.set_b(1);

    exec!(cpu, mmu, LD(PtrReg16(HL), Reg8(B)));

    assert_eq!(mmu.read8(0xff90u16), 1);
  }

  #[test]
  fn opcode_add_a_r8() {
    let (mut cpu, mut mmu) = new_test_cpu();
    cpu.regs.set_a(1);
    cpu.regs.set_b(2);

    exec!(cpu, mmu, ALU(Add, Reg8(A), Reg8(B)));

    assert_eq!(cpu.regs.a(), 3);
  }

  #[test]
  fn opcode_add_a_hl() {
    let (mut cpu, mut mmu) = new_test_cpu();
    cpu.regs.set_a(1);
    cpu.regs.set_hl(0xff90);
    mmu.write8(0xff90u16, 2);

    exec!(cpu, mmu, ALU(Add, Reg8(A), PtrReg16(HL)));

    assert_eq!(cpu.regs.a(), 3);
  }

  #[test]
  fn opcode_add_a_n() {
    let (mut cpu, mut mmu) = new_test_cpu();
    cpu.regs.set_a(1);

    exec!(cpu, mmu, ALU(Add, Reg8(A), Imm8), arg8 => 2);

    assert_eq!(cpu.regs.a(), 3);
  }

  #[test]
  fn opcode_add_flags() {
    let (mut cpu, mut mmu) = new_test_cpu();

    // ZF flag set if result is zero
    cpu.regs.set_a(0);
    cpu.regs.set_b(0);
    exec!(cpu, mmu, ALU(Add, Reg8(A), Reg8(B)));
    assert_eq!(cpu.regs.get_flag(ZF), true);

    // HF flag set if overflow from bit 3
    cpu.regs.set_a(0x0A);
    cpu.regs.set_b(0x0A);
    exec!(cpu, mmu, ALU(Add, Reg8(A), Reg8(B)));
    assert_eq!(cpu.regs.get_flag(HF), true);

    // NF flag reset
    cpu.regs.set_a(7);
    cpu.regs.set_b(7);
    exec!(cpu, mmu, ALU(Add, Reg8(A), Reg8(B)));
    assert_eq!(cpu.regs.get_flag(NF), false);

    // CF flag reset
    cpu.regs.set_a(0xA0);
    cpu.regs.set_b(0xA0);
    exec!(cpu, mmu, ALU(Add, Reg8(A), Reg8(B)));
    assert_eq!(cpu.regs.get_flag(CF), true);
  }

  #[test]
  fn opcode_adc() {
    let (mut cpu, mut mmu) = new_test_cpu();
    cpu.regs.set_a(1);
    cpu.regs.set_b(2);

    exec!(cpu, mmu, ALU(Adc, Reg8(A), Reg8(B)));

    assert_eq!(cpu.regs.a(), 3);
  }

  #[test]
  fn opcode_adc_flags() {
    let (mut cpu, mut mmu) = new_test_cpu();

    // ZF flag set if result is zero
    cpu.regs.set_a(0);
    cpu.regs.set_b(0);
    exec!(cpu, mmu, ALU(Adc, Reg8(A), Reg8(B)));
    assert_eq!(cpu.regs.get_flag(ZF), true);

    // HF flag set if overflow from bit 3
    cpu.regs.set_a(0x0A);
    cpu.regs.set_b(0x0A);
    exec!(cpu, mmu, ALU(Adc, Reg8(A), Reg8(B)));
    assert_eq!(cpu.regs.get_flag(HF), true);

    // NF flag reset
    cpu.regs.set_a(7);
    cpu.regs.set_b(7);
    exec!(cpu, mmu, ALU(Adc, Reg8(A), Reg8(B)));
    assert_eq!(cpu.regs.get_flag(NF), false);

    // CF flag reset
    cpu.regs.set_a(0xA0);
    cpu.regs.set_b(0xA0);
    exec!(cpu, mmu, ALU(Adc, Reg8(A), Reg8(B)));
    assert_eq!(cpu.regs.get_flag(CF), true);
  }

  #[test]
  fn opcode_sub() {
    let (mut cpu, mut mmu) = new_test_cpu();
    cpu.regs.set_a(5);
    cpu.regs.set_b(2);

    exec!(cpu, mmu, ALU(Sub, Reg8(A), Reg8(B)));

    assert_eq!(cpu.regs.a(), 3);
  }

  #[test]
  fn opcode_sub_flags() {
    let (mut cpu, mut mmu) = new_test_cpu();

    // ZF flag set if result is zero
    cpu.regs.set_a(0);
    cpu.regs.set_b(0);
    exec!(cpu, mmu, ALU(Sub, Reg8(A), Reg8(B)));
    assert_eq!(cpu.regs.get_flag(ZF), true);

    // HF flag set if borrow from bit 4
    cpu.regs.set_a(0x10);
    cpu.regs.set_b(0x01);
    exec!(cpu, mmu, ALU(Sub, Reg8(A), Reg8(B)));
    assert_eq!(cpu.regs.get_flag(HF), true);

    // NF flag set
    cpu.regs.set_a(7);
    cpu.regs.set_b(7);
    exec!(cpu, mmu, ALU(Sub, Reg8(A), Reg8(B)));
    assert_eq!(cpu.regs.get_flag(NF), true);

    // CF flag set if r8 > A
    cpu.regs.set_a(0xA0);
    cpu.regs.set_b(0xB0);
    exec!(cpu, mmu, ALU(Sub, Reg8(A), Reg8(B)));
    assert_eq!(cpu.regs.get_flag(CF), true);
  }

  #[test]
  fn opcode_sbc() {
    let (mut cpu, mut mmu) = new_test_cpu();
    cpu.regs.set_a(5);
    cpu.regs.set_b(2);

    exec!(cpu, mmu, ALU(Sbc, Reg8(A), Reg8(B)));

    assert_eq!(cpu.regs.a(), 3);
  }

  #[test]
  fn opcode_sbc_flags() {
    let (mut cpu, mut mmu) = new_test_cpu();

    // ZF flag set if result is zero
    cpu.regs.set_a(0);
    cpu.regs.set_b(0);
    exec!(cpu, mmu, ALU(Sbc, Reg8(A), Reg8(B)));
    assert_eq!(cpu.regs.get_flag(ZF), true);

    // HF flag set if borrow from bit 4
    cpu.regs.set_a(0x10);
    cpu.regs.set_b(0x01);
    exec!(cpu, mmu, ALU(Sbc, Reg8(A), Reg8(B)));
    assert_eq!(cpu.regs.get_flag(HF), true);

    // NF flag set
    cpu.regs.set_a(7);
    cpu.regs.set_b(7);
    exec!(cpu, mmu, ALU(Sbc, Reg8(A), Reg8(B)));
    assert_eq!(cpu.regs.get_flag(NF), true);

    // CF flag set if r8 > A
    cpu.regs.set_a(0xA0);
    cpu.regs.set_b(0xB0);
    exec!(cpu, mmu, ALU(Sbc, Reg8(A), Reg8(B)));
    assert_eq!(cpu.regs.get_flag(CF), true);
  }

  #[test]
  fn opcode_and() {
    let (mut cpu, mut mmu) = new_test_cpu();
    cpu.regs.set_a(5);
    cpu.regs.set_b(3);

    exec!(cpu, mmu, ALU(And, Reg8(A), Reg8(B)));

    assert_eq!(cpu.regs.a(), 1);
  }

  #[test]
  fn opcode_and_flags() {
    let (mut cpu, mut mmu) = new_test_cpu();

    // ZF flag set if result is zero
    cpu.regs.set_a(0);
    cpu.regs.set_b(0);
    exec!(cpu, mmu, ALU(And, Reg8(A), Reg8(B)));
    assert_eq!(cpu.regs.get_flag(ZF), true);

    // HF flag always set
    cpu.regs.set_a(0x10);
    cpu.regs.set_b(0x01);
    exec!(cpu, mmu, ALU(And, Reg8(A), Reg8(B)));
    assert_eq!(cpu.regs.get_flag(HF), true);

    // NF flag reset
    cpu.regs.set_a(7);
    cpu.regs.set_b(7);
    exec!(cpu, mmu, ALU(And, Reg8(A), Reg8(B)));
    assert_eq!(cpu.regs.get_flag(NF), false);

    // CF flag reset
    cpu.regs.set_a(0xA0);
    cpu.regs.set_b(0xB0);
    exec!(cpu, mmu, ALU(And, Reg8(A), Reg8(B)));
    assert_eq!(cpu.regs.get_flag(CF), false);
  }

  #[test]
  fn opcode_xor() {
    let (mut cpu, mut mmu) = new_test_cpu();
    cpu.regs.set_a(5);
    cpu.regs.set_b(3);

    exec!(cpu, mmu, ALU(Xor, Reg8(A), Reg8(B)));

    assert_eq!(cpu.regs.a(), 6);
  }

  #[test]
  fn opcode_xor_flags() {
    let (mut cpu, mut mmu) = new_test_cpu();

    // ZF flag set if result is zero
    cpu.regs.set_a(0);
    cpu.regs.set_b(0);
    exec!(cpu, mmu, ALU(Xor, Reg8(A), Reg8(B)));
    assert_eq!(cpu.regs.get_flag(ZF), true);

    // HF flag always reset
    cpu.regs.set_a(0x10);
    cpu.regs.set_b(0x01);
    exec!(cpu, mmu, ALU(Xor, Reg8(A), Reg8(B)));
    assert_eq!(cpu.regs.get_flag(HF), false);

    // NF flag reset
    cpu.regs.set_a(7);
    cpu.regs.set_b(7);
    exec!(cpu, mmu, ALU(Xor, Reg8(A), Reg8(B)));
    assert_eq!(cpu.regs.get_flag(NF), false);

    // CF flag reset
    cpu.regs.set_a(0xA0);
    cpu.regs.set_b(0xB0);
    exec!(cpu, mmu, ALU(Xor, Reg8(A), Reg8(B)));
    assert_eq!(cpu.regs.get_flag(CF), false);
  }

  #[test]
  fn opcode_or() {
    let (mut cpu, mut mmu) = new_test_cpu();
    cpu.regs.set_a(5);
    cpu.regs.set_b(3);

    exec!(cpu, mmu, ALU(Or, Reg8(A), Reg8(B)));

    assert_eq!(cpu.regs.a(), 7);
  }

  #[test]
  fn opcode_or_flags() {
    let (mut cpu, mut mmu) = new_test_cpu();

    // ZF flag set if result is zero
    cpu.regs.set_a(0);
    cpu.regs.set_b(0);
    exec!(cpu, mmu, ALU(Or, Reg8(A), Reg8(B)));
    assert_eq!(cpu.regs.get_flag(ZF), true);

    // HF flag always reset
    cpu.regs.set_a(0x10);
    cpu.regs.set_b(0x01);
    exec!(cpu, mmu, ALU(Or, Reg8(A), Reg8(B)));
    assert_eq!(cpu.regs.get_flag(HF), false);

    // NF flag reset
    cpu.regs.set_a(7);
    cpu.regs.set_b(7);
    exec!(cpu, mmu, ALU(Or, Reg8(A), Reg8(B)));
    assert_eq!(cpu.regs.get_flag(NF), false);

    // CF flag reset
    cpu.regs.set_a(0xA0);
    cpu.regs.set_b(0xB0);
    exec!(cpu, mmu, ALU(Or, Reg8(A), Reg8(B)));
    assert_eq!(cpu.regs.get_flag(CF), false);
  }

  #[test]
  fn opcode_cp() {
    let (mut cpu, mut mmu) = new_test_cpu();
    cpu.regs.set_a(5);
    cpu.regs.set_b(2);

    exec!(cpu, mmu, ALU(Cp, Reg8(A), Reg8(B)));

    assert_eq!(cpu.regs.a(), 5);
  }

  #[test]
  fn opcode_cp_flags() {
    let (mut cpu, mut mmu) = new_test_cpu();

    // ZF flag set if result is zero
    cpu.regs.set_a(0);
    cpu.regs.set_b(0);
    exec!(cpu, mmu, ALU(Cp, Reg8(A), Reg8(B)));
    assert_eq!(cpu.regs.get_flag(ZF), true);

    // HF flag set if borrow from bit 4
    cpu.regs.set_a(0x10);
    cpu.regs.set_b(0x01);
    exec!(cpu, mmu, ALU(Cp, Reg8(A), Reg8(B)));
    assert_eq!(cpu.regs.get_flag(HF), true);

    // NF flag set
    cpu.regs.set_a(7);
    cpu.regs.set_b(7);
    exec!(cpu, mmu, ALU(Cp, Reg8(A), Reg8(B)));
    assert_eq!(cpu.regs.get_flag(NF), true);

    // CF flag set if r8 > A
    cpu.regs.set_a(0xA0);
    cpu.regs.set_b(0xB0);
    exec!(cpu, mmu, ALU(Cp, Reg8(A), Reg8(B)));
    assert_eq!(cpu.regs.get_flag(CF), true);
  }

  #[test]
  fn opcode_pop() {
    let (mut cpu, mut mmu) = new_test_cpu();
    cpu.regs.set_sp(1024);
    mmu.write16(1024u16, 0xAF);

    exec!(cpu, mmu, POP(BC));

    assert_eq!(cpu.regs.bc(), 0xAF);
    assert_eq!(cpu.regs.sp(), 1026);
  }

  #[test]
  fn opcode_push() {
    let (mut cpu, mut mmu) = new_test_cpu();
    cpu.regs.set_sp(0xff90);
    cpu.regs.set_bc(0xAF);

    exec!(cpu, mmu, PUSH(BC));

    assert_eq!(cpu.regs.sp(), 0xff8e);
    assert_eq!(mmu.read16(0xff8eu16), 0xAF);
  }

  #[test]
  fn opcode_rst() {
    let (mut cpu, mut mmu) = new_test_cpu();
    cpu.regs.set_sp(0xff90);

    let (jump, _) = exec!(cpu, mmu, RST(7));

    assert_eq!(jump, Some(0x38));
  }

  #[test]
  fn opcode_ret_f() {
    let (mut cpu, mut mmu) = new_test_cpu();

    // RET NZ if Z flag is not set
    cpu.regs.set_pc(0);
    cpu.regs.set_sp(0xff90);
    cpu.regs.set_flag(ZF, false);
    cpu.push(666, &mut mmu);
    let (jump, _) = exec!(cpu, mmu, RET(NotZero));
    assert_eq!(cpu.regs.sp(), 0xff90);
    assert_eq!(jump, Some(666));

    // RET NZ if Z flag is set
    cpu.regs.set_pc(0);
    cpu.regs.set_sp(0xff90);
    cpu.regs.set_flag(ZF, true);
    cpu.push(666, &mut mmu);
    let (jump, _) = exec!(cpu, mmu, RET(NotZero));
    assert_eq!(jump, None);
    assert_eq!(cpu.regs.sp(), 0xff8e);
  }

  #[test]
  fn opcode_ret() {
    let (mut cpu, mut mmu) = new_test_cpu();

    cpu.regs.set_sp(0xff90);
    cpu.push(666, &mut mmu);

    let (jump, _) = exec!(cpu, mmu, RET(Always));

    assert_eq!(cpu.regs.sp(), 0xff90);
    assert_eq!(jump, Some(666));
  }

  #[test]
  fn opcode_reti() {
    let (mut cpu, mut mmu) = new_test_cpu();

    cpu.regs.set_sp(0xff90);
    cpu.push(666, &mut mmu);
    let (jump, _) = exec!(cpu, mmu, RETI);
    assert_eq!(cpu.regs.sp(), 0xff90);
    assert_eq!(jump, Some(666));
    assert_eq!(cpu.interrupts, 1);
  }

  #[test]
  fn opcode_jp_f_n() {
    let (mut cpu, mut mmu) = new_test_cpu();

    // JP NZ, N when ZF is not set
    cpu.regs.set_pc(0);
    cpu.regs.set_flag(ZF, false);
    let (jump, _) = exec!(cpu, mmu, JUMP(NotZero, Addr16), arg16 => 123);
    assert_eq!(jump, Some(123));

    // JP NZ, N when ZF is set
    cpu.regs.set_pc(0);
    cpu.regs.set_flag(ZF, true);
    let (jump, _) = exec!(cpu, mmu, JUMP(NotZero, Addr16), arg16 => 123);
    assert_eq!(jump, None);
  }

  #[test]
  fn opcode_jp_n() {
    let (mut cpu, mut mmu) = new_test_cpu();

    let (jump, _) = exec!(cpu, mmu, JUMP(Always, Imm8), arg16 => 123);

    assert_eq!(jump, Some(125));
  }

  #[test]
  fn opcode_call_f_n() {
    let (mut cpu, mut mmu) = new_test_cpu();

    // CALL NZ, N when ZF is not set
    cpu.regs.set_pc(0);
    cpu.regs.set_flag(ZF, false);
    cpu.regs.set_sp(0xff90);
    let (jump, _) = exec!(cpu, mmu, CALL(NotZero, Addr16), arg16 => 123);
    assert_eq!(cpu.regs.sp(), 0xff8e);
    assert_eq!(mmu.read16(0xff8eu16), 3);
    assert_eq!(jump, Some(123));

    // CALL NZ, N when ZF is set
    cpu.regs.set_pc(0);
    cpu.regs.set_flag(ZF, true);
    cpu.regs.set_sp(0xff90);
    let (jump, _) = exec!(cpu, mmu, CALL(NotZero, Addr16), arg16 => 123);
    assert_eq!(cpu.regs.sp(), 0xff90);
    assert_eq!(jump, None);
  }

  #[test]
  fn opcode_call_n() {
    let (mut cpu, mut mmu) = new_test_cpu();
    cpu.regs.set_sp(0xff90);

    let (jump, _) = exec!(cpu, mmu, CALL(Always, Addr16), arg16 => 123);

    assert_eq!(cpu.regs.sp(), 0xff8e);
    assert_eq!(mmu.read16(0xff8eu16), 3);
    assert_eq!(jump, Some(123));
  }

  #[test]
  fn opcode_add_sp_n() {
    let (mut cpu, mut mmu) = new_test_cpu();
    cpu.regs.set_sp(1);

    exec!(cpu, mmu, ADD(Reg16(SP), Imm8), arg8 => 3);

    assert_eq!(cpu.regs.sp(), 4);
  }

  #[test]
  fn opcode_ld_hl_sp_n() {
    let (mut cpu, mut mmu) = new_test_cpu();
    cpu.regs.set_sp(1);

    exec!(cpu, mmu, LD(Reg16(HL), SPPlusImm8), arg8 => 3);

    assert_eq!(cpu.regs.hl(), 4);
  }

  #[test]
  fn opcode_ld_ff00_n_a() {
    let (mut cpu, mut mmu) = new_test_cpu();
    cpu.regs.set_a(1);

    exec!(cpu, mmu, LD(HighMemImm8, Reg8(A)), arg8 => 0x80);

    assert_eq!(mmu.read8(0xFF80u16), 1);
  }

  #[test]
  fn opcode_ld_a_ff00_n() {
    let (mut cpu, mut mmu) = new_test_cpu();
    mmu.write8(0xFF80u16, 1);

    exec!(cpu, mmu, LD(Reg8(A), HighMemImm8), arg8 => 0x80);

    assert_eq!(cpu.regs.a(), 1);
  }

  #[test]
  fn opcode_ld_c_a() {
    let (mut cpu, mut mmu) = new_test_cpu();
    cpu.regs.set_a(1);
    cpu.regs.set_c(0x80);

    exec!(cpu, mmu, LD(HighMemReg8(C), Reg8(A)));

    assert_eq!(mmu.read8(0xFF80u16), 1);
  }

  #[test]
  fn opcode_ld_a_c() {
    let (mut cpu, mut mmu) = new_test_cpu();
    cpu.regs.set_c(0x80);
    mmu.write8(0xFF80u16, 1);

    exec!(cpu, mmu, LD(Reg8(A), HighMemReg8(C)));

    assert_eq!(cpu.regs.a(), 1);
  }

  #[test]
  fn opcode_ld_n_a() {
    let (mut cpu, mut mmu) = new_test_cpu();
    cpu.regs.set_a(1);

    exec!(cpu, mmu, LD(Addr16, Reg8(A)), arg16 => 0xff90);

    assert_eq!(mmu.read8(0xff90u16), 1);
  }

  #[test]
  fn opcode_ld_a_n() {
    let (mut cpu, mut mmu) = new_test_cpu();
    mmu.write16(0xff90u16, 1);

    exec!(cpu, mmu, LD(Reg8(A), Addr16), arg16 => 0xff90);

    assert_eq!(cpu.regs.a(), 1);
  }

  #[test]
  fn opcode_jp_hl() {
    let (mut cpu, mut mmu) = new_test_cpu();
    cpu.regs.set_hl(123);

    let (jump, _) = exec!(cpu, mmu, JUMP(Always, Reg16(HL)));

    assert_eq!(jump, Some(123));
  }

  #[test]
  fn opcode_ld_sp_hl() {
    let (mut cpu, mut mmu) = new_test_cpu();
    cpu.regs.set_hl(123);

    exec!(cpu, mmu, LD(Reg16(SP), Reg16(HL)));

    assert_eq!(cpu.regs.sp(), 123);
  }

  #[test]
  fn opcode_di() {
    let (mut cpu, mut mmu) = new_test_cpu();

    exec!(cpu, mmu, DI);

    assert_eq!(cpu.interrupts, 0);
  }

  #[test]
  fn opcode_ei() {
    let (mut cpu, mut mmu) = new_test_cpu();

    exec!(cpu, mmu, EI);

    assert_eq!(cpu.interrupts, 1);
  }

  #[test]
  fn opcode_cb_rlc() {
    let (mut cpu, mut mmu) = new_test_cpu();
    cpu.regs.set_b(0b0000_0010);

    exec_cb!(cpu, mmu, RLC(Reg8(B)));

    assert_eq!(cpu.regs.b(), 0b0000_0100);
  }

  #[test]
  fn opcode_cb_rrc() {
    let (mut cpu, mut mmu) = new_test_cpu();
    cpu.regs.set_b(0b0000_1010);
    mmu.write8(0u16, 0xCB);

    exec_cb!(cpu, mmu, RRC(Reg8(B)));

    assert_eq!(cpu.regs.b(), 0b0000_0101);
  }

  #[test]
  fn opcode_cb_rl() {
    let (mut cpu, mut mmu) = new_test_cpu();
    cpu.regs.set_b(0b0000_0010);
    mmu.write8(0u16, 0xCB);

    exec_cb!(cpu, mmu, RL(Reg8(B)));

    assert_eq!(cpu.regs.b(), 0b0000_0100);
  }

  #[test]
  fn opcode_cb_rr() {
    let (mut cpu, mut mmu) = new_test_cpu();
    cpu.regs.set_b(0b0000_1010);

    exec_cb!(cpu, mmu, RR(Reg8(B)));

    assert_eq!(cpu.regs.b(), 0b0000_0101);
  }

  #[test]
  fn opcode_cb_sla_d() {
    let (mut cpu, mut mmu) = new_test_cpu();
    cpu.regs.set_b(0b0000_0001);

    exec_cb!(cpu, mmu, SLA(Reg8(B)));

    assert_eq!(cpu.regs.b(), 0b0000_0010);
  }

  #[test]
  fn opcode_cb_sla_d_flags() {
    let (mut cpu, mut mmu) = new_test_cpu();

    // Sets ZF flag if result is 0
    cpu.regs.set_b(0b1000_0000);
    exec_cb!(cpu, mmu, SLA(Reg8(B)));
    assert_eq!(cpu.regs.get_flag(ZF), true);

    // Does not set ZF flag if result is 0
    cpu.regs.set_b(0b0100_0000);
    exec_cb!(cpu, mmu, SLA(Reg8(B)));
    assert_eq!(cpu.regs.get_flag(ZF), false);
  }

  #[test]
  fn opcode_cb_sra_d() {
    let (mut cpu, mut mmu) = new_test_cpu();
    cpu.regs.set_b(0b0000_0010);

    exec_cb!(cpu, mmu, SRA(Reg8(B)));

    assert_eq!(cpu.regs.b(), 0b0000_0001);
  }

  #[test]
  fn opcode_cb_sra_d_flags() {
    let (mut cpu, mut mmu) = new_test_cpu();

    // Sets ZF flag if result is 0
    cpu.regs.set_b(0b0000_0001);
    exec_cb!(cpu, mmu, SRA(Reg8(B)));
    assert_eq!(cpu.regs.get_flag(ZF), true);

    // Does not set ZF flag if result is 0
    cpu.regs.set_b(0b0000_0010);
    exec_cb!(cpu, mmu, SRA(Reg8(B)));
    assert_eq!(cpu.regs.get_flag(ZF), false);
  }

  #[test]
  fn opcode_cb_swap_d() {
    let (mut cpu, mut mmu) = new_test_cpu();
    cpu.regs.set_hl(0xff90);
    mmu.write8(0xff90u16, 0x12);

    exec_cb!(cpu, mmu, SWAP(PtrReg16(HL)));

    assert_eq!(mmu.read8(0xff90u16), 0x21);
  }

  #[test]
  fn opcode_cb_srl_d() {
    let (mut cpu, mut mmu) = new_test_cpu();
    cpu.regs.set_b(0b0000_0010);

    exec_cb!(cpu, mmu, SRL(Reg8(B)));

    assert_eq!(cpu.regs.b(), 0b0000_0001);
  }

  #[test]
  fn opcode_cb_srl_d_flags() {
    let (mut cpu, mut mmu) = new_test_cpu();

    // Sets ZF flag if result is 0
    cpu.regs.set_b(0b0000_0001);
    exec_cb!(cpu, mmu, SRL(Reg8(B)));
    assert_eq!(cpu.regs.get_flag(ZF), true);

    // Does not set ZF flag if result is 0
    cpu.regs.set_b(0b0000_0010);
    exec_cb!(cpu, mmu, SRL(Reg8(B)));
    assert_eq!(cpu.regs.get_flag(ZF), false);
  }

  #[test]
  fn opcode_cb_bit_n_d_flags() {
    let (mut cpu, mut mmu) = new_test_cpu();

    // BIT N, B sets ZF if bit N is zero
    cpu.regs.set_b(0b0000_0000);
    exec_cb!(cpu, mmu, BIT(0, Reg8(B)));
    assert_eq!(cpu.regs.get_flag(ZF), true);

    // BIT N, C does not change ZF if bit N is 1
    cpu.regs.set_flag(ZF, false);
    cpu.regs.set_c(0b0000_0100);
    exec_cb!(cpu, mmu, BIT(2, Reg8(C)));
    assert_eq!(cpu.regs.get_flag(ZF), false);

    // BIT N, (HL) sets ZF if bit N is zero
    cpu.regs.set_hl(123);
    mmu.write8(123u16, 0b0000_0000);
    exec_cb!(cpu, mmu, BIT(0, Reg8(B)));
    assert_eq!(cpu.regs.get_flag(ZF), true);

    // BIT N, A does not change ZF if bit N is 1
    cpu.regs.set_flag(ZF, false);
    cpu.regs.set_a(0b1000_0000);
    exec_cb!(cpu, mmu, BIT(7, Reg8(A)));
    assert_eq!(cpu.regs.get_flag(ZF), false);
  }

  #[test]
  fn opcode_cb_res_n_d() {
    let (mut cpu, mut mmu) = new_test_cpu();

    // RES N, B
    cpu.regs.set_b(0xFF);
    exec_cb!(cpu, mmu, RES(2, Reg8(B)));
    assert_eq!(cpu.regs.b(), 0b1111_1011);

    // RES N, (HL)
    cpu.regs.set_hl(0xff90);
    mmu.write8(0xff90u16, 0xFF);
    exec_cb!(cpu, mmu, RES(2, PtrReg16(HL)));
    assert_eq!(mmu.read8(0xff90u16), 0b1111_1011);
  }

  #[test]
  fn opcode_cb_set_n_d() {
    let (mut cpu, mut mmu) = new_test_cpu();

    // SET N, B
    cpu.regs.set_b(0x00);
    exec_cb!(cpu, mmu, SET(2, Reg8(B)));
    assert_eq!(cpu.regs.b(), 0b0000_0100);

    // SET N, (HL)
    cpu.regs.set_hl(0xff90);
    mmu.write8(0xff90u16, 0x00);
    exec_cb!(cpu, mmu, SET(2, PtrReg16(HL)));
    assert_eq!(mmu.read8(0xff90u16), 0b0000_0100);
  }
}
