use std::time::Duration;

mod cartridge;
pub mod cpu;
mod display;
mod game_boy;
mod mmu;

fn main() {
  let mut game_boy = game_boy::GameBoy::new();

  game_boy.display.show();
  game_boy.display.draw();

  std::thread::sleep(Duration::new(10, 0));
}
