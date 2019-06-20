extern crate glium;

use glium::{glutin, Surface};

pub struct Display {
  size: glutin::dpi::LogicalSize,
  display: Option<glium::Display>,
}

impl Display {
  pub fn new() -> Display {
    Display {
      size: glutin::dpi::LogicalSize::new(800.0, 600.0),
      display: None,
    }
  }

  pub fn show(&mut self) {
    let events_loop = glutin::EventsLoop::new();
    let window_builder = glutin::WindowBuilder::new()
      .with_dimensions(self.size)
      .with_title("RGBA");

    let context = glutin::ContextBuilder::new();

    self.display = Some(glium::Display::new(window_builder, context, &events_loop).unwrap());
  }

  pub fn draw(self) {
    let mut frame = self.display.unwrap().draw();

    frame.clear_color(1.0, 0.0, 0.0, 1.0);

    frame.finish();
  }
}
