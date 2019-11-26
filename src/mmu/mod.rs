pub mod addrs;
mod interface;
pub mod real_mmu;

#[cfg(test)]
pub mod test_mmu;

use std::convert::Into;

pub trait MMU {
  fn read8<I>(&self, index: I) -> u8
  where
    I: Into<usize>,
    Self: Sized;

  fn read16<I>(&self, index: I) -> u16
  where
    I: Into<usize>,
    Self: Sized;

  fn write8<I>(&mut self, index: I, value: u8)
  where
    I: Into<usize>,
    Self: Sized;

  fn write16<I>(&mut self, index: I, value: u16)
  where
    I: Into<usize>,
    Self: Sized;
}
