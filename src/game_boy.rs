use super::{cpu, display, mmu};

#[allow(dead_code)]
pub struct GameBoy {
  cpu: cpu::CPU,
  mmu: mmu::MMU,
  pub display: display::Display,
  // video_ram: memory::Memory,
  // cartridge: cartridge::Cartridge,
  // display: display::Display,
}

impl GameBoy {
  #[allow(dead_code)]
  pub fn new() -> GameBoy {
    GameBoy {
      cpu: cpu::CPU::new(),
      mmu: mmu::MMU::new(),
      display: display::Display::new(),
      // video_ram: memory::Memory::new(8 * 1024),
      // display: display::Display::new(),
      // cartridge: cartridge::Cartridge::new(),
    }
  }
}
