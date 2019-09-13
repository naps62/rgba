extern crate crossbeam_channel;
extern crate glium;

mod render_thread;

use glium::glutin::Event;

#[allow(dead_code)]
pub struct Display {
  render_thread: std::thread::JoinHandle<()>,
}

impl Display {
  pub fn new(input_sender: crossbeam_channel::Sender<Event>) -> Display {
    Display {
      render_thread: render_thread::spawn(input_sender),
    }
  }
}
