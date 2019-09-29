extern crate rand;
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

  pub fn step(&mut self, mmu: &mut MMU, cycles: u8) {
    use step::Result::*;

    match self.step.calc(cycles) {
      Renderscan => renderscan(&self.step, mmu, &self.buffer),
      Noop => (),
    }
  }
}
