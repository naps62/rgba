pub mod addrs;
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

  fn set_flag<I, U>(&mut self, address: I, mask: U)
  where
    I: Into<usize>,
    U: Into<u8>;

  fn unset_flag<I, U>(&mut self, address: I, mask: U)
  where
    I: Into<usize>,
    U: Into<u8>;

  fn get_flag<I, U>(&self, address: I, mask: U) -> bool
  where
    I: Into<usize>,
    U: Into<u8>;
}
