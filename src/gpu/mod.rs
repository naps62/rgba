#![feature(associated_type_bounds)]

extern crate rand;
pub mod renderer;
mod step;

use crate::buffer::Buffer;
use crate::mmu::MMU;
use std::sync::Arc;
use step::Step;

pub struct GPU<M, R> {
  step: Step,
  buffer: Arc<Buffer>,
  renderer: R,
  mmu: std::marker::PhantomData<M>,
}

impl<M: MMU, R: renderer::Interface<M>> GPU<M, R> {
  pub fn new(buffer: Arc<Buffer>, renderer: R) -> GPU<M, R> {
    GPU {
      step: Step::new(),
      buffer: buffer,
      renderer: renderer,
      mmu: std::marker::PhantomData,
    }
  }

  pub fn step(&mut self, cycles: u8, mmu: &mut M) {
    use step::Result::*;

    match self.step.calc(cycles, mmu) {
      Renderscan => (),
      Noop => (),
    }
  }
}
