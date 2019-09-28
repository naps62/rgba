extern crate glium;
extern crate glium_graphics;
extern crate piston;

use crate::display::buffer::Buffer;
use crate::input::KeyEvent;
use crossbeam_channel::Sender;
use glium_graphics::{Glium2d, GliumWindow, Texture};
use piston::Button;
use std::{sync::Arc, thread};

const OPEN_GL: glium_graphics::OpenGL = glium_graphics::OpenGL::V3_2;

pub fn spawn(
  width: f64,
  height: f64,
  input_sender: Sender<KeyEvent>,
  buffer: Arc<Buffer>,
) -> thread::JoinHandle<()> {
  use piston::window::WindowSettings;

  thread::spawn(move || {
    let ref mut window: GliumWindow = WindowSettings::new("RGBA", [width, height])
      .graphics_api(OPEN_GL)
      .build()
      .unwrap();

    render_loop(window, input_sender, buffer);
  })
}

fn buffer_to_texture(window: &GliumWindow, buffer: &Arc<Buffer>) -> Texture {
  use glium::texture::srgb_texture2d::SrgbTexture2d;

  let srgb_texture = SrgbTexture2d::new(window, buffer.get().clone()).unwrap();

  Texture::new(srgb_texture)
}

fn render_loop(window: &mut GliumWindow, input_sender: Sender<KeyEvent>, buffer: Arc<Buffer>) {
  use piston::input::{PressEvent, ReleaseEvent, RenderEvent};

  let mut g2d = Glium2d::new(OPEN_GL, window);

  while let Some(event) = window.next() {
    if let Some(args) = event.render_args() {
      render(&window, &mut g2d, args, &buffer);
    }

    if let Some(args) = event.press_args() {
      process_button(args, true, &input_sender);
    }

    if let Some(args) = event.release_args() {
      process_button(args, false, &input_sender);
    }
  }
}

fn render(window: &GliumWindow, g2d: &mut Glium2d, args: piston::RenderArgs, buffer: &Arc<Buffer>) {
  use glium::Surface;

  let mut target = window.draw();
  let (width, height) = target.get_dimensions();

  let img = buffer_to_texture(window, buffer);

  g2d.draw(&mut target, args.viewport(), |c, g| {
    use graphics::*;

    clear(color::WHITE, g);

    let (iw, ih) = img.get_size();

    let dx: f64 = width as f64 / iw as f64;
    let dy: f64 = height as f64 / ih as f64;
    image(&img, c.transform.trans(0.0, 0.0).scale(dx, dy), g);
  });
  target.finish().unwrap();
}

fn process_button(args: Button, state: bool, input_sender: &Sender<KeyEvent>) {
  match args {
    piston::Button::Keyboard(key) => {
      input_sender.send((key, state)).unwrap();
    }
    _ => (),
  }
}
