#[derive(Debug, PartialEq)]
pub enum Opcode {
  Nop,
  LD(Arg, Arg),
  ADD(Arg, Arg),
  INC(Arg),
  DEC(Arg),
  RLCA,
  RRCA,
  RLA,
  RRA,
  STOP,
  JUMP(JumpCondition, Arg),
  LDI(Arg, Arg),
  LDD(Arg, Arg),
  DAA,
  CPL,
  SCF,
  CCF,
  HALT,
  ALU(AluOp, Arg, Arg),
  POP(registers::Register16),
  PUSH(registers::Register16),
  RST(u8),
  RET(JumpCondition),
  RETI,
  CALL(JumpCondition, Arg),
  DI,
  EI,
  CALLBACK,
}

use super::registers;
use super::registers::{Register16::*, Register8::*};
use AluOp::*;
use Arg::*;
use JumpCondition::*;
use Opcode::*;

#[derive(Debug, PartialEq)]
pub enum Arg {
  Addr16,
  Imm8,
  Imm16,
  PtrHL,
  Reg8(registers::Register8),
  Reg16(registers::Register16),
  SPPlusImm8,
  FF00PlusImm8,
  PtrC,
}

#[derive(Debug, PartialEq)]
pub enum JumpCondition {
  Always,
  NotZero,
  Zero,
  NotCarry,
  Carry,
}

#[derive(Debug, PartialEq)]
pub enum AluOp {
  Add,
  Adc,
  Sub,
  Sbc,
  And,
  Xor,
  Or,
  Cp,
}

fn op_match(opcode: u8, mask: u8, expectation: u8) -> bool {
  opcode & mask == expectation
}

