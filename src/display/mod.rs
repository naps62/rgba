extern crate glium;

mod render_thread;

pub struct Display {
  render_thread: std::thread::JoinHandle<()>,
}

impl Display {
  pub fn new() -> Display {
    Display {
      render_thread: render_thread::spawn(),
    }
  }
}
