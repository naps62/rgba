extern crate graphics;
extern crate piston;
extern crate rand;

mod buffer;
mod render_thread;

use crate::input::KeyEvent;
use buffer::Buffer;
use crossbeam_channel::Sender;

#[allow(dead_code)]
pub struct Display {
  render_thread: std::thread::JoinHandle<()>,
  buffer_sender: Sender<Buffer>,
}

impl Display {
  pub fn new(input_sender: crossbeam_channel::Sender<KeyEvent>) -> Display {
    let (buffer_sender, buffer_receiver) = crossbeam_channel::unbounded();

    let sender_clone = buffer_sender.clone();

    std::thread::spawn(move || loop {
      sender_clone.send(Buffer::randomize(100, 100));
      // std::thread::sleep(std::time::Duration::from_millis(100));
    });

    Display {
      render_thread: render_thread::spawn(input_sender, buffer_receiver),
      buffer_sender,
    }
  }
}
