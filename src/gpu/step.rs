#[derive(Debug, PartialEq)]
pub enum Mode {
  ScanlineOAM = 2,
  ScanlineVRAM = 3,
  HBlank = 0,
  VBlank = 1,
}

#[derive(Debug, PartialEq)]
pub enum Result {
  Renderscan,
  Noop,
}

pub struct Scroll {
  pub x: u32,
  pub y: u32,
}

pub struct Step {
  mode: Mode,
  mode_clock: u32,
  pub line: u32,
  pub scroll: Scroll,
}

use Mode::*;

impl Step {
  pub fn new() -> Step {
    Step {
      mode: Mode::ScanlineOAM,
      mode_clock: 0,
      line: 0,
      scroll: Scroll { x: 0, y: 0 },
    }
  }

  // inspired in
  // http://imrannazar.com/GameBoy-Emulation-in-JavaScript:-step.Timings
  pub fn calc(&mut self, cycles: u8) -> Result {
    self.mode_clock = self.mode_clock + cycles as u32;

    match self.mode {
      ScanlineOAM => {
        if self.mode_clock >= 80 {
          self.mode_clock = 0;
          self.mode = ScanlineVRAM;
        }

        Result::Noop
      }
      ScanlineVRAM => {
        if self.mode_clock >= 172 {
          self.mode_clock = 0;
          self.mode = HBlank;

          Result::Renderscan
        } else {
          Result::Noop
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

        Result::Noop
      }
      VBlank => {
        if self.mode_clock >= 456 {
          self.mode_clock = 0;
          self.line = self.line + 1;

          if self.line > 153 {
            self.mode = ScanlineOAM;
            self.line = 0;
          }
        }

        Result::Noop
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn initial() {
    let step = Step::new();

    assert_eq!(step.mode, ScanlineOAM);
    assert_eq!(step.line, 0);
    assert_eq!(step.mode_clock, 0);
  }

  #[test]
  fn after_8_cycles() {
    let mut step = Step::new();

    step.calc(8);

    assert_eq!(step.mode, ScanlineOAM);
    assert_eq!(step.line, 0);
    assert_eq!(step.mode_clock, 8);
  }

  #[test]
  fn after_80_cycles() {
    let mut step = Step::new();

    step.calc(80);

    assert_eq!(step.mode, ScanlineVRAM);
    assert_eq!(step.line, 0);
    assert_eq!(step.mode_clock, 0);
  }

  #[test]
  fn after_204_cycles() {
    let mut step = Step::new();

    step.calc(80);
    step.calc(172);

    assert_eq!(step.mode, HBlank);
    assert_eq!(step.line, 0);
    assert_eq!(step.mode_clock, 0);
  }

  #[test]
  fn after_456_cycles() {
    let mut step = Step::new();

    step.calc(80);
    step.calc(172);
    step.calc(204);

    assert_eq!(step.mode, ScanlineOAM);
    assert_eq!(step.line, 1);
    assert_eq!(step.mode_clock, 0);
  }

  #[test]
  fn after_144_lines() {
    let mut step = Step::new();

    for _i in 0..144 {
      step.calc(80);
      step.calc(172);
      step.calc(204);
    }

    assert_eq!(step.mode, VBlank);
    assert_eq!(step.line, 144);
    assert_eq!(step.mode_clock, 0);
  }

  #[test]
  fn after_154_lines() {
    let mut step = Step::new();

    for _ in 0..144 {
      step.calc(80);
      step.calc(172);
      step.calc(204);
    }

    for _ in 0..10 {
      step.calc(201);
      step.calc(255);
    }

    assert_eq!(step.mode, ScanlineOAM);
    assert_eq!(step.line, 0);
    assert_eq!(step.mode_clock, 0);
  }
}
