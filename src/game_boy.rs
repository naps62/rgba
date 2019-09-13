use super::{cartridge, cpu, display, gpu, mmu};

#[allow(dead_code)]
pub struct GameBoy {
  mmu: mmu::MMU,
  cpu: cpu::CPU,
  cartridge: cartridge::Cartridge,
  gpu: gpu::GPU,
  display: display::Display,
  // pub display: display::Display,
  // video_ram: memory::Memory,
}

impl GameBoy {
  #[allow(dead_code)]
  pub fn new(cartridge_path: &str) -> GameBoy {
    GameBoy {
      mmu: mmu::MMU::new(true),
      cpu: cpu::CPU::new(),
      cartridge: cartridge::Cartridge::new(cartridge_path),
      gpu: gpu::GPU::new(),
      display: display::Display::new(),
      // video_ram: memory::Memory::new(8 * 1024),
    }
  }
}
