use std::time::Duration;

#[macro_use]
extern crate clap;
extern crate crossbeam_channel;
extern crate glium;

mod cartridge;
pub mod cpu;
mod debug;
mod display;
mod game_boy;
mod gpu;
mod input;
mod mmu;

use crossbeam_channel::unbounded;

fn x() -> (
  crossbeam_channel::Sender<String>,
  crossbeam_channel::Receiver<String>,
) {
  unbounded()
}

fn main() {
  let yaml = load_yaml!("../assets/cli.yml");
  let matches = clap::App::from_yaml(yaml).get_matches();

  debug::set_verbosity_level(matches.occurrences_of("verbosity") as u8);

  let cartridge_path = matches.value_of("cartridge").unwrap();

  let _game_boy = game_boy::GameBoy::new(cartridge_path);

  std::thread::sleep(Duration::new(10, 0));
  // let (s, r) = unbounded();
  // s.send("Hello");
  // println!("{:?}", r.recv());

  // s.send("Hello2");
  // println!("{:?}", r.recv());
  // std::thread::sleep(Duration::new(10, 0));

  // s.send("Hello3");
  // println!("{:?}", r.recv());
}
