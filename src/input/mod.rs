extern crate crossbeam_channel;
extern crate glium;

use crossbeam_channel::Receiver;
use std::thread;

pub type Event = glium::glutin::Event;

#[allow(dead_code)]
pub struct Input {
  thread: thread::JoinHandle<()>,
}

impl Input {
  pub fn new(receiver: Receiver<Event>) -> Input {
    let thread = thread::spawn(move || {
      let event = receiver.recv().expect("failed to receive an event");

      println!("{:?}", event)
    });

    Input { thread: thread }
  }
}
