use std::sync::Arc;

use super::Interface;
use crate::buffer::Buffer;
use crate::mmu::MMU;

pub struct BGMapRenderer {}

impl BGMapRenderer {
  pub fn new() -> BGMapRenderer {
    BGMapRenderer {}
  }
}

impl<M: MMU> Interface<M> for BGMapRenderer {
  fn render(&self, buffer: &Arc<Buffer>, mmu: &mut M) {}
}
