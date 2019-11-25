extern crate rand;
mod registers;
mod renderscan;
mod step;

use crate::buffer::Buffer;
use crate::mmu::MMU;
use renderscan::renderscan;
use std::sync::Arc;
use step::Step;

pub struct GPU {
  step: Step,
  buffer: Arc<Buffer>,
}

impl GPU {
  pub fn new(buffer: Arc<Buffer>) -> GPU {
    GPU {
      step: Step::new(),
      buffer: buffer,
    }
  }

  pub fn step(&mut self, cycles: u8, mmu: &mut dyn MMU) {
    use step::Result::*;

    match self.step.calc(cycles, mmu) {
      Renderscan => renderscan(&self.buffer, mmu),
      Noop => (),
    }
  }
}
