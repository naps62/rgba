pub trait Interface {
  fn read8(&self, index: usize) -> u8;
  fn read16(&self, index: usize) -> u16;

  fn write8(&mut self, index: usize, value: u8);
  fn write16(&mut self, index: usize, value: u16);
}