pub fn decode(byte: u8) -> Opcode {
  match byte {
    0x0 => Nop,
    0b0000_1000 => LD(Addr16, Reg16(SP)),
    _ if op_match(byte, 0b1100_1111, 0b0000_0001) => LD(Reg16(reg16(byte, 1, 2)), Imm16),
    _ if op_match(byte, 0b1100_1111, 0b0000_1001) => ADD(Reg16(HL), Reg16(reg16(byte, 1, 2))),
    _ if op_match(byte, 0b1110_1111, 0b0000_0010) => LD(Reg16(reg16(byte, 0, 2)), Reg8(A)),
    _ if op_match(byte, 0b1110_1111, 0b0000_0010) => LD(Reg16(reg16(byte, 0, 2)), Reg8(A)),
    _ if op_match(byte, 0b1110_1111, 0b0000_1010) => LD(Reg8(A), Reg16(reg16(byte, 0, 2))),
    _ if op_match(byte, 0b1100_1111, 0b0000_0011) => INC(Reg16(reg16(byte, 1, 2))),
    _ if op_match(byte, 0b1100_1111, 0b0000_1011) => DEC(Reg16(reg16(byte, 1, 2))),
    _ if op_match(byte, 0b1100_0111, 0b0000_0100) => INC(destination(byte, 2)),
    _ if op_match(byte, 0b1100_0111, 0b0000_0101) => DEC(destination(byte, 2)),
    _ if op_match(byte, 0b1100_0111, 0b0000_0110) => LD(destination(byte, 2), Imm8),
    0b0000_0111 => RLCA,
    0b0000_1111 => RRCA,
    0b0001_0111 => RLA,
    0b0001_1111 => RRA,
    0b0001_0000 => STOP,
    0b0001_1000 => JUMP(Always, Imm8),
    _ if op_match(byte, 0b1110_0111, 0b0010_0000) => JUMP(condition(byte, 3), Imm8),
    0b0010_0010 => LDI(PtrHL, Reg8(A)),
    0b0010_1010 => LDI(Reg8(A), PtrHL),
    0b0011_0010 => LDD(PtrHL, Reg8(A)),
    0b0011_1010 => LDD(Reg8(A), PtrHL),
    0b0010_0111 => DAA,
    0b0010_1111 => CPL,
    0b0011_0111 => SCF,
    0b0011_1111 => CCF,
    0b0111_0110 => HALT,
    _ if op_match(byte, 0b1100_0000, 0b0100_0000) => LD(destination(byte, 2), destination(byte, 5)),
    _ if op_match(byte, 0b1100_0111, 0b1100_0110) => ALU(operation(byte, 2), Reg8(A), Imm8),
    _ if op_match(byte, 0b1100_0000, 0b1000_0000) => {
      ALU(operation(byte, 2), Reg8(A), destination(byte, 5))
    }
    _ if op_match(byte, 0b11001111, 0b11000001) => POP(reg16(byte, 2, 2)),
    _ if op_match(byte, 0b11001111, 0b11000101) => PUSH(reg16(byte, 2, 2)),
    _ if op_match(byte, 0b11000111, 0b11000111) => RST(n(byte, 2)),
    _ if op_match(byte, 0b11100111, 0b11000000) => RET(condition(byte, 3)),
    0b110_01001 => RET(Always),
    0b110_11001 => RETI,
    _ if op_match(byte, 0b11100111, 0b11000010) => JUMP(condition(byte, 3), Addr16),
    0b110_00011 => JUMP(Always, Addr16),
    _ if op_match(byte, 0b11100111, 0b11000100) => CALL(condition(byte, 3), Addr16),
    0b110_01101 => CALL(Always, Addr16),
    0b1110_1000 => ADD(Reg16(SP), Imm8),
    0b1111_1000 => LD(Reg16(HL), SPPlusImm8),
    0b1110_0000 => LD(FF00PlusImm8, Reg8(A)),
    0b1111_0000 => LD(Reg8(A), FF00PlusImm8),
    0b1110_0010 => LD(PtrC, Reg8(A)),
    0b1111_0010 => LD(Reg8(A), PtrC),
    0b1110_1010 => LD(Addr16, Reg8(A)),
    0b1111_1010 => LD(Reg8(A), Addr16),
    0b1110_1001 => JUMP(Always, Reg16(HL)),
    0b1111_1001 => LD(Reg16(SP), Reg16(HL)),
    0b1111_0011 => DI,
    0b1111_1011 => EI,
    0b11001011 => CALLBACK,
    _ => panic!("Invalid opcode {:#02b}", byte),
  }
}

fn reg16(byte: u8, column: usize, index: usize) -> registers::Register16 {
  let row = (byte >> (8 - index - 2)) & 0b0011;

  match (column, row) {
    (0, 0) => BC,
    (0, 1) => DE,
    (1, 0) => BC,
    (1, 1) => DE,
    (1, 2) => HL,
    (1, 3) => SP,
    (2, 0) => BC,
    (2, 1) => DE,
    (2, 2) => HL,
    (2, 3) => AF,
    _ => unreachable!(),
  }
}

fn destination(byte: u8, index: usize) -> Arg {
  let row = (byte >> (8 - index - 3)) & 0b0111;

  match row {
    0 => Reg8(B),
    1 => Reg8(C),
    2 => Reg8(D),
    3 => Reg8(E),
    4 => Reg8(H),
    5 => Reg8(L),
    6 => PtrHL,
    7 => Reg8(A),
    _ => unreachable!(),
  }
}

fn condition(byte: u8, index: usize) -> JumpCondition {
  match (byte >> (8 - index - 2)) & 0b0011 {
    0 => NotZero,
    1 => Zero,
    2 => NotCarry,
    3 => Carry,
    _ => unreachable!(),
  }
}

fn operation(byte: u8, index: usize) -> AluOp {
  match (byte >> (8 - index - 3)) & 0b0111 {
    0 => Add,
    1 => Adc,
    2 => Sub,
    3 => Sbc,
    4 => And,
    5 => Xor,
    6 => Or,
    7 => Cp,
    _ => unreachable!(),
  }
}

