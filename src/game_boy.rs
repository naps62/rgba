use super::{cpu, mmu};

#[allow(dead_code)]
pub struct GameBoy {
  cpu: cpu::CPU,
  mmu: mmu::MMU,
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
      // video_ram: memory::Memory::new(8 * 1024),
      // display: display::Display::new(),
      // cartridge: cartridge::Cartridge::new(),
    }
  }
}
