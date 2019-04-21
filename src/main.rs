mod cartridge;
mod cpu;
mod display;
mod game_boy;
mod memory;

use game_boy::GameBoy;

fn main() {
  let game_boy = GameBoy::new();

  println!("Hello")
}