fn n(byte: u8, index: usize) -> u8 {
  byte >> (8 - index - 3) & 0b0111
}

#[cfg(test)]
mod test {
  use super::*;
  use super::{AluOp::*, Arg::*, JumpCondition::*, Opcode::*};

  macro_rules! assert_decode {
    ($op:expr, $expectation:expr) => {{
      assert_eq!(super::decode($op), $expectation);
    }};
  }

  #[test]
  fn decode_nop() {
    assert_decode!(0x0, Nop);
  }

  #[test]
  fn decode_ld_n16_sp() {
    assert_decode!(0b0000_1000, LD(Addr16, Reg16(SP)));
  }

  #[test]
  fn decode_ld_r16_imm() {
    assert_decode!(0b0000_0001, LD(Reg16(BC), Imm16));
    assert_decode!(0b0001_0001, LD(Reg16(DE), Imm16));
    assert_decode!(0b0010_0001, LD(Reg16(HL), Imm16));
    assert_decode!(0b0011_0001, LD(Reg16(SP), Imm16));
  }

  #[test]
  fn decode_add_hl_r16() {
    assert_decode!(0b0000_1001, ADD(Reg16(HL), Reg16(BC)));
    assert_decode!(0b0001_1001, ADD(Reg16(HL), Reg16(DE)));
    assert_decode!(0b0010_1001, ADD(Reg16(HL), Reg16(HL)));
    assert_decode!(0b0011_1001, ADD(Reg16(HL), Reg16(SP)));
  }

  #[test]
  fn decode_ld_r16_a() {
    assert_decode!(0b0000_0010, LD(Reg16(BC), Reg8(A)));
    assert_decode!(0b0001_0010, LD(Reg16(DE), Reg8(A)));
  }

  #[test]
  fn decode_ld_a_r16() {
    assert_decode!(0b0000_1010, LD(Reg8(A), Reg16(BC)));
    assert_decode!(0b0001_1010, LD(Reg8(A), Reg16(DE)));
  }

  #[test]
  fn decode_inc_r() {
    assert_decode!(0b0000_0011, INC(Reg16(BC)));
    assert_decode!(0b0001_0011, INC(Reg16(DE)));
    assert_decode!(0b0010_0011, INC(Reg16(HL)));
    assert_decode!(0b0011_0011, INC(Reg16(SP)));
  }

  #[test]
  fn decode_dec_r() {
    assert_decode!(0b0000_1011, DEC(Reg16(BC)));
    assert_decode!(0b0001_1011, DEC(Reg16(DE)));
    assert_decode!(0b0010_1011, DEC(Reg16(HL)));
    assert_decode!(0b0011_1011, DEC(Reg16(SP)));
  }

  #[test]
  fn decode_inc_d() {
    assert_decode!(0b0000_0100, INC(Reg8(B)));
    assert_decode!(0b0000_1100, INC(Reg8(C)));
    assert_decode!(0b0001_0100, INC(Reg8(D)));
    assert_decode!(0b0001_1100, INC(Reg8(E)));
    assert_decode!(0b0010_0100, INC(Reg8(H)));
    assert_decode!(0b0010_1100, INC(Reg8(L)));
    assert_decode!(0b0011_0100, INC(PtrHL));
    assert_decode!(0b0011_1100, INC(Reg8(A)));
  }

  #[test]
  fn decode_dec_d() {
    assert_decode!(0b0000_0101, DEC(Reg8(B)));
    assert_decode!(0b0000_1101, DEC(Reg8(C)));
    assert_decode!(0b0001_0101, DEC(Reg8(D)));
    assert_decode!(0b0001_1101, DEC(Reg8(E)));
    assert_decode!(0b0010_0101, DEC(Reg8(H)));
    assert_decode!(0b0010_1101, DEC(Reg8(L)));
    assert_decode!(0b0011_0101, DEC(PtrHL));
    assert_decode!(0b0011_1101, DEC(Reg8(A)));
  }

