//
// should be derived from
// http://goldencrystal.free.fr/GBZ80Opcodes.pdf
//

use super::registers::{Register16, Register8};

#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq)]
pub enum Arg {
  R8(Register8),
  R16(Register16),
  U8(u8),
  U16(u16),
}

#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq)]
pub enum OpCode {
  I_NOP,
  I_HALT,
  I_LD(Arg, Arg),
  I_ADD(Arg, Arg),
  I_ADC(Arg, Arg),
  I_SUB(Arg),
  I_SBC(Arg, Arg),
  I_AND(Arg),
  I_XOR(Arg),
  I_OR(Arg),
  I_CP(Arg),
  I_INC(Arg),
}

use Arg::*;
use OpCode::*;
use Register16::*;
use Register8::*;

pub fn decode(opcode: u8) -> OpCode {
  match opcode {
    0x76 => I_HALT,
    0x00 => I_NOP,

    0x03 | 0x13 | 0x23 | 0x33 => I_INC(reg16_1(opcode)),
    0x0e | 0x1e | 0x2e | 0x3e => panic!("requires reading second byte"),

    0x40..=0x7f => I_LD(ld_reg_1(opcode), reg_2(opcode)),
    0x80..=0x87 => I_ADD(R8(A), reg_2(opcode)),
    0x88..=0x8f => I_ADC(R8(A), reg_2(opcode)),
    0x90..=0x97 => I_SUB(reg_2(opcode)),
    0x98..=0x9f => I_SBC(R8(A), reg_2(opcode)),
    0xa0..=0xa7 => I_AND(reg_2(opcode)),
    0xa8..=0xaf => I_XOR(reg_2(opcode)),
    0xb0..=0xb7 => I_OR(reg_2(opcode)),
    0xb8..=0xbf => I_CP(reg_2(opcode)),

    _ => panic!("Unknown opcode {}", opcode),
  }
}

fn reg_2(opcode: u8) -> Arg {
  match opcode & 0x07 {
    0x00 => R8(B),
    0x01 => R8(C),
    0x02 => R8(D),
    0x03 => R8(E),
    0x04 => R8(H),
    0x05 => R8(L),
    0x06 => R16(HL),
    0x07 => R8(A),
    _ => panic!("Can't match reg8_2 on opcode: 0x{:x}", opcode),
  }
}

fn ld_reg_1(opcode: u8) -> Arg {
  match opcode & 0x38 {
    0x00 => R8(B),
    0x08 => R8(C),
    0x10 => R8(D),
    0x18 => R8(E),
    0x20 => R8(H),
    0x28 => R8(L),
    0x30 => R16(HL),
    0x38 => R8(A),
    _ => panic!("Can't match reg8_1 on opcode: 0x{:x}", opcode),
  }
}

fn reg8_2(opcode: u8) -> Arg {
  let res = match opcode & 0x07 {
    0x00 => B,
    0x01 => C,
    0x02 => D,
    0x03 => E,
    0x04 => H,
    0x05 => L,
    0x07 => A,
    _ => panic!("Can't match reg8_2 on opcode: 0x{:x}", opcode),
  };

  R8(res)
}

