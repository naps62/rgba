use crate::mmu::{addrs::Addr, MMU};

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

pub struct Step {
  mode: Mode,
  mode_clock: u32,
}

use Mode::*;

impl Step {
  pub fn new() -> Step {
    Step {
      mode: Mode::ScanlineOAM,
      mode_clock: 0,
    }
  }

  // inspired in
  // http://imrannazar.com/GameBoy-Emulation-in-JavaScript:-step.Timings
  pub fn calc<M: MMU>(&mut self, cycles: u8, mmu: &mut M) -> Result {
    let line = mmu.read8(Addr::CurrentScanLine as usize);

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
          mmu.write8(Addr::CurrentScanLine, line + 1);

          if line == 143 {
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
          mmu.write8(Addr::CurrentScanLine, line + 1);

          if line > 152 {
            self.mode = ScanlineOAM;
            mmu.write8(Addr::CurrentScanLine, 0);
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
  use crate::mmu::test_mmu::TestMMU;

  #[test]
  fn initial() {
    let step = Step::new();

    assert_eq!(step.mode, ScanlineOAM);
    assert_eq!(step.mode_clock, 0);
  }

  #[test]
  fn after_8_cycles() {
    let mut mmu = TestMMU::new();
    let mut step = Step::new();

    step.calc(8, &mut mmu);

    assert_eq!(step.mode, ScanlineOAM);
    assert_eq!(mmu.read8(Addr::CurrentScanLine), 0);
    assert_eq!(step.mode_clock, 8);
  }

  #[test]
  fn after_80_cycles() {
    let mut mmu = TestMMU::new();
    let mut step = Step::new();

    step.calc(80, &mut mmu);

    assert_eq!(step.mode, ScanlineVRAM);
    assert_eq!(mmu.read8(Addr::CurrentScanLine), 0);
    assert_eq!(step.mode_clock, 0);
  }

  #[test]
  fn after_204_cycles() {
    let mut mmu = TestMMU::new();
    let mut step = Step::new();

    step.calc(80, &mut mmu);
    step.calc(172, &mut mmu);

    assert_eq!(step.mode, HBlank);
    assert_eq!(mmu.read8(Addr::CurrentScanLine), 0);
    assert_eq!(step.mode_clock, 0);
  }

  #[test]
  fn after_456_cycles() {
    let mut mmu = TestMMU::new();
    let mut step = Step::new();

    step.calc(80, &mut mmu);
    step.calc(172, &mut mmu);
    step.calc(204, &mut mmu);

    assert_eq!(step.mode, ScanlineOAM);
    assert_eq!(mmu.read8(Addr::CurrentScanLine), 1);
    assert_eq!(step.mode_clock, 0);
  }

  #[test]
  fn after_144_lines() {
    let mut mmu = TestMMU::new();
    let mut step = Step::new();

    for _i in 0..144 {
      step.calc(80, &mut mmu);
      step.calc(172, &mut mmu);
      step.calc(204, &mut mmu);
    }

    assert_eq!(step.mode, VBlank);
    assert_eq!(mmu.read8(Addr::CurrentScanLine), 144);
    assert_eq!(step.mode_clock, 0);
  }

  #[test]
  fn after_154_lines() {
    let mut mmu = TestMMU::new();
    let mut step = Step::new();

    for _ in 0..144 {
      step.calc(80, &mut mmu);
      step.calc(172, &mut mmu);
      step.calc(204, &mut mmu);
    }

    for _ in 0..10 {
      step.calc(201, &mut mmu);
      step.calc(255, &mut mmu);
    }

    assert_eq!(step.mode, ScanlineOAM);
    assert_eq!(mmu.read8(Addr::CurrentScanLine), 0);
    assert_eq!(step.mode_clock, 0);
  }
}
