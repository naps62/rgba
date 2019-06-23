use std::time::Duration;

#[macro_use]
extern crate clap;

use clap::App;

mod cartridge;
pub mod cpu;
mod debug;
mod display;
mod game_boy;
mod mmu;

fn main() {
  let yaml = load_yaml!("../assets/cli.yml");
  let matches = App::from_yaml(yaml).get_matches();

  debug::set_verbosity_level(matches.occurrences_of("verbosity") as u8);

  let cartridge_path = matches.value_of("cartridge").unwrap();

  let game_boy = game_boy::GameBoy::new(cartridge_path);

  // std::thread::sleep(Duration::new(10, 0));
}
