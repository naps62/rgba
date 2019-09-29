extern crate gfx_graphics;
extern crate piston;
extern crate piston_window;
extern crate texture;

use piston_window::{PistonWindow, Texture};

use crate::display::buffer::Buffer;
use crate::input::KeyEvent;
use crossbeam_channel::Sender;
use piston::Button;
use piston::Event;
use std::{sync::Arc, thread};

pub fn spawn(
  width: f64,
  height: f64,
  input_sender: Sender<KeyEvent>,
  buffer: Arc<Buffer>,
) -> thread::JoinHandle<()> {
  use piston::window::WindowSettings;

  thread::spawn(move || {
    let ref mut window: PistonWindow = WindowSettings::new("RGBA", [width, height])
      .build()
      .unwrap();

    render_loop(window, input_sender, buffer);
  })
}

fn buffer_to_texture(
  window: &mut PistonWindow,
  buffer: &Arc<Buffer>,
) -> Texture<gfx_device_gl::Resources> {
  use texture::{Filter, TextureSettings};

  Texture::from_image(
    &mut window.create_texture_context(),
    &buffer.get(),
    &TextureSettings::new().filter(Filter::Nearest),
  )
  .unwrap()
}

fn render_loop(window: &mut PistonWindow, input_sender: Sender<KeyEvent>, buffer: Arc<Buffer>) {
  use piston::input::{PressEvent, ReleaseEvent, RenderEvent};

  while let Some(event) = window.next() {
    if let Some(_args) = event.render_args() {
      render(window, &event, &buffer);
    }

    if let Some(args) = event.press_args() {
      process_button(args, true, &input_sender);
    }

    if let Some(args) = event.release_args() {
      process_button(args, false, &input_sender);
    }
  }
}

fn render(window: &mut PistonWindow, event: &Event, buffer: &Arc<Buffer>) {
  use piston_window::Window;

  let size = window.size();

  let img = buffer_to_texture(window, buffer);

  window.draw_2d(event, |c, g, _| {
    use graphics::*;

    clear(color::WHITE, g);

    let (iw, ih) = img.get_size();

    let dx: f64 = size.width as f64 / iw as f64;
    let dy: f64 = size.height as f64 / ih as f64;
    image(&img, c.transform.trans(0.0, 0.0).scale(dx, dy), g);
  });
}

fn process_button(args: Button, state: bool, input_sender: &Sender<KeyEvent>) {
  match args {
    piston::Button::Keyboard(key) => {
      input_sender.send((key, state)).unwrap();
    }
    _ => (),
  }
}
