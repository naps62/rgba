extern crate graphics;
extern crate piston;

mod render_thread;

use crate::input::KeyEvent;

#[allow(dead_code)]
pub struct Display {
  render_thread: std::thread::JoinHandle<()>,
}

impl Display {
  pub fn new(input_sender: crossbeam_channel::Sender<KeyEvent>) -> Display {
    Display {
      render_thread: render_thread::spawn(input_sender),
    }
  }
}