  #[test]
  fn ld_d_n() {
    assert_decode!(0b0000_0110, LD(Reg8(B), Imm8));
    assert_decode!(0b0000_1110, LD(Reg8(C), Imm8));
    assert_decode!(0b0001_0110, LD(Reg8(D), Imm8));
    assert_decode!(0b0001_1110, LD(Reg8(E), Imm8));
    assert_decode!(0b0010_0110, LD(Reg8(H), Imm8));
    assert_decode!(0b0010_1110, LD(Reg8(L), Imm8));
    assert_decode!(0b0011_0110, LD(PtrHL, Imm8));
    assert_decode!(0b0011_1110, LD(Reg8(A), Imm8));
  }

  #[test]
  fn rd_ca() {
    assert_decode!(0b0000_0111, RLCA);
    assert_decode!(0b0000_1111, RRCA);
  }

  #[test]
  fn rd_a() {
    assert_decode!(0b0001_0111, RLA);
    assert_decode!(0b0001_1111, RRA);
  }

  #[test]
  fn stop() {
    assert_decode!(0b0001_0000, STOP);
  }

  #[test]
  fn j_r_n() {
    assert_decode!(0b0001_1000, JUMP(Always, Imm8));
  }

  #[test]
  fn j_r_f_n() {
    assert_decode!(0b0010_0000, JUMP(NotZero, Imm8));
    assert_decode!(0b0010_1000, JUMP(Zero, Imm8));
    assert_decode!(0b0011_0000, JUMP(NotCarry, Imm8));
    assert_decode!(0b0011_1000, JUMP(Carry, Imm8));
  }

  #[test]
  fn ldi_ldd() {
    assert_decode!(0b0010_0010, LDI(PtrHL, Reg8(A)));
    assert_decode!(0b0010_1010, LDI(Reg8(A), PtrHL));
    assert_decode!(0b0011_0010, LDD(PtrHL, Reg8(A)));
    assert_decode!(0b0011_1010, LDD(Reg8(A), PtrHL));
  }

  #[test]
  fn daa() {
    assert_decode!(0b0010_0111, DAA);
  }

  #[test]
  fn cpl() {
    assert_decode!(0b0010_1111, CPL);
  }

  #[test]
  fn scf() {
    assert_decode!(0b0011_0111, SCF);
  }

  #[test]
  fn ccf() {
    assert_decode!(0b0011_1111, CCF);
  }

  #[test]
  fn ld_d_d() {
    assert_decode!(0b0100_0000, LD(Reg8(B), Reg8(B)));
    assert_decode!(0b0100_1000, LD(Reg8(C), Reg8(B)));
    assert_decode!(0b0101_0000, LD(Reg8(D), Reg8(B)));
    assert_decode!(0b0101_1000, LD(Reg8(E), Reg8(B)));
    assert_decode!(0b0110_0000, LD(Reg8(H), Reg8(B)));
    assert_decode!(0b0110_1000, LD(Reg8(L), Reg8(B)));
    assert_decode!(0b0111_0000, LD(PtrHL, Reg8(B)));
    assert_decode!(0b0111_1000, LD(Reg8(A), Reg8(B)));

    assert_decode!(0b0100_0001, LD(Reg8(B), Reg8(C)));
    assert_decode!(0b0100_1001, LD(Reg8(C), Reg8(C)));
    assert_decode!(0b0101_0001, LD(Reg8(D), Reg8(C)));
    assert_decode!(0b0101_1001, LD(Reg8(E), Reg8(C)));
    assert_decode!(0b0110_0001, LD(Reg8(H), Reg8(C)));
    assert_decode!(0b0110_1001, LD(Reg8(L), Reg8(C)));
    assert_decode!(0b0111_0001, LD(PtrHL, Reg8(C)));
    assert_decode!(0b0111_1001, LD(Reg8(A), Reg8(C)));
  }

