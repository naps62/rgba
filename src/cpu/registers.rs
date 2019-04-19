type Reg8x2 = (u8, u8);
type Reg16 = u16;

pub enum Flag {
  ZF = 7, // zero flag
  NF = 6, // add/sub flag
  HF = 5, // half-carry flag
  CF = 4, // carry flag
}

#[derive(Debug, PartialEq)]
pub enum Register8 {
  A,
  B,
  C,
  D,
  E,
  H,
  L,
}

#[derive(Debug, PartialEq)]
pub enum Register16 {
  AF,
  BC,
  DE,
  HL,
  SP,
  PC,
}

#[derive(Debug, PartialEq)]
pub enum RegisterAny {
  A,
  B,
  C,
  D,
  E,
  H,
  L,
  AF,
  BC,
  DE,
  HL,
  SP,
  PC,
}

#[derive(Debug)]
#[allow(non_snake_case)]
pub struct Registers {
  AF: Reg8x2,
  BC: Reg8x2,
  DE: Reg8x2,
  HL: Reg8x2,
  SP: Reg16,
  PC: Reg16,
}

impl Registers {
  pub fn new() -> Registers {
    Registers {
      AF: (0, 0),
      BC: (0, 0),
      DE: (0, 0),
      HL: (0, 0),
      SP: 0,
      PC: 0,
    }
  }

  pub fn read8(&self, reg: Register8) -> u8 {
    use Register8::*;

    match reg {
      A => self.AF.0,
      B => self.BC.0,
      C => self.BC.1,
      D => self.DE.0,
      E => self.DE.1,
      H => self.HL.0,
      L => self.HL.1,
    }
  }

  pub fn read16(&self, reg: Register16) -> u16 {
    use Register16::*;

    match reg {
      AF => reg8x2_to_reg16(self.AF),
      BC => reg8x2_to_reg16(self.BC),
      DE => reg8x2_to_reg16(self.DE),
      HL => reg8x2_to_reg16(self.HL),
      SP => self.SP,
      PC => self.PC,
    }
  }

  pub fn write8(&mut self, reg: Register8, value: u8) {
    use Register8::*;

    match reg {
      A => self.AF.0 = value,
      B => self.BC.0 = value,
      C => self.BC.1 = value,
      D => self.DE.0 = value,
      E => self.DE.1 = value,
      H => self.HL.0 = value,
      L => self.HL.1 = value,
    };
  }

  pub fn write16(&mut self, reg: Register16, value: u16) {
    use Register16::*;

    match reg {
      AF => self.AF = reg16_to_reg8x2(value),
      BC => self.BC = reg16_to_reg8x2(value),
      DE => self.DE = reg16_to_reg8x2(value),
      HL => self.HL = reg16_to_reg8x2(value),
      SP => self.SP = value,
      PC => self.PC = value,
    }
  }

  pub fn get_flag(&self, flag: Flag) -> u8 {
    let flag_index = flag as u8;
    let mask = (1 as u8) << flag_index;

    (self.AF.1 & mask) >> flag_index
  }

  pub fn set_flag(&mut self, flag: Flag, value: u8) {
    if value == 0 {
      let mask = !((1 as u8) << (flag as u8));

      self.AF.1 = self.AF.1 & mask;
    } else {
      let mask = (1 as u8) << (flag as u8);

      self.AF.1 = self.AF.1 | mask;
    }
  }
}

fn reg8x2_to_reg16(reg: Reg8x2) -> Reg16 {
  (reg.1 as u16) | ((reg.0 as u16) << 8)
}

fn reg16_to_reg8x2(reg: Reg16) -> Reg8x2 {
  (((reg & 0xff00) >> 8) as u8, ((reg & 0x00ff) as u8))
}

#[cfg(test)]
mod tests {
  use super::*;
  use Flag::*;
  use Register16::*;
  use Register8::*;

