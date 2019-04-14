use super::registers::Register8;

#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq)]
pub enum Instruction {
  I_NOP,
  I_HALT,
  I_LD_R8_R8(Register8, Register8),
}

use Instruction::*;
use Register8::*;

pub fn decode(opcode: u8) -> Instruction {
  match opcode {
    0x76 => I_HALT,
    0x00 => I_NOP,

    0x40..=0x7F => decode_ld_r8_r8(opcode),
    _ => panic!("Unknown opcode {}", opcode),
  }
}

fn decode_ld_r8_r8(opcode: u8) -> Instruction {
  let reg1 = match opcode & 0xf8 {
    0x40 => B,
    0x48 => C,
    0x50 => D,
    0x58 => E,
    0x60 => H,
    0x68 => L,
    0x78 => A,
    _ => panic!("Can't match reg1 on LD opcode: 0x{:x}", opcode),
  };

  let reg2 = match opcode & 0x07 {
    0x00 => B,
    0x01 => C,
    0x02 => D,
    0x03 => E,
    0x04 => H,
    0x05 => L,
    0x07 => A,
    _ => panic!("Can't match reg2 on LD opcode: 0x{:x}", opcode),
  };

  I_LD_R8_R8(reg1, reg2)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn decode_nop() {
    assert_eq!(decode(0b00000000), I_NOP);
  }

  #[test]
  #[should_panic]
  fn decode_unknown() {
    decode(0b11111111);
  }

  #[test]
  fn test_decode_0x4x() {
    assert_eq!(decode(0x40), I_LD_R8_R8(B, B));
    assert_eq!(decode(0x41), I_LD_R8_R8(B, C));
    assert_eq!(decode(0x42), I_LD_R8_R8(B, D));
    assert_eq!(decode(0x43), I_LD_R8_R8(B, E));
    assert_eq!(decode(0x44), I_LD_R8_R8(B, H));
    assert_eq!(decode(0x45), I_LD_R8_R8(B, L));

    assert_eq!(decode(0x47), I_LD_R8_R8(B, A));
    assert_eq!(decode(0x48), I_LD_R8_R8(C, B));
    assert_eq!(decode(0x49), I_LD_R8_R8(C, C));
    assert_eq!(decode(0x4a), I_LD_R8_R8(C, D));
    assert_eq!(decode(0x4b), I_LD_R8_R8(C, E));
    assert_eq!(decode(0x4c), I_LD_R8_R8(C, H));
    assert_eq!(decode(0x4d), I_LD_R8_R8(C, L));

    assert_eq!(decode(0x4f), I_LD_R8_R8(C, A));
  }

  #[test]
  fn test_decode_0x5x() {
    assert_eq!(decode(0x50), I_LD_R8_R8(D, B));
    assert_eq!(decode(0x51), I_LD_R8_R8(D, C));
    assert_eq!(decode(0x52), I_LD_R8_R8(D, D));
    assert_eq!(decode(0x53), I_LD_R8_R8(D, E));
    assert_eq!(decode(0x54), I_LD_R8_R8(D, H));
    assert_eq!(decode(0x55), I_LD_R8_R8(D, L));

    assert_eq!(decode(0x57), I_LD_R8_R8(D, A));
    assert_eq!(decode(0x58), I_LD_R8_R8(E, B));
    assert_eq!(decode(0x59), I_LD_R8_R8(E, C));
    assert_eq!(decode(0x5a), I_LD_R8_R8(E, D));
    assert_eq!(decode(0x5b), I_LD_R8_R8(E, E));
    assert_eq!(decode(0x5c), I_LD_R8_R8(E, H));
    assert_eq!(decode(0x5d), I_LD_R8_R8(E, L));

    assert_eq!(decode(0x5f), I_LD_R8_R8(E, A));
  }

  #[test]
  fn test_decode_0x6x() {
    assert_eq!(decode(0x60), I_LD_R8_R8(H, B));
    assert_eq!(decode(0x61), I_LD_R8_R8(H, C));
    assert_eq!(decode(0x62), I_LD_R8_R8(H, D));
    assert_eq!(decode(0x63), I_LD_R8_R8(H, E));
    assert_eq!(decode(0x64), I_LD_R8_R8(H, H));
    assert_eq!(decode(0x65), I_LD_R8_R8(H, L));

    assert_eq!(decode(0x67), I_LD_R8_R8(H, A));
    assert_eq!(decode(0x68), I_LD_R8_R8(L, B));
    assert_eq!(decode(0x69), I_LD_R8_R8(L, C));
    assert_eq!(decode(0x6a), I_LD_R8_R8(L, D));
    assert_eq!(decode(0x6b), I_LD_R8_R8(L, E));
    assert_eq!(decode(0x6c), I_LD_R8_R8(L, H));
    assert_eq!(decode(0x6d), I_LD_R8_R8(L, L));

    assert_eq!(decode(0x6f), I_LD_R8_R8(L, A));
  }

  #[test]
  fn test_decode_0x7x() {
    assert_eq!(decode(0x76), I_HALT);

    assert_eq!(decode(0x78), I_LD_R8_R8(A, B));
    assert_eq!(decode(0x79), I_LD_R8_R8(A, C));
    assert_eq!(decode(0x7a), I_LD_R8_R8(A, D));
    assert_eq!(decode(0x7b), I_LD_R8_R8(A, E));
    assert_eq!(decode(0x7c), I_LD_R8_R8(A, H));
    assert_eq!(decode(0x7d), I_LD_R8_R8(A, L));

    assert_eq!(decode(0x7f), I_LD_R8_R8(A, A));
  }
}
