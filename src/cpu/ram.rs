pub struct RAM {
  internal: [i8; 8 * 1024], // 8 KB of internal RAM
  video: [i8; 8 * 1024],    // 8 KB of video RAM
}

impl RAM {
  pub fn new() -> RAM {
    RAM {
      internal: [0; 8 * 1024],
      video: [0; 8 * 1024],
    }
  }
}