  #[test]
  fn initialize() {
    use Register16::*;

    let registers = Registers::new();

    assert_eq!(registers.AF.0, 0);
    assert_eq!(registers.AF.1, 0);
    assert_eq!(registers.BC.0, 0);
    assert_eq!(registers.BC.1, 0);
    assert_eq!(registers.DE.0, 0);
    assert_eq!(registers.DE.1, 0);
    assert_eq!(registers.HL.0, 0);
    assert_eq!(registers.HL.1, 0);
    assert_eq!(registers.SP, 0);
    assert_eq!(registers.PC, 0);
  }

  #[test]
  fn read_8_bit_registers() {
    let registers = Registers::new();
  }

  #[test]
  fn read_16_bit_registers() {
    let registers = Registers::new();

    assert_eq!(registers.read16(AF), 0);
    assert_eq!(registers.read16(BC), 0);
    assert_eq!(registers.read16(DE), 0);
    assert_eq!(registers.read16(HL), 0);
    assert_eq!(registers.read16(SP), 0);
    assert_eq!(registers.read16(PC), 0);
  }

  #[test]
  fn registers_8() {
    let mut registers = Registers::new();

    registers.write8(A, 255);
    registers.write8(B, 254);
    registers.write8(C, 253);
    registers.write8(D, 252);
    registers.write8(E, 251);
    registers.write8(H, 250);
    registers.write8(L, 249);

    assert_eq!(registers.read8(A), 255);
    assert_eq!(registers.read8(B), 254);
    assert_eq!(registers.read8(C), 253);
    assert_eq!(registers.read8(D), 252);
    assert_eq!(registers.read8(E), 251);
    assert_eq!(registers.read8(H), 250);
    assert_eq!(registers.read8(L), 249);
  }

  #[test]
  fn registers_16() {
    let mut registers = Registers::new();

    assert_eq!(registers.read8(A), 0);
    assert_eq!(registers.read8(B), 0);
    assert_eq!(registers.read8(C), 0);
    assert_eq!(registers.read8(D), 0);
    assert_eq!(registers.read8(E), 0);
    assert_eq!(registers.read8(H), 0);
    assert_eq!(registers.read8(L), 0);

    registers.write16(AF, 65_535);
    registers.write16(BC, 65_534);
    registers.write16(DE, 65_533);
    registers.write16(HL, 65_532);
    registers.write16(SP, 65_531);
    registers.write16(PC, 65_530);

    println!("{:?}", registers);

    assert_eq!(registers.read16(AF), 65_535);
    assert_eq!(registers.read16(BC), 65_534);
    assert_eq!(registers.read16(DE), 65_533);
    assert_eq!(registers.read16(HL), 65_532);
    assert_eq!(registers.read16(SP), 65_531);
    assert_eq!(registers.read16(PC), 65_530);
  }

  #[test]
  fn zf_flag() {
    let mut registers = Registers::new();

    assert_eq!(registers.get_flag(ZF), 0);
    registers.set_flag(ZF, 1);
    assert_eq!(registers.get_flag(ZF), 1);
    assert_eq!(registers.read16(AF), 128);
    registers.set_flag(ZF, 0);
    assert_eq!(registers.get_flag(ZF), 0);
  }

  #[test]
  fn nf_flag() {
    let mut registers = Registers::new();

    assert_eq!(registers.get_flag(NF), 0);
    registers.set_flag(NF, 1);
    assert_eq!(registers.get_flag(NF), 1);
    assert_eq!(registers.read16(AF), 64);
    registers.set_flag(NF, 0);
    assert_eq!(registers.get_flag(NF), 0);
  }

  fn hf_flag() {
    let mut registers = Registers::new();

    assert_eq!(registers.get_flag(HF), 0);
    registers.set_flag(HF, 1);
    assert_eq!(registers.get_flag(HF), 1);
    assert_eq!(registers.read16(AF), 32);
    registers.set_flag(HF, 0);
    assert_eq!(registers.get_flag(HF), 0);
  }
  #[test]
  fn cf_flag() {
    let mut registers = Registers::new();
    assert_eq!(registers.get_flag(CF), 0);
    registers.set_flag(CF, 1);
    assert_eq!(registers.get_flag(CF), 1);
    assert_eq!(registers.read16(AF), 16);
    registers.set_flag(CF, 0);
    assert_eq!(registers.get_flag(CF), 0);
  }
}
