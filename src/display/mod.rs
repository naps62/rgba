extern crate graphics;
extern crate piston;
extern crate rand;

mod buffer;
mod render_thread;

use crate::input::KeyEvent;
use buffer::Buffer;
use std::sync::Arc;

#[allow(dead_code)]
pub struct Display {
  render_thread: std::thread::JoinHandle<()>,
  buffer: Arc<Buffer>,
}

const WIDTH: f64 = 600.0;
const HEIGHT: f64 = 600.0;

impl Display {
  pub fn new(input_sender: crossbeam_channel::Sender<KeyEvent>) -> Display {
    let buffer = Arc::new(Buffer::from_size(160, 144));

    let buffer_clone = Arc::clone(&buffer);

    std::thread::spawn(move || loop {
      buffer_clone.randomize();
      std::thread::sleep(std::time::Duration::from_millis(100));
    });

    let render = render_thread::spawn(WIDTH, HEIGHT, input_sender, Arc::clone(&buffer));

    Display {
      render_thread: render,
      buffer: buffer,
    }
  }
}
