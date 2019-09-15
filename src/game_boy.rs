extern crate crossbeam_channel;

use std::cell::RefCell;
use std::rc::Rc;

use super::{cpu, display, gpu, input, mmu};

#[allow(dead_code)]
pub struct GameBoy {
  mmu: mmu::MMU,
  cpu: cpu::CPU,
  gpu: Rc<RefCell<gpu::GPU>>,
  display: display::Display,
  input: input::Input,
  // video_ram: memory::Memory,
}

impl GameBoy {
  #[allow(dead_code)]
  pub fn new(cartridge_path: &str) -> GameBoy {
    let (input_sender, input_receiver) = crossbeam_channel::unbounded();

    let gpu = Rc::new(RefCell::new(gpu::GPU::new()));
    let cartridge = std::fs::read(cartridge_path).unwrap();

    GameBoy {
      cpu: cpu::CPU::new(),
      mmu: mmu::MMU::new(true, gpu.clone(), cartridge),
      gpu: gpu,
      display: display::Display::new(input_sender),
      input: input::Input::new(input_receiver),
    }
  }

  pub fn run(&mut self) {
    loop {
      self.cpu.exec(&mut self.mmu);
    }
  }
}
