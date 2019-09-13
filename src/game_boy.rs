extern crate crossbeam_channel;

use super::{cartridge, cpu, display, gpu, input, mmu};

#[allow(dead_code)]
pub struct GameBoy {
  mmu: mmu::MMU,
  cpu: cpu::CPU,
  cartridge: cartridge::Cartridge,
  gpu: gpu::GPU,
  display: display::Display,
  input: input::Input,
  // video_ram: memory::Memory,
}

impl GameBoy {
  #[allow(dead_code)]
  pub fn new(cartridge_path: &str) -> GameBoy {
    let (input_sender, input_receiver) = crossbeam_channel::unbounded();

    let gameboy = GameBoy {
      mmu: mmu::MMU::new(true),
      cpu: cpu::CPU::new(),
      cartridge: cartridge::Cartridge::new(cartridge_path),
      gpu: gpu::GPU::new(),
      display: display::Display::new(input_sender),
      input: input::Input::new(input_receiver),
      // video_ram: memory::Memory::new(8 * 1024),
    };

    std::thread::sleep(std::time::Duration::new(10, 0));

    gameboy
  }
}
