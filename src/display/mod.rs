extern crate graphics;
extern crate piston;
extern crate rand;

mod render_thread;

use super::buffer::Buffer;
use crate::input::KeyEvent;
use std::sync::Arc;

#[allow(dead_code)]
pub struct Display {
  render_thread: std::thread::JoinHandle<()>,
  buffer: Arc<Buffer>,
}

const WIDTH: f64 = 600.0;
const HEIGHT: f64 = 600.0;

impl Display {
  pub fn new(input_sender: crossbeam_channel::Sender<KeyEvent>, buffer: Arc<Buffer>) -> Display {
    let render = render_thread::spawn(WIDTH, HEIGHT, input_sender, Arc::clone(&buffer));

    Display {
      render_thread: render,
      buffer: buffer,
    }
  }
}
