// note: memory is little-endian
// when reading a 2 byte number, we need to invert the two bytes

pub struct Memory {
  mem: Vec<u8>,
}

impl Memory {
  pub fn new(size: usize) -> Memory {
    Memory { mem: vec![0; size] }
  }

  pub fn read8(&self, index: usize) -> u8 {
    self.mem[index]
  }

  pub fn read16(&self, index: usize) -> u16 {
    ((self.mem[index + 1] as u16) << 8) | (self.mem[index] as u16)
  }

  pub fn write8(&mut self, index: usize, value: u8) {
    self.mem[index] = value;
  }

  pub fn write16(&mut self, index: usize, value: u16) {
    self.mem[index] = (value & 0x00FF) as u8;
    self.mem[index + 1] = ((value & 0xFF00) >> 8) as u8;
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn read8() {
    let mut memory = Memory::new(1);

    memory.mem[0] = 255;

    assert_eq!(memory.read8(0), 255);
  }

  #[test]
  fn write8() {
    let mut memory = Memory::new(1);

    memory.write8(0, 0b1111_1111);

    assert_eq!(memory.mem[0], 0b1111_1111);
  }

  #[test]
  fn read16() {
    let mut memory = Memory::new(2);

    memory.mem[0] = 255;
    memory.mem[1] = 1;

    assert_eq!(memory.read16(0), 511);
  }

  #[test]
  fn write16() {
    let mut memory = Memory::new(2);

    memory.write16(0, 511);

    assert_eq!(memory.mem[0], 255);
    assert_eq!(memory.mem[1], 1);
  }
}
