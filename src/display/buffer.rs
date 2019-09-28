extern crate rand;

use std::sync::{Mutex, MutexGuard};

pub type Pixel = (f32, f32, f32);
pub type Data = Vec<Vec<Pixel>>;

pub struct Buffer {
  data: Mutex<Data>,
}

impl Buffer {
  pub fn get(&self) -> MutexGuard<Data> {
    self.data.lock().unwrap()
  }

  pub fn from_data(data: Data) -> Buffer {
    Buffer {
      data: Mutex::new(data),
    }
  }

  pub fn from_size(w: i32, h: i32) -> Buffer {
    let data = (0..w)
      .map(|_| (0..h).map(|_| (1.0, 1.0, 1.0)).collect())
      .collect();

    Buffer::from_data(data)
  }

  pub fn from_random(w: i32, h: i32) -> Buffer {
    let buffer = Buffer::from_size(w, h);

    buffer.randomize();

    buffer
  }

  pub fn randomize(&self) {
    let mut rng = rand::thread_rng();

    let mut data = self.get();

    for i in 0..data.len() {
      for j in 0..data[0].len() {
        data[i][j] = random_pixel(&mut rng);
      }
    }
  }
}

fn random_pixel<T>(rng: &mut T) -> Pixel
where
  T: rand::Rng,
{
  // let x: f32 = rng.gen();
  rng.gen()

  //   (x, x, x)
}
