pub struct Memory {
  mem: Vec<u8>,
}

impl Memory {
  pub fn new(size: usize) -> Memory {
    Memory { mem: vec![0; size] }
  }

  pub fn read8(&self, index: u8) -> u8 {
    // 0
    // self.mem[index]
  }

  pub fn write8(&mut self, index: u8, value: u8) {
    // self.mem[index] = value;
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

    memory.write8(0, 255);

    assert_eq!(memory.mem[0], 255);
  }
}
