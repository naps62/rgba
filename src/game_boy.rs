use super::{cartridge, cpu, display, mmu};

#[allow(dead_code)]
pub struct GameBoy {
  cpu: cpu::CPU,
  cartridge: cartridge::Cartridge,
  // pub display: display::Display,
  // video_ram: memory::Memory,
  // display: display::Display,
}

impl GameBoy {
  #[allow(dead_code)]
  pub fn new(cartridge_path: &str) -> GameBoy {
    let mmu = mmu::MMU::new();

    GameBoy {
      cpu: cpu::CPU::new(mmu),
      cartridge: cartridge::Cartridge::new(cartridge_path),
      // display: display::Display::new(),
      // video_ram: memory::Memory::new(8 * 1024),
      // display: display::Display::new(),
    }
  }
}
