pub struct GPU {}

// use super::mmu::MMU;

impl GPU {
  pub fn new() -> GPU {
    GPU {}
  }

  pub fn read8(&self, _index: usize) -> u8 {
    1
  }

  pub fn write8(&mut self, _index: usize, _value: u8) {}
}
