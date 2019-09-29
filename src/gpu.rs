#[derive(Debug, PartialEq)]
pub enum Mode {
  ScanlineOAM = 2,
  ScanlineVRAM = 3,
  HBlank = 0,
  VBlank = 1,
}

pub struct GPU {
  mode: Mode,
  mode_clock: u32,
  line: u8,
}

impl GPU {
  pub fn new() -> GPU {
    GPU {
      mode: Mode::ScanlineOAM,
      mode_clock: 0,
      line: 0,
    }
  }

  pub fn step(&mut self, cycles: u8) {
    use Mode::*;

    self.mode_clock = self.mode_clock + cycles as u32;

    match &self.mode {
      ScanlineOAM => {
        if self.mode_clock >= 80 {
          self.mode_clock = 0;
          self.mode = ScanlineVRAM;
        }
      }
      ScanlineVRAM => {
        if self.mode_clock >= 172 {
          self.mode_clock = 0;
          self.mode = HBlank;
        }
      }
      HBlank => {
        if self.mode_clock >= 204 {
          self.mode_clock = 0;
          self.line = self.line + 1;

          if self.line == 144 {
            self.mode = VBlank;

          // panic!("render screen");
          } else {
            self.mode = ScanlineOAM;
          }
        }
      }
      VBlank => {
        if self.mode_clock >= 456 {
          self.mode_clock = 0;
          self.line = self.line + 1;

          if self.line == 154 {
            self.mode = ScanlineOAM;
            self.line = 0;
          }
        }
      }
      _ => unreachable!(),
    }
  }

  pub fn read8(&self, _index: usize) -> u8 {
    1
  }

  pub fn write8(&mut self, _index: usize, _value: u8) {}
}

#[cfg(test)]
mod tests {
  use super::Mode::*;
  use super::*;

  #[test]
  fn initial() {
    let gpu = GPU::new();

    assert_eq!(gpu.mode, ScanlineOAM);
    assert_eq!(gpu.line, 0);
    assert_eq!(gpu.mode_clock, 0);
  }

  #[test]
  fn after_8_cycles() {
    let mut gpu = GPU::new();

    gpu.step(8);

    assert_eq!(gpu.mode, ScanlineOAM);
    assert_eq!(gpu.line, 0);
    assert_eq!(gpu.mode_clock, 8);
  }

  #[test]
  fn after_80_cycles() {
    let mut gpu = GPU::new();

    gpu.step(80);

    assert_eq!(gpu.mode, ScanlineVRAM);
    assert_eq!(gpu.line, 0);
    assert_eq!(gpu.mode_clock, 0);
  }

  #[test]
  fn after_204_cycles() {
    let mut gpu = GPU::new();

    gpu.step(80);
    gpu.step(172);

    assert_eq!(gpu.mode, HBlank);
    assert_eq!(gpu.line, 0);
    assert_eq!(gpu.mode_clock, 0);
  }

  #[test]
  fn after_456_cycles() {
    let mut gpu = GPU::new();

    gpu.step(80);
    gpu.step(172);
    gpu.step(204);

    assert_eq!(gpu.mode, ScanlineOAM);
    assert_eq!(gpu.line, 1);
    assert_eq!(gpu.mode_clock, 0);
  }

  #[test]
  fn after_144_lines() {
    let mut gpu = GPU::new();

    for _i in 0..144 {
      gpu.step(80);
      gpu.step(172);
      gpu.step(204);
    }

    assert_eq!(gpu.mode, VBlank);
    assert_eq!(gpu.line, 144);
    assert_eq!(gpu.mode_clock, 0);
  }

  #[test]
  fn after_154_lines() {
    let mut gpu = GPU::new();

    for _ in 0..144 {
      gpu.step(80);
      gpu.step(172);
      gpu.step(204);
    }

    for _ in 0..10 {
      gpu.step(201);
      gpu.step(255);
    }

    assert_eq!(gpu.mode, ScanlineOAM);
    assert_eq!(gpu.line, 0);
    assert_eq!(gpu.mode_clock, 0);
  }
}
