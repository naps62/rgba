extern crate crossbeam_channel;
extern crate glium;

use crossbeam_channel::Receiver;
use std::thread;

use glium::glutin::{DeviceEvent, ElementState, Event, KeyboardInput, VirtualKeyCode};

#[allow(dead_code)]
pub struct Input {
  thread: thread::JoinHandle<()>,
}

impl Input {
  pub fn new(receiver: Receiver<Event>) -> Input {
    let thread = thread::spawn(move || receiver_loop(receiver));

    Input { thread: thread }
  }
}

fn receiver_loop(receiver: Receiver<Event>) {
  loop {
    match receiver.recv().expect("Failed to receive input event") {
      Event::DeviceEvent {
        event: DeviceEvent::Key(key),
        ..
      } => handle_key(key),
      _ => (),
    }
  }
}

fn handle_key(key: KeyboardInput) {
  match key {
    KeyboardInput {
      state: ElementState::Pressed,
      virtual_keycode: Some(keycode),
      ..
    } => handle_key_press(keycode),

    KeyboardInput {
      state: ElementState::Released,
      virtual_keycode: Some(keycode),
      ..
    } => handle_key_release(keycode),
    _ => (),
  }
}

fn handle_key_press(keycode: VirtualKeyCode) {
  match keycode {
    VirtualKeyCode::Escape => std::process::exit(0),
    VirtualKeyCode::Up => println!("UP"),
    VirtualKeyCode::Down => println!("DOWN"),
    VirtualKeyCode::Left => println!("LEFT"),
    VirtualKeyCode::Right => println!("RIGHT"),
    _ => (),
  }
}

fn handle_key_release(keycode: VirtualKeyCode) {
  match keycode {
    _ => (),
  }
}
