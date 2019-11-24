extern crate crossbeam_channel;

use super::mmu::real_mmu::RealMMU;
use super::{buffer::Buffer, cpu::CPU, display::Display, gpu::GPU, input::Input};
use std::sync::Arc;

#[allow(dead_code)]
pub struct GameBoy {
  cpu: CPU,
  gpu: GPU,
  mmu: RealMMU,
  display: Display,
  input: Input,
}

impl GameBoy {
  #[allow(dead_code)]
  pub fn new(cartridge_path: &str) -> GameBoy {
    let (input_sender, input_receiver) = crossbeam_channel::unbounded();

    let cartridge = std::fs::read(cartridge_path).unwrap();
    let buffer = Arc::new(Buffer::from_size(160, 144));
    let mmu = RealMMU::new(true, cartridge);

    let cpu = CPU::new();
    let gpu = GPU::new(Arc::clone(&buffer));
    let display = Display::new(input_sender, Arc::clone(&buffer));
    let input = Input::new(input_receiver);

    GameBoy {
      cpu,
      mmu,
      gpu,
      input,
      display,
    }
  }

  pub fn run(&mut self) {
    loop {
      self.cpu.exec(&mut self.mmu);
      self.gpu.step(self.cpu.last_instr_cycles, &mut self.mmu);
    }
  }
}
