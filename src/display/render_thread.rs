use glium::{glutin, Display, Frame, Surface};
use schedule_recv::oneshot_ms;
use std::{thread, time};

pub fn spawn() -> thread::JoinHandle<()> {
  thread::spawn(move || {
    let display = setup_display();

    main_loop(display);
  })
}

fn setup_display() -> Display {
  let size = glutin::dpi::LogicalSize::new(800.0, 600.0);

  let events_loop = glutin::EventsLoop::new();
  let window_builder = glutin::WindowBuilder::new()
    .with_dimensions(size)
    .with_title("RGBA");

  let context = glutin::ContextBuilder::new();

  Display::new(window_builder, context, &events_loop).unwrap()
}

fn main_loop(display: Display) {
  let mut frames = 0;
  let mut start = time::Instant::now();

  loop {
    let frame_start = time::Instant::now();

    let mut frame = display.draw();
    render_frame(&mut frame);
    frame.finish();

    let frame_duration = frame_start.elapsed().as_millis() as u64;
    thread::sleep(time::Duration::from_millis(16 - frame_duration));

    frames = frames + 1;

    let full_duration = start.elapsed().as_secs();
    if full_duration >= 1 {
      let fps = (frames as f64) / (start.elapsed().as_secs() as f64);
      println!("FPS: {}", fps);
      frames = 0;
      start = time::Instant::now();
    }
  }
}

fn render_frame(frame: &mut Frame) {
  frame.clear_color(1.0, 0.0, 0.0, 0.5);
}
