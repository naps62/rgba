use crate::mmu::MMU;

pub enum Reg {
  LCDControl = 0xFF40,
  ScrollY = 0xFF42,
  ScrollX = 0xFF43,
  CurrentScanLine = 0xFF44,
  BGPalette = 0xFF47,
}

// LCD Control (0xFF40) reg:
// 0x01 -> switchBG
// 0x08 -> BGMap
// 0x10 -> BGTile
// 0x80 -> SwitchLCD

impl std::convert::From<Reg> for usize {
  fn from(reg: Reg) -> usize {
    reg as usize
  }
}

pub fn read(mmu: &dyn MMU, reg: Reg) -> u8 {
  mmu.read8(reg as usize)
}

pub fn write(mmu: &mut dyn MMU, reg: Reg, value: u8) {
  mmu.write8(reg as usize, value);
}
