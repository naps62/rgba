extern crate crossbeam_channel;

use super::{buffer::Buffer, cpu, display, gpu, input, mmu};
use std::sync::Arc;

#[allow(dead_code)]
pub struct GameBoy {
  mmu: mmu::MMU,
  cpu: cpu::CPU,
  gpu: gpu::GPU,
  display: display::Display,
  input: input::Input,
}

impl GameBoy {
  #[allow(dead_code)]
  pub fn new(cartridge_path: &str) -> GameBoy {
    let (input_sender, input_receiver) = crossbeam_channel::unbounded();

    let cartridge = std::fs::read(cartridge_path).unwrap();
    let buffer = Arc::new(Buffer::from_size(160, 144));

    GameBoy {
      cpu: cpu::CPU::new(),
      mmu: mmu::MMU::new(true, cartridge),
      gpu: gpu::GPU::new(Arc::clone(&buffer)),
      display: display::Display::new(input_sender, Arc::clone(&buffer)),
      input: input::Input::new(input_receiver),
    }
  }

  pub fn run(&mut self) {
    loop {
      self.cpu.exec(&mut self.mmu);
      self.gpu.step(&mut self.mmu, self.cpu.last_instr_cycles);
    }
  }
}
