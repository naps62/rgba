extern crate image;
extern crate rand;

use image::{ImageBuffer, Rgba, RgbaImage};
use std::sync::{Mutex, MutexGuard};

pub struct Buffer {
  data: Mutex<RgbaImage>,
}

impl Buffer {
  pub fn get(&self) -> MutexGuard<RgbaImage> {
    self.data.lock().unwrap()
  }

  pub fn from_data(data: RgbaImage) -> Buffer {
    Buffer {
      data: Mutex::new(data),
    }
  }

  pub fn from_size(w: u32, h: u32) -> Buffer {
    let buf = (0..w * h * 4).map(|_| 127).collect();

    let data = ImageBuffer::from_vec(w, h, buf).unwrap();

    Buffer::from_data(data)
  }

  pub fn randomize(&self) {
    let mut rng = rand::thread_rng();

    let mut data = self.get();

    for p in data.pixels_mut() {
      *p = random_pixel(&mut rng);
    }
  }

  #[allow(dead_code)]
  pub fn reset(&self) {
    let mut data = self.get();

    for p in data.pixels_mut() {
      *p = new_pixel();
    }
  }
}

fn new_pixel() -> Rgba<u8> {
  Rgba::<u8>([0, 0, 0, 0])
}

fn random_pixel<T>(rng: &mut T) -> Rgba<u8>
where
  T: rand::Rng,
{
  Rgba::<u8>([rng.gen(), rng.gen(), rng.gen(), rng.gen()])
}