  #[test]
  fn halt() {
    assert_decode!(0b0111_0110, HALT);
  }

  #[test]
  fn alu_a_d() {
    assert_decode!(0b10_000_000, ALU(Add, Reg8(A), Reg8(B)));
    assert_decode!(0b10_000_001, ALU(Add, Reg8(A), Reg8(C)));
    assert_decode!(0b10_000_010, ALU(Add, Reg8(A), Reg8(D)));
    assert_decode!(0b10_000_011, ALU(Add, Reg8(A), Reg8(E)));
    assert_decode!(0b10_000_100, ALU(Add, Reg8(A), Reg8(H)));
    assert_decode!(0b10_000_101, ALU(Add, Reg8(A), Reg8(L)));
    assert_decode!(0b10_000_110, ALU(Add, Reg8(A), PtrHL));
    assert_decode!(0b10_000_111, ALU(Add, Reg8(A), Reg8(A)));

    assert_decode!(0b10_001_000, ALU(Adc, Reg8(A), Reg8(B)));
    assert_decode!(0b10_001_001, ALU(Adc, Reg8(A), Reg8(C)));
    assert_decode!(0b10_001_010, ALU(Adc, Reg8(A), Reg8(D)));
    assert_decode!(0b10_001_011, ALU(Adc, Reg8(A), Reg8(E)));
    assert_decode!(0b10_001_100, ALU(Adc, Reg8(A), Reg8(H)));
    assert_decode!(0b10_001_101, ALU(Adc, Reg8(A), Reg8(L)));
    assert_decode!(0b10_001_110, ALU(Adc, Reg8(A), PtrHL));
    assert_decode!(0b10_001_111, ALU(Adc, Reg8(A), Reg8(A)));

    assert_decode!(0b10_010_000, ALU(Sub, Reg8(A), Reg8(B)));
    assert_decode!(0b10_010_001, ALU(Sub, Reg8(A), Reg8(C)));
    assert_decode!(0b10_010_010, ALU(Sub, Reg8(A), Reg8(D)));
    assert_decode!(0b10_010_011, ALU(Sub, Reg8(A), Reg8(E)));
    assert_decode!(0b10_010_100, ALU(Sub, Reg8(A), Reg8(H)));
    assert_decode!(0b10_010_101, ALU(Sub, Reg8(A), Reg8(L)));
    assert_decode!(0b10_010_110, ALU(Sub, Reg8(A), PtrHL));
    assert_decode!(0b10_010_111, ALU(Sub, Reg8(A), Reg8(A)));

    assert_decode!(0b10_011_000, ALU(Sbc, Reg8(A), Reg8(B)));
    assert_decode!(0b10_011_001, ALU(Sbc, Reg8(A), Reg8(C)));
    assert_decode!(0b10_011_010, ALU(Sbc, Reg8(A), Reg8(D)));
    assert_decode!(0b10_011_011, ALU(Sbc, Reg8(A), Reg8(E)));
    assert_decode!(0b10_011_100, ALU(Sbc, Reg8(A), Reg8(H)));
    assert_decode!(0b10_011_101, ALU(Sbc, Reg8(A), Reg8(L)));
    assert_decode!(0b10_011_110, ALU(Sbc, Reg8(A), PtrHL));
    assert_decode!(0b10_011_111, ALU(Sbc, Reg8(A), Reg8(A)));

    assert_decode!(0b10_100_000, ALU(And, Reg8(A), Reg8(B)));
    assert_decode!(0b10_100_001, ALU(And, Reg8(A), Reg8(C)));
    assert_decode!(0b10_100_010, ALU(And, Reg8(A), Reg8(D)));
    assert_decode!(0b10_100_011, ALU(And, Reg8(A), Reg8(E)));
    assert_decode!(0b10_100_100, ALU(And, Reg8(A), Reg8(H)));
    assert_decode!(0b10_100_101, ALU(And, Reg8(A), Reg8(L)));
    assert_decode!(0b10_100_110, ALU(And, Reg8(A), PtrHL));
    assert_decode!(0b10_100_111, ALU(And, Reg8(A), Reg8(A)));

    assert_decode!(0b10_101_000, ALU(Xor, Reg8(A), Reg8(B)));
    assert_decode!(0b10_101_001, ALU(Xor, Reg8(A), Reg8(C)));
    assert_decode!(0b10_101_010, ALU(Xor, Reg8(A), Reg8(D)));
    assert_decode!(0b10_101_011, ALU(Xor, Reg8(A), Reg8(E)));
    assert_decode!(0b10_101_100, ALU(Xor, Reg8(A), Reg8(H)));
    assert_decode!(0b10_101_101, ALU(Xor, Reg8(A), Reg8(L)));
    assert_decode!(0b10_101_110, ALU(Xor, Reg8(A), PtrHL));
    assert_decode!(0b10_101_111, ALU(Xor, Reg8(A), Reg8(A)));

    assert_decode!(0b10_110_000, ALU(Or, Reg8(A), Reg8(B)));
    assert_decode!(0b10_110_001, ALU(Or, Reg8(A), Reg8(C)));
    assert_decode!(0b10_110_010, ALU(Or, Reg8(A), Reg8(D)));
    assert_decode!(0b10_110_011, ALU(Or, Reg8(A), Reg8(E)));
    assert_decode!(0b10_110_100, ALU(Or, Reg8(A), Reg8(H)));
    assert_decode!(0b10_110_101, ALU(Or, Reg8(A), Reg8(L)));
    assert_decode!(0b10_110_110, ALU(Or, Reg8(A), PtrHL));
    assert_decode!(0b10_110_111, ALU(Or, Reg8(A), Reg8(A)));

    assert_decode!(0b10_111_000, ALU(Cp, Reg8(A), Reg8(B)));
    assert_decode!(0b10_111_001, ALU(Cp, Reg8(A), Reg8(C)));
    assert_decode!(0b10_111_010, ALU(Cp, Reg8(A), Reg8(D)));
    assert_decode!(0b10_111_011, ALU(Cp, Reg8(A), Reg8(E)));
    assert_decode!(0b10_111_100, ALU(Cp, Reg8(A), Reg8(H)));
    assert_decode!(0b10_111_101, ALU(Cp, Reg8(A), Reg8(L)));
    assert_decode!(0b10_111_110, ALU(Cp, Reg8(A), PtrHL));
    assert_decode!(0b10_111_111, ALU(Cp, Reg8(A), Reg8(A)));
  }

