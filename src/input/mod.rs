use crossbeam_channel::Receiver;
use std::thread;

use piston::keyboard::Key;

pub type KeyEvent = (Key, bool);

#[allow(dead_code)]
pub struct Input {
  thread: thread::JoinHandle<()>,
}

impl Input {
  pub fn new(receiver: Receiver<KeyEvent>) -> Input {
    let thread = thread::spawn(move || receiver_loop(receiver));

    Input { thread: thread }
  }
}

fn receiver_loop(receiver: Receiver<KeyEvent>) {
  loop {
    let (key, state) = receiver.recv().expect("Failed to receive input event");

    if state {
      handle_key_press(key)
    } else {
      handle_key_release(key)
    }
  }
}

fn handle_key_press(keycode: Key) {
  match keycode {
    Key::Escape => std::process::exit(0),
    Key::Up => println!("UP"),
    Key::Down => println!("DOWN"),
    Key::Left => println!("LEFT"),
    Key::Right => println!("RIGHT"),
    _ => (),
  }
}

fn handle_key_release(keycode: Key) {
  match keycode {
    _ => (),
  }
}
