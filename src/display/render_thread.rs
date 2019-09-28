extern crate glium;
extern crate glium_graphics;
extern crate piston;

use crate::display::buffer::Buffer;
use crate::input::KeyEvent;
use crossbeam_channel::{Receiver, Sender};
use glium_graphics::{Glium2d, GliumWindow, Texture};
use piston::Button;
use std::thread;

const WINDOW_WIDTH: f64 = 600.0;
const WINDOW_HEIGHT: f64 = 600.0;

const OPEN_GL: glium_graphics::OpenGL = glium_graphics::OpenGL::V3_2;

pub fn spawn(
  input_sender: Sender<KeyEvent>,
  buffer_receiver: Receiver<Buffer>,
) -> thread::JoinHandle<()> {
  use piston::window::WindowSettings;

  thread::spawn(move || {
    let ref mut window: GliumWindow = WindowSettings::new("RGBA", [WINDOW_WIDTH, WINDOW_HEIGHT])
      .graphics_api(OPEN_GL)
      .build()
      .unwrap();

    render_loop(window, input_sender, buffer_receiver);
  })
}

fn random_pixel<T>(rng: &mut T) -> (f32, f32, f32)
where
  T: rand::Rng,
{
  let x: f32 = rng.gen();

  (x, x, x)
}

fn buffer_to_texture(window: &GliumWindow, buffer: Buffer) -> Texture {
  use glium::texture::srgb_texture2d::SrgbTexture2d;

  let srgb_texture = SrgbTexture2d::new(window, buffer.get()).unwrap();

  Texture::new(srgb_texture)
}

fn render_loop(
  window: &mut GliumWindow,
  input_sender: Sender<KeyEvent>,
  buffer_receiver: Receiver<Buffer>,
) {
  use piston::input::{PressEvent, ReleaseEvent, RenderEvent};

  let mut g2d = Glium2d::new(OPEN_GL, window);

  while let Some(event) = window.next() {
    if let Some(args) = event.render_args() {
      render(&window, &mut g2d, args, &buffer_receiver);
    }

    if let Some(args) = event.press_args() {
      process_button(args, true, &input_sender);
    }

    if let Some(args) = event.release_args() {
      process_button(args, false, &input_sender);
    }
  }
}

fn render(
  window: &GliumWindow,
  g2d: &mut Glium2d,
  args: piston::RenderArgs,
  buffer_receiver: &Receiver<Buffer>,
) {
  if let Ok(buffer) = buffer_receiver.try_recv() {
    println!("rendering");
    let mut target = window.draw();
    let img = buffer_to_texture(window, buffer);

    g2d.draw(&mut target, args.viewport(), |c, g| {
      use graphics::*;

      clear(color::WHITE, g);

      let (iw, ih) = img.get_size();

      let dx: f64 = WINDOW_WIDTH / iw as f64;
      let dy: f64 = WINDOW_HEIGHT / ih as f64;
      image(&img, c.transform.trans(0.0, 0.0).scale(dx, dy), g);
    });
    target.finish().unwrap();
  };
}

fn process_button(args: Button, state: bool, input_sender: &Sender<KeyEvent>) {
  match args {
    piston::Button::Keyboard(key) => {
      input_sender.send((key, state)).unwrap();
    }
    _ => (),
  }
}