  #[test]
  fn alu_a_n() {
    assert_decode!(0b11_000_110, ALU(Add, Reg8(A), Imm8));
    assert_decode!(0b11_001_110, ALU(Adc, Reg8(A), Imm8));
    assert_decode!(0b11_010_110, ALU(Sub, Reg8(A), Imm8));
    assert_decode!(0b11_011_110, ALU(Sbc, Reg8(A), Imm8));
    assert_decode!(0b11_100_110, ALU(And, Reg8(A), Imm8));
    assert_decode!(0b11_101_110, ALU(Xor, Reg8(A), Imm8));
    assert_decode!(0b11_110_110, ALU(Or, Reg8(A), Imm8));
    assert_decode!(0b11_111_110, ALU(Cp, Reg8(A), Imm8));
  }

  #[test]
  fn pop_r() {
    assert_decode!(0b11_00_0001, POP(BC));
    assert_decode!(0b11_01_0001, POP(DE));
    assert_decode!(0b11_10_0001, POP(HL));
    assert_decode!(0b11_11_0001, POP(AF));
  }

  #[test]
  fn push_r() {
    assert_decode!(0b11_00_0101, PUSH(BC));
    assert_decode!(0b11_01_0101, PUSH(DE));
    assert_decode!(0b11_10_0101, PUSH(HL));
    assert_decode!(0b11_11_0101, PUSH(AF));
  }

