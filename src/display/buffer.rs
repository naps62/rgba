extern crate rand;

use std::sync::{Arc, Mutex, MutexGuard};

pub type Pixel = (f32, f32, f32);
pub type Data = Vec<Vec<Pixel>>;

pub struct Buffer {
  data: Arc<Mutex<Data>>,
}

impl Buffer {
  pub fn get(&self) -> Data {
    self.data.lock().unwrap().clone()
  }

  pub fn from_data(data: Data) -> Buffer {
    Buffer {
      data: Arc::new(Mutex::new(data)),
    }
  }

  pub fn from_size(w: i32, h: i32) -> Buffer {
    let data = (0..w)
      .map(|_| (0..h).map(|_| (1.0, 1.0, 1.0)).collect())
      .collect();

    Buffer::from_data(data)
  }

  pub fn randomize(w: i32, h: i32) -> Buffer {
    let mut rng = rand::thread_rng();

    let data = (0..w)
      .map(|_| (0..h).map(|_| random_pixel(&mut rng)).collect())
      .collect();

    Buffer::from_data(data)
  }
}

fn random_pixel<T>(rng: &mut T) -> Pixel
where
  T: rand::Rng,
{
  let x: f32 = rng.gen();

  (x, x, x)
}
