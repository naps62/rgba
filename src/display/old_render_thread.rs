// use crossbeam_channel::Sender;
// use glium::{glutin, Display, Frame};
// use glutin::{dpi::LogicalSize, Event, EventsLoop};
// use std::{thread, time};

// pub fn spawn(input_sender: Sender<Event>) -> thread::JoinHandle<()> {
//   thread::spawn(move || {
//     use glutin::WindowBuilder;

//     let size = LogicalSize::new(160.0, 144.0);

//     let mut events_loop = EventsLoop::new();
//     let window_builder = WindowBuilder::new()
//       .with_dimensions(size)
//       .with_title("RGBA");

//     let context = glutin::ContextBuilder::new();

//     let display = Display::new(window_builder, context, &events_loop).unwrap();

//     render_loop(display, &mut events_loop, input_sender);
//   })
// }


// #[derive(Send, Copy, Clone)]
// struct Pixel {
//   color: [f32, 3];
// }

// fn render_loop(display: Display, events_loop: &mut EventsLoop, input_sender: Sender<Event>) {
//   let mut frames = 0;
//   let mut start = time::Instant::now();

//   let texture = glium::Texture2d::new(&display, [

//   random_pixel(),
//   random_pixel(),
//   random_pixel(),
//   random_pixel()
//   ]);
//   // let vertex_buffer = glium::VertexBuffer::new(&display);

//   loop {
//     events_loop.poll_events(|event| {
//       input_sender.send(event).expect("Failed to send event");
//     });

//     let mut frame = display.draw();
//     render_frame(&mut frame);
//     frame.finish().expect("could not finish frame");

//     let full_duration = start.elapsed().as_secs();
//     if full_duration >= 1 {
//       let fps = (frames as f64) / (start.elapsed().as_secs() as f64);
//       println!("FPS: {}", fps);
//       frames = 0;
//       start = time::Instant::now();
//     } else {
//       frames = frames + 1;
//     }
//   }
// }

// fn render_frame(frame: &mut Frame) {
//   use glium::Surface;

//   frame.clear_color(0.5, 0.0, 0.0, 0.5);
// }

// fn random_pixel(rng: rand::Rand) -> Pixel {
//   Pixel {
//     color: vec![rng.gen::<f32>(), rng.gen::<f32>(), rng.gen::<f32>()]
//   }
// }
