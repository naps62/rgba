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
  fn read8<I>(&self, idx: I) -> u8
  where
    I: Into<usize>,
  {
    let index: usize = idx.into();

    match self.mem.get(&index.into()) {
      Some(&value) => value,
      None => 0,
    }
  }

  fn read16<I>(&self, idx: I) -> u16
  where
    I: Into<usize>,
  {
    let index: usize = idx.into();

    ((self.read8(index + 1) as u16) << 8) | (self.read8(index) as u16)
  }

  fn write8<I>(&mut self, idx: I, value: u8)
  where
    I: Into<usize>,
  {
    let index: usize = idx.into();

    self.mem.insert(index, value);
  }

  fn write16<I>(&mut self, idx: I, value: u16)
  where
    I: Into<usize>,
  {
    let index: usize = idx.into();

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
    mmu.write8(0xffffu16, 1u8);

    assert_eq!(mmu.read8(0xff00u16), 0u8);
    assert_eq!(mmu.read8(0xffffu16), 1u8);
  }

  #[test]
  fn read_and_write_16() {
    let mut mmu = TestMMU::new();
    mmu.write16(0x0u16, 0x1234);

    assert_eq!(mmu.read16(0x0u16), 0x1234);
    assert_eq!(mmu.read8(0x0u16), 0x34);
    assert_eq!(mmu.read8(0x1u16), 0x12);
  }
}
