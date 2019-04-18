pub struct GameBoy {
  cpu: cpu::CPU,
  ram: ram::RAM,
  cartridge: cartrige::Cartridge,
  display: display::Display,
}

impl GameBoy {
  pub fn new() -> GameBoy {
    GameBoy { cpu: CPU::new() }
  }
}
