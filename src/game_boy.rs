use super::{cpu, memory};

#[allow(dead_code)]
pub struct GameBoy {
  cpu: cpu::CPU,
  ram: memory::Memory,
  // video_ram: memory::Memory,
  // cartridge: cartridge::Cartridge,
  // display: display::Display,
}

impl GameBoy {
  #[allow(dead_code)]
  pub fn new() -> GameBoy {
    GameBoy {
      cpu: cpu::CPU::new(),
      ram: memory::Memory::new(8 * 1024),
      // video_ram: memory::Memory::new(8 * 1024),
      // display: display::Display::new(),
      // cartridge: cartridge::Cartridge::new(),
    }
  }
}
