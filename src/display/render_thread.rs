use glium::{glutin, Display, Frame};
use glutin::{dpi::LogicalSize, Event, EventsLoop};
use std::{thread, time};

pub fn spawn() -> thread::JoinHandle<()> {
  thread::spawn(move || {
    use glutin::WindowBuilder;

    let size = LogicalSize::new(800.0, 600.0);

    let mut events_loop = EventsLoop::new();
    let window_builder = WindowBuilder::new()
      .with_dimensions(size)
      .with_title("RGBA");

    let context = glutin::ContextBuilder::new();

    let display = Display::new(window_builder, context, &events_loop).unwrap();

    render_loop(&mut events_loop, display);
  })
}

fn render_loop(events_loop: &mut EventsLoop, display: Display) {
  use glium::Surface;
  let mut frames = 0;
  let mut start = time::Instant::now();

  loop {
    events_loop.poll_events(handle_event);

    let mut frame = display.draw();
    render_frame(&mut frame);
    frame.finish().expect("could not finish frame");

    let full_duration = start.elapsed().as_secs();
    if full_duration >= 1 {
      let fps = (frames as f64) / (start.elapsed().as_secs() as f64);
      println!("FPS: {}", fps);
      frames = 0;
      start = time::Instant::now();
    } else {
      frames = frames + 1;
    }
  }
}

fn render_frame(frame: &mut Frame) {
  use glium::Surface;

  frame.clear_color(1.0, 0.0, 0.0, 0.5);
}

fn handle_event(event: Event) {
  use glutin::{DeviceEvent::Key, ElementState, Event};

  match event {
    Event::DeviceEvent {
      event: Key(key),
      ..
    } => println!("key {:?}", key),

    _ => {}
  }
}
