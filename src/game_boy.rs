use super::{cartridge, cpu, display, memory};

pub struct GameBoy {
  cpu: cpu::CPU,
  ram: memory::Memory,
  video_ram: memory::Memory,
  cartridge: cartridge::Cartridge,
  display: display::Display,
}

impl GameBoy {
  pub fn new() -> GameBoy {
    GameBoy {
      cpu: cpu::CPU::new(),
      ram: memory::Memory::new(8 * 1024),
      video_ram: memory::Memory::new(8 * 1024),
      display: display::Display::new(),
      cartridge: cartridge::Cartridge::new(),
    }
  }
}
