pub mod bg_map_renderer;
pub mod renderscan;

use std::sync::Arc;

use crate::buffer::Buffer;
use crate::mmu::MMU;

pub trait Interface<M: MMU>: Sized {
  fn render(&self, buffer: &Arc<Buffer>, mmu: &mut M)
  where
    Self: Sized;
}
