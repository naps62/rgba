#[macro_use]
extern crate clap;
extern crate crossbeam_channel;
extern crate glium;

pub mod cpu;
mod debug;
mod display;
mod game_boy;
mod gpu;
mod input;
mod mmu;

fn main() {
  let yaml = load_yaml!("../assets/cli.yml");
  let matches = clap::App::from_yaml(yaml).get_matches();

  debug::set_verbosity_level(matches.occurrences_of("verbosity") as u8);

  let cartridge_path = matches.value_of("cartridge").unwrap();
  let mut game_boy = game_boy::GameBoy::new(cartridge_path);

  game_boy.run();
}
