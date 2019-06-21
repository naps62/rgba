use std::time::Duration;

mod cartridge;
pub mod cpu;
mod display;
mod game_boy;
mod mmu;

fn main() {
  let game_boy = game_boy::GameBoy::new();

  std::thread::sleep(Duration::new(10, 0));
}
