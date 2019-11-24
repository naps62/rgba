use super::MMU;
use std::collections::HashMap;

pub struct TestMMU {
  mem: HashMap<usize, u8>,
}

impl TestMMU {
  pub fn new() -> TestMMU {
    TestMMU {
      mem: HashMap::new(),
    }
  }
}

impl MMU for TestMMU {
  fn read8(&self, index: usize) -> u8 {
    match self.mem.get(&index) {
      Some(&value) => value,
      None => 0,
    }
  }

  fn read16(&self, index: usize) -> u16 {
    ((self.read8(index + 1) as u16) << 8) | (self.read8(index) as u16)
  }

  fn write8(&mut self, index: usize, value: u8) {
    self.mem.insert(index, value);
  }

  fn write16(&mut self, index: usize, value: u16) {
    self.mem.insert(index, (value & 0x00FF) as u8);
    self.mem.insert(index + 1, ((value & 0xFF00) >> 8) as u8);
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn read_and_write_8() {
    let mut mmu = TestMMU::new();
    mmu.write8(0xffff, 1);

    assert_eq!(mmu.read8(0xff00), 0);
    assert_eq!(mmu.read8(0xffff), 1);
  }

  #[test]
  fn read_and_write_16() {
    let mut mmu = TestMMU::new();
    mmu.write16(0x0, 0x1234);

    assert_eq!(mmu.read16(0x0), 0x1234);
    assert_eq!(mmu.read8(0x0), 0x34);
    assert_eq!(mmu.read8(0x1), 0x12);
  }
}
