use std::fs;

#[macro_use]
use super::debug;

pub struct Cartridge {
  data: Vec<u8>,
}

impl Cartridge {
  pub fn new(path: &str) -> Cartridge {
    let data = fs::read(path).unwrap();

    // debug!(1, "Loading cartridge {}", path);
    // debug!(1, "Lenght: {}", data.len());

    Cartridge { data: data }
  }
}
