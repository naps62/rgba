#[macro_use]
extern crate clap;
extern crate crossbeam_channel;

mod buffer;
pub mod cpu;
mod display;
mod game_boy;
mod gpu;
mod input;
mod mmu;

fn main() {
  let yaml = load_yaml!("../assets/cli.yml");
  let matches = clap::App::from_yaml(yaml).get_matches();

  let cartridge_path = matches.value_of("cartridge").unwrap();
  let mut game_boy = game_boy::GameBoy::new(cartridge_path);

  game_boy.run();
}