fn reg16_1(opcode: u8) -> Arg {
  let res = match opcode & 0x30 {
    0x00 => BC,
    0x10 => DE,
    0x20 => HL,
    0x30 => SP,
    _ => panic!("Can't match reg16_2 on opcode: 0x{:x}", opcode),
  };

  R16(res)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_decode_0x0x() {
    assert_eq!(decode(0x03), I_INC(R16(BC)));
  }

  #[test]
  fn test_decode_0x1x() {
    assert_eq!(decode(0x13), I_INC(R16(DE)));
  }
  #[test]
  fn test_decode_0x2x() {
    assert_eq!(decode(0x23), I_INC(R16(HL)));
  }
  #[test]
  fn test_decode_0x3x() {
    assert_eq!(decode(0x33), I_INC(R16(SP)));
  }

  #[test]
  fn test_decode_0x4x() {
    assert_eq!(decode(0x40), I_LD(R8(B), R8(B)));
    assert_eq!(decode(0x41), I_LD(R8(B), R8(C)));
    assert_eq!(decode(0x42), I_LD(R8(B), R8(D)));
    assert_eq!(decode(0x43), I_LD(R8(B), R8(E)));
    assert_eq!(decode(0x44), I_LD(R8(B), R8(H)));
    assert_eq!(decode(0x45), I_LD(R8(B), R8(L)));
    assert_eq!(decode(0x46), I_LD(R8(B), R16(HL)));
    assert_eq!(decode(0x47), I_LD(R8(B), R8(A)));
    assert_eq!(decode(0x48), I_LD(R8(C), R8(B)));
    assert_eq!(decode(0x49), I_LD(R8(C), R8(C)));
    assert_eq!(decode(0x4a), I_LD(R8(C), R8(D)));
    assert_eq!(decode(0x4b), I_LD(R8(C), R8(E)));
    assert_eq!(decode(0x4c), I_LD(R8(C), R8(H)));
    assert_eq!(decode(0x4d), I_LD(R8(C), R8(L)));
    assert_eq!(decode(0x4e), I_LD(R8(C), R16(HL)));
    assert_eq!(decode(0x4f), I_LD(R8(C), R8(A)));
  }

  #[test]
  fn test_decode_0x5x() {
    assert_eq!(decode(0x50), I_LD(R8(D), R8(B)));
    assert_eq!(decode(0x51), I_LD(R8(D), R8(C)));
    assert_eq!(decode(0x52), I_LD(R8(D), R8(D)));
    assert_eq!(decode(0x53), I_LD(R8(D), R8(E)));
    assert_eq!(decode(0x54), I_LD(R8(D), R8(H)));
    assert_eq!(decode(0x55), I_LD(R8(D), R8(L)));
    assert_eq!(decode(0x56), I_LD(R8(D), R16(HL)));
    assert_eq!(decode(0x57), I_LD(R8(D), R8(A)));
    assert_eq!(decode(0x58), I_LD(R8(E), R8(B)));
    assert_eq!(decode(0x59), I_LD(R8(E), R8(C)));
    assert_eq!(decode(0x5a), I_LD(R8(E), R8(D)));
    assert_eq!(decode(0x5b), I_LD(R8(E), R8(E)));
    assert_eq!(decode(0x5c), I_LD(R8(E), R8(H)));
    assert_eq!(decode(0x5d), I_LD(R8(E), R8(L)));
    assert_eq!(decode(0x5e), I_LD(R8(E), R16(HL)));
    assert_eq!(decode(0x5f), I_LD(R8(E), R8(A)));
  }

  #[test]
  fn test_decode_0x6x() {
    assert_eq!(decode(0x60), I_LD(R8(H), R8(B)));
    assert_eq!(decode(0x61), I_LD(R8(H), R8(C)));
    assert_eq!(decode(0x62), I_LD(R8(H), R8(D)));
    assert_eq!(decode(0x63), I_LD(R8(H), R8(E)));
    assert_eq!(decode(0x64), I_LD(R8(H), R8(H)));
    assert_eq!(decode(0x65), I_LD(R8(H), R8(L)));
    assert_eq!(decode(0x66), I_LD(R8(H), R16(HL)));
    assert_eq!(decode(0x67), I_LD(R8(H), R8(A)));
    assert_eq!(decode(0x68), I_LD(R8(L), R8(B)));
    assert_eq!(decode(0x69), I_LD(R8(L), R8(C)));
    assert_eq!(decode(0x6a), I_LD(R8(L), R8(D)));
    assert_eq!(decode(0x6b), I_LD(R8(L), R8(E)));
    assert_eq!(decode(0x6c), I_LD(R8(L), R8(H)));
    assert_eq!(decode(0x6d), I_LD(R8(L), R8(L)));
    assert_eq!(decode(0x6e), I_LD(R8(L), R16(HL)));
    assert_eq!(decode(0x6f), I_LD(R8(L), R8(A)));
  }

  #[test]
  fn test_decode_0x7x() {
    assert_eq!(decode(0x70), I_LD(R16(HL), R8(B)));
    assert_eq!(decode(0x71), I_LD(R16(HL), R8(C)));
    assert_eq!(decode(0x72), I_LD(R16(HL), R8(D)));
    assert_eq!(decode(0x73), I_LD(R16(HL), R8(E)));
    assert_eq!(decode(0x74), I_LD(R16(HL), R8(H)));
    assert_eq!(decode(0x75), I_LD(R16(HL), R8(L)));
    // assert_eq!(decode(0x76), IHA        )     )
    assert_eq!(decode(0x77), I_LD(R16(HL), R8(A)));

    assert_eq!(decode(0x78), I_LD(R8(A), R8(B)));
    assert_eq!(decode(0x79), I_LD(R8(A), R8(C)));
    assert_eq!(decode(0x7a), I_LD(R8(A), R8(D)));
    assert_eq!(decode(0x7b), I_LD(R8(A), R8(E)));
    assert_eq!(decode(0x7c), I_LD(R8(A), R8(H)));
    assert_eq!(decode(0x7d), I_LD(R8(A), R8(L)));
    assert_eq!(decode(0x7e), I_LD(R8(A), R16(HL)));
    assert_eq!(decode(0x7f), I_LD(R8(A), R8(A)));
  }

  #[test]
  fn test_decode_0x8x() {
    assert_eq!(decode(0x80), I_ADD(R8(A), R8(B)));
    assert_eq!(decode(0x81), I_ADD(R8(A), R8(C)));
    assert_eq!(decode(0x82), I_ADD(R8(A), R8(D)));
    assert_eq!(decode(0x83), I_ADD(R8(A), R8(E)));
    assert_eq!(decode(0x84), I_ADD(R8(A), R8(H)));
    assert_eq!(decode(0x85), I_ADD(R8(A), R8(L)));

    assert_eq!(decode(0x87), I_ADD(R8(A), R8(A)));
    assert_eq!(decode(0x88), I_ADC(R8(A), R8(B)));
    assert_eq!(decode(0x89), I_ADC(R8(A), R8(C)));
    assert_eq!(decode(0x8a), I_ADC(R8(A), R8(D)));
    assert_eq!(decode(0x8b), I_ADC(R8(A), R8(E)));
    assert_eq!(decode(0x8c), I_ADC(R8(A), R8(H)));
    assert_eq!(decode(0x8d), I_ADC(R8(A), R8(L)));

    assert_eq!(decode(0x8f), I_ADC(R8(A), R8(A)));
  }
  #[test]
  fn test_decode_0x9x() {
    assert_eq!(decode(0x90), I_SUB(R8(B)));
    assert_eq!(decode(0x91), I_SUB(R8(C)));
    assert_eq!(decode(0x92), I_SUB(R8(D)));
    assert_eq!(decode(0x93), I_SUB(R8(E)));
    assert_eq!(decode(0x94), I_SUB(R8(H)));
    assert_eq!(decode(0x95), I_SUB(R8(L)));

    assert_eq!(decode(0x97), I_SUB(R8(A)));
    assert_eq!(decode(0x98), I_SBC(R8(A), R8(B)));
    assert_eq!(decode(0x99), I_SBC(R8(A), R8(C)));
    assert_eq!(decode(0x9a), I_SBC(R8(A), R8(D)));
    assert_eq!(decode(0x9b), I_SBC(R8(A), R8(E)));
    assert_eq!(decode(0x9c), I_SBC(R8(A), R8(H)));
    assert_eq!(decode(0x9d), I_SBC(R8(A), R8(L)));

    assert_eq!(decode(0x9f), I_SBC(R8(A), R8(A)));
  }
  #[test]
  fn test_decode_0xax() {
    assert_eq!(decode(0xa0), I_AND(R8(B)));
    assert_eq!(decode(0xa1), I_AND(R8(C)));
    assert_eq!(decode(0xa2), I_AND(R8(D)));
    assert_eq!(decode(0xa3), I_AND(R8(E)));
    assert_eq!(decode(0xa4), I_AND(R8(H)));
    assert_eq!(decode(0xa5), I_AND(R8(L)));

    assert_eq!(decode(0xa7), I_AND(R8(A)));
    assert_eq!(decode(0xa8), I_XOR(R8(B)));
    assert_eq!(decode(0xa9), I_XOR(R8(C)));
    assert_eq!(decode(0xaa), I_XOR(R8(D)));
    assert_eq!(decode(0xab), I_XOR(R8(E)));
    assert_eq!(decode(0xac), I_XOR(R8(H)));
    assert_eq!(decode(0xad), I_XOR(R8(L)));

    assert_eq!(decode(0xaf), I_XOR(R8(A)));
  }

  #[test]
  fn test_decode_0xbx() {
    assert_eq!(decode(0xb0), I_OR(R8(B)));
    assert_eq!(decode(0xb1), I_OR(R8(C)));
    assert_eq!(decode(0xb2), I_OR(R8(D)));
    assert_eq!(decode(0xb3), I_OR(R8(E)));
    assert_eq!(decode(0xb4), I_OR(R8(H)));
    assert_eq!(decode(0xb5), I_OR(R8(L)));

    assert_eq!(decode(0xb7), I_OR(R8(A)));
    assert_eq!(decode(0xb8), I_CP(R8(B)));
    assert_eq!(decode(0xb9), I_CP(R8(C)));
    assert_eq!(decode(0xba), I_CP(R8(D)));
    assert_eq!(decode(0xbb), I_CP(R8(E)));
    assert_eq!(decode(0xbc), I_CP(R8(H)));
    assert_eq!(decode(0xbd), I_CP(R8(L)));

    assert_eq!(decode(0xbf), I_CP(R8(A)));
  }

  #[test]
  #[should_panic]
  fn decode_unknown() {
    decode(0b11111111);
  }
}
