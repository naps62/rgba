type Reg8x2 = (u8, u8);
type Reg16 = u16;

pub enum Flag {
  ZF = 7, // zero flag
  NF = 6, // add/sub flag
  HF = 5, // half-carry flag
  CF = 4, // carry flag
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Register8 {
  A,
  B,
  C,
  D,
  E,
  H,
  L,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Register16 {
  BC,
  DE,
  HL,
  SP,
  PC,
  AF,
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
      A => self.a(),
      B => self.b(),
      C => self.c(),
      D => self.d(),
      E => self.e(),
      H => self.h(),
      L => self.l(),
    }
  }

  pub fn a(&self) -> u8 {
    self.AF.0
  }
  pub fn b(&self) -> u8 {
    self.BC.0
  }
  pub fn c(&self) -> u8 {
    self.BC.1
  }
  pub fn d(&self) -> u8 {
    self.DE.0
  }
  pub fn e(&self) -> u8 {
    self.DE.1
  }
  pub fn h(&self) -> u8 {
    self.HL.0
  }
  pub fn l(&self) -> u8 {
    self.HL.1
  }

  pub fn read16(&self, reg: Register16) -> u16 {
    use Register16::*;

    match reg {
      BC => self.bc(),
      DE => self.de(),
      HL => self.hl(),
      SP => self.sp(),
      PC => self.pc(),
      AF => self.af(),
    }
  }

  pub fn af(&self) -> u16 {
    reg8x2_to_reg16(self.AF)
  }
  pub fn bc(&self) -> u16 {
    reg8x2_to_reg16(self.BC)
  }
  pub fn de(&self) -> u16 {
    reg8x2_to_reg16(self.DE)
  }
  pub fn hl(&self) -> u16 {
    reg8x2_to_reg16(self.HL)
  }
  pub fn sp(&self) -> u16 {
    self.SP
  }
  pub fn pc(&self) -> u16 {
    self.PC
  }

  pub fn write8(&mut self, reg: Register8, value: u8) {
    use Register8::*;

    match reg {
      A => self.set_a(value),
      B => self.set_b(value),
      C => self.set_c(value),
      D => self.set_d(value),
      E => self.set_e(value),
      H => self.set_h(value),
      L => self.set_l(value),
    };
  }

  pub fn set_a(&mut self, value: u8) {
    self.AF.0 = value
  }
  pub fn set_b(&mut self, value: u8) {
    self.BC.0 = value
  }
  pub fn set_c(&mut self, value: u8) {
    self.BC.1 = value
  }
  pub fn set_d(&mut self, value: u8) {
    self.DE.0 = value
  }
  pub fn set_e(&mut self, value: u8) {
    self.DE.1 = value
  }
  pub fn set_h(&mut self, value: u8) {
    self.HL.0 = value
  }
  pub fn set_l(&mut self, value: u8) {
    self.HL.1 = value
  }

  pub fn write16(&mut self, reg: Register16, value: u16) {
    use Register16::*;

    match reg {
      BC => self.set_bc(value),
      DE => self.set_de(value),
      HL => self.set_hl(value),
      SP => self.set_sp(value),
      PC => self.set_pc(value),
      AF => self.set_af(value),
    }
  }

  pub fn set_af(&mut self, v: u16) {
    self.AF = reg16_to_reg8x2(v)
  }
  pub fn set_bc(&mut self, v: u16) {
    self.BC = reg16_to_reg8x2(v)
  }
  pub fn set_de(&mut self, v: u16) {
    self.DE = reg16_to_reg8x2(v)
  }
  pub fn set_hl(&mut self, v: u16) {
    self.HL = reg16_to_reg8x2(v)
  }
  pub fn set_sp(&mut self, v: u16) {
    self.SP = v
  }
  pub fn set_pc(&mut self, v: u16) {
    self.PC = v
  }

  pub fn get_flag(&self, flag: Flag) -> bool {
    let flag_index = flag as u8;
    let mask = (1 as u8) << flag_index;

    ((self.AF.1 & mask) >> flag_index) == 1
  }

  pub fn set_flag(&mut self, flag: Flag, value: bool) {
    if value {
      let mask = (1 as u8) << (flag as u8);

      self.AF.1 = self.AF.1 | mask;
    } else {
      let mask = !((1 as u8) << (flag as u8));

      self.AF.1 = self.AF.1 & mask;
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

    assert_eq!(registers.read8(B), 0);
    assert_eq!(registers.read8(C), 0);
    assert_eq!(registers.read8(D), 0);
    assert_eq!(registers.read8(E), 0);
    assert_eq!(registers.read8(H), 0);
    assert_eq!(registers.read8(L), 0);
    assert_eq!(registers.read8(A), 0);
  }

  #[test]
  fn read_16_bit_registers() {
    let registers = Registers::new();

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

    registers.write16(BC, 65_534);
    registers.write16(DE, 65_533);
    registers.write16(HL, 65_532);
    registers.write16(SP, 65_531);
    registers.write16(PC, 65_530);

    println!("{:?}", registers);

    assert_eq!(registers.read16(BC), 65_534);
    assert_eq!(registers.read16(DE), 65_533);
    assert_eq!(registers.read16(HL), 65_532);
    assert_eq!(registers.read16(SP), 65_531);
    assert_eq!(registers.read16(PC), 65_530);
  }

  #[test]
  fn zf_flag() {
    let mut registers = Registers::new();

    assert_eq!(registers.get_flag(ZF), false);
    registers.set_flag(ZF, true);
    assert_eq!(registers.get_flag(ZF), true);
    registers.set_flag(ZF, false);
    assert_eq!(registers.get_flag(ZF), false);
  }

  #[test]
  fn nf_flag() {
    let mut registers = Registers::new();

    assert_eq!(registers.get_flag(NF), false);
    registers.set_flag(NF, true);
    assert_eq!(registers.get_flag(NF), true);
    registers.set_flag(NF, false);
    assert_eq!(registers.get_flag(NF), false);
  }

  #[test]
  fn hf_flag() {
    let mut registers = Registers::new();

    assert_eq!(registers.get_flag(HF), false);
    registers.set_flag(HF, true);
    assert_eq!(registers.get_flag(HF), true);
    registers.set_flag(HF, false);
    assert_eq!(registers.get_flag(HF), false);
  }
  #[test]
  fn cf_flag() {
    let mut registers = Registers::new();
    assert_eq!(registers.get_flag(CF), false);
    registers.set_flag(CF, true);
    assert_eq!(registers.get_flag(CF), true);
    registers.set_flag(CF, false);
    assert_eq!(registers.get_flag(CF), false);
  }
}