  #[test]
  fn rst_n() {
    assert_decode!(0b11_000_111, RST(0));
    assert_decode!(0b11_001_111, RST(1));
    assert_decode!(0b11_010_111, RST(2));
    assert_decode!(0b11_011_111, RST(3));
    assert_decode!(0b11_100_111, RST(4));
    assert_decode!(0b11_101_111, RST(5));
    assert_decode!(0b11_110_111, RST(6));
    assert_decode!(0b11_111_111, RST(7));
  }

  #[test]
  fn ret_f() {
    assert_decode!(0b110_00_000, RET(NotZero));
    assert_decode!(0b110_01_000, RET(Zero));
    assert_decode!(0b110_10_000, RET(NotCarry));
    assert_decode!(0b110_11_000, RET(Carry));
  }

  #[test]
  fn ret() {
    assert_decode!(0b110_01_001, RET(Always));
  }

  #[test]
  fn reti() {
    assert_decode!(0b110_11001, RETI);
  }

  #[test]
  fn jp_f_n() {
    assert_decode!(0b110_00_010, JUMP(NotZero, Addr16));
    assert_decode!(0b110_01_010, JUMP(Zero, Addr16));
    assert_decode!(0b110_10_010, JUMP(NotCarry, Addr16));
    assert_decode!(0b110_11_010, JUMP(Carry, Addr16));
  }

  #[test]
  fn jp_n() {
    assert_decode!(0b110_00_011, JUMP(Always, Addr16));
  }

  #[test]
  fn call_f_n() {
    assert_decode!(0b110_00_100, CALL(NotZero, Addr16));
    assert_decode!(0b110_01_100, CALL(Zero, Addr16));
    assert_decode!(0b110_10_100, CALL(NotCarry, Addr16));
    assert_decode!(0b110_11_100, CALL(Carry, Addr16));
  }

  #[test]
  fn call_n() {
    assert_decode!(0b110_01_101, CALL(Always, Addr16));
  }

  #[test]
  fn add_sp_n() {
    assert_decode!(0b111_01000, ADD(Reg16(SP), Imm8));
  }

  #[test]
  fn ld_hl_sp_n() {
    assert_decode!(0b111_11000, LD(Reg16(HL), SPPlusImm8));
  }

  #[test]
  fn ld_high_mem_a() {
    assert_decode!(0b111_00000, LD(FF00PlusImm8, Reg8(A)));
  }

  #[test]
  fn ld_a_high_mem() {
    assert_decode!(0b111_10000, LD(Reg8(A), FF00PlusImm8));
  }

  #[test]
  fn ld_c_a() {
    assert_decode!(0b111_00010, LD(PtrC, Reg8(A)));
  }

  #[test]
  fn ld_a_c() {
    assert_decode!(0b111_10010, LD(Reg8(A), PtrC));
  }

  #[test]
  fn ld_n_a() {
    assert_decode!(0b111_01010, LD(Addr16, Reg8(A)));
  }

  #[test]
  fn ld_a_n() {
    assert_decode!(0b111_11010, LD(Reg8(A), Addr16));
  }

  #[test]
  fn jp_hl() {
    assert_decode!(0b1110_1001, JUMP(Always, Reg16(HL)));
  }

  #[test]
  fn ld_sp_hl() {
    assert_decode!(0b1111_1001, LD(Reg16(SP), Reg16(HL)));
  }

  #[test]
  fn di() {
    assert_decode!(0b1111_0011, DI);
  }

  #[test]
  fn ei() {
    assert_decode!(0b1111_1011, EI);
  }

  #[test]
  fn callback() {
    assert_decode!(0b1100_1011, CALLBACK);
  }
}
