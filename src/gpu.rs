pub struct GPU {}

// use super::mmu::MMU;

impl GPU {
  pub fn new() -> GPU {
    GPU {}
  }

  pub fn read8(&self, index: usize) -> u8 {
    1
  }

  pub fn write8(&mut self, index: usize, value: u8) {}
}
