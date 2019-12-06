#[derive(Debug)]
pub enum Addr {
  LCDControl = 0xFF40,
  ScrollY = 0xFF42,
  ScrollX = 0xFF43,
  CurrentScanLine = 0xFF44,
  BGPalette = 0xFF47,
}

pub enum LCDControlReg {
  BackgroundEnabled = 0b0000_0001,
  SpritesEnabled = 0b0000_0010,
  SpriteSize = 0b0000_0100,
  BGTileMap = 0b0000_1000,
  BGTileSet = 0b0001_0000,
  WindowEnabled = 0b0010_0000,
  WindowTileMap = 0b0100_0000,
  LCDEnabled = 0b1000_0000,
}

impl From<Addr> for usize {
  fn from(addr: Addr) -> Self {
    addr as usize
  }
}

impl From<LCDControlReg> for u8 {
  fn from(addr: LCDControlReg) -> Self {
    addr as u8
  }
}
