use super::{cartridge, cpu, display, mmu};

#[allow(dead_code)]
pub struct GameBoy {
  cpu: cpu::CPU,
  mmu: mmu::MMU,
  // pub display: display::Display,
  cartridge: cartridge::Cartridge,
  // video_ram: memory::Memory,
  // display: display::Display,
}

impl GameBoy {
  #[allow(dead_code)]
  pub fn new(cartridge_path: &str) -> GameBoy {
    GameBoy {
      cpu: cpu::CPU::new(),
      mmu: mmu::MMU::new(),
      // display: display::Display::new(),
      cartridge: cartridge::Cartridge::new(cartridge_path),
      // video_ram: memory::Memory::new(8 * 1024),
      // display: display::Display::new(),
    }
  }
}
