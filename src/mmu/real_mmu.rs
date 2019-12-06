use super::MMU;
use std::fs;

// note: memory is little-endian
// when reading a 2 byte number, we need to invert the two bytes

// http://imrannazar.com/GameBoy-Emulation-in-JavaScript:-Memory
// http://gameboy.mongenel.com/dmg/asmmemmap.html<Paste>
// http://gameboy.mongenel.com/dmg/asmmemmap.html
type MemRange = (usize, usize);

// Restart and Interrupt vectors
const BOOT_BEG: usize = 0x0000;
const BOOT_END: usize = 0x00ff;
// const BOOT_RANGE: MemRange = (BOOT_BEG, BOOT_END);

const INTERRUPT_BEG: usize = 0x0000;
const INTERRUPT_END: usize = 0x00ff;
const INTERRUPT_RANGE: MemRange = (INTERRUPT_BEG, INTERRUPT_END);

// ROM, bank 0
const ROM0_BEG: usize = 0x0000;
const ROM0_END: usize = 0x3fff;
// const ROM0_RANGE: MemRange = (ROM0_BEG, ROM0_END);

// const BIOS_BEG: usize = 0x0000;
// const BIOS_END: usize = 0x00ff;
// const BIOS_RANGE: MemRange = (BIOS_BEG, BIOS_END);

// const HEADER_BEG: usize = 0x0100;
// const HEADER_END: usize = 0x014f;
// const RANGE_HEADER: MemRange = (HEADER_BEG, HEADER_END);

// ROM, switchable banks
const ROMX_BEG: usize = 0x4000;
const ROMX_END: usize = 0x7fff;
// const ROMX_RANGE: MemRange = (ROMX_BEG, ROMX_END);

// Video RAM
pub const VRAM_BEG: usize = 0x8000;
pub const VRAM_END: usize = 0x9fff;
const VRAM_RANGE: MemRange = (VRAM_BEG, VRAM_END);

// External (cartridge) RAM
const ERAM_BEG: usize = 0xa000;
const ERAM_END: usize = 0xbfff;
const ERAM_RANGE: MemRange = (ERAM_BEG, ERAM_END);

// Work RAM, Bank 0
const WRAM0_BEG: usize = 0xc000;
const WRAM0_END: usize = 0xcfff;
const WRAM0_RANGE: MemRange = (WRAM0_BEG, WRAM0_END);

// Work RAM, switchable banks (only bank 1 in non-CGB)
const WRAMX_BEG: usize = 0xd000;
const WRAMX_END: usize = 0xdfff;
const WRAMX_RANGE: MemRange = (WRAMX_BEG, WRAMX_END);

// Echo RAM (reserved, do not use)
// const ECHO_BEG: usize = 0xe000;
// const ECHO_END: usize = 0xfdff;
// const ECHO_RANGE: MemRange = (ECHO_BEG, ECHO_END);

// OAM (Object Attribute Memory)
// const OAM_BEG: usize = 0xfe00;
// const OAM_END: usize = 0xfe9f;
// const OAM_RANGE: MemRange = (OAM_BEG, OAM_END);

// Unused memory range
// const UNUSED_BEG: usize = 0xfea0;
// const UNUSED_END: usize = 0xfeff;

// IO
const IO_BEG: usize = 0xff00;
const IO_END: usize = 0xff7f;
const IO_RANGE: MemRange = (IO_BEG, IO_END);

// Zero-page RAM
const ZRAM_BEG: usize = 0xff80;
const ZRAM_END: usize = 0xfffe;
const ZRAM_RANGE: MemRange = (ZRAM_BEG, ZRAM_END);

const FLAG_BOOT: usize = 0xff50;
const FLAG_INTERRUPT: usize = 0xffff;

macro_rules! declare_mem_bank {
  ($range:ident) => {
    [u8; $range.1 - $range.0 + 1]
  };
}

macro_rules! init_mem_bank {
  ($range:ident) => {
    [0; $range.1 - $range.0 + 1]
  };
}

pub struct RealMMU {
  boot: Vec<u8>,
  cartridge: Vec<u8>,
  interrupts: declare_mem_bank!(INTERRUPT_RANGE),
  vram: declare_mem_bank!(VRAM_RANGE),
  eram: declare_mem_bank!(ERAM_RANGE),
  wram0: declare_mem_bank!(WRAM0_RANGE),
  wramx: declare_mem_bank!(WRAMX_RANGE),
  io: declare_mem_bank!(IO_RANGE),
  zram: declare_mem_bank!(ZRAM_RANGE),
  flag_interrupt: u8,
}

impl RealMMU {
  pub fn new(boot_rom: bool, cartridge: Vec<u8>) -> RealMMU {
    let boot = if boot_rom {
      fs::read("assets/boot_rom.bin").unwrap()
    } else {
      Vec::new()
    };

    let mut mmu = RealMMU {
      boot: boot,
      cartridge: cartridge,
      interrupts: init_mem_bank!(INTERRUPT_RANGE),
      vram: init_mem_bank!(VRAM_RANGE),
      eram: init_mem_bank!(ERAM_RANGE),
      wram0: init_mem_bank!(WRAM0_RANGE),
      wramx: init_mem_bank!(WRAMX_RANGE),
      io: init_mem_bank!(IO_RANGE),
      zram: init_mem_bank!(ZRAM_RANGE),
      flag_interrupt: 0u8,
    };

    if boot_rom {
      mmu.set_flag(FLAG_BOOT, 0b0000_0001);
    }

    mmu
  }
}

use std::convert::Into;

impl MMU for RealMMU {
  fn read8<I>(&self, idx: I) -> u8
  where
    I: Into<usize>,
  {
    let index: usize = idx.into();

    match index {
      BOOT_BEG..=BOOT_END if self.read8(FLAG_BOOT) == 1 => self.boot[index],
      INTERRUPT_BEG..=INTERRUPT_END if self.read8(FLAG_BOOT) > 0 => self.interrupts[index],
      ROM0_BEG..=ROM0_END => self.cartridge[index],
      ROMX_BEG..=ROMX_END => self.cartridge[index - ROMX_BEG],
      ERAM_BEG..=ERAM_END => self.eram[index - ERAM_BEG],
      VRAM_BEG..=VRAM_END => self.vram[index - VRAM_BEG],
      WRAM0_BEG..=WRAM0_END => self.wram0[index - WRAM0_BEG],
      WRAMX_BEG..=WRAMX_END => self.wramx[index - WRAMX_BEG],
      ZRAM_BEG..=ZRAM_END => self.zram[index - ZRAM_BEG],
      IO_BEG..=IO_END => self.io[index - IO_BEG],
      FLAG_INTERRUPT => self.flag_interrupt,
      _ => panic!("Unsupported MMU read8 to address 0x{:x}", index),
    }
  }
  fn read16<I>(&self, idx: I) -> u16
  where
    I: Into<usize>,
  {
    let index: usize = idx.into();

    ((self.read8(index + 1) as u16) << 8) | (self.read8(index) as u16)
  }

  fn write8<I>(&mut self, idx: I, value: u8)
  where
    I: Into<usize>,
  {
    let index: usize = idx.into();

    match index {
      INTERRUPT_BEG..=INTERRUPT_END => self.interrupts[index] = value,
      VRAM_BEG..=VRAM_END => self.vram[index - VRAM_BEG] = value,
      WRAM0_BEG..=WRAM0_END => self.wram0[index - WRAM0_BEG] = value,
      WRAMX_BEG..=WRAMX_END => self.wramx[index - WRAMX_BEG] = value,
      ZRAM_BEG..=ZRAM_END => self.zram[index - ZRAM_BEG] = value,
      IO_BEG..=IO_END => self.io[index - IO_BEG] = value,
      FLAG_INTERRUPT => self.flag_interrupt = value,
      _ => panic!("Unsupported MMU write8 to address {:#06x}", index),
    };
  }

  fn write16<I>(&mut self, idx: I, value: u16)
  where
    I: Into<usize>,
  {
    let index: usize = idx.into();

    self.write8(index, (value & 0x00FF) as u8);
    self.write8(index + 1, ((value & 0xFF00) >> 8) as u8);
  }

  fn set_flag<I, U>(&mut self, addr: I, mask: U)
  where
    I: Into<usize>,
    U: Into<u8>,
  {
    let address: usize = addr.into();

    let real_address = match address {
      IO_BEG..=IO_END => address - IO_BEG,

      _ => panic!("Unsupported MMU flag address {:#06x}", address),
    };

    self.io[real_address] = self.io[real_address] | mask.into();
  }

  fn unset_flag<I, U>(&mut self, addr: I, mask: U)
  where
    I: Into<usize>,
    U: Into<u8>,
  {
    let address: usize = addr.into();

    let real_address = match address {
      IO_BEG..=IO_END => address - IO_BEG,

      _ => panic!("Unsupported MMU flag address {:#06x}", address),
    };

    self.io[real_address] = self.io[real_address] ^ mask.into();
  }

  fn get_flag<I, U>(&self, addr: I, mask: U) -> bool
  where
    I: Into<usize>,
    U: Into<u8>,
  {
    let address: usize = addr.into();

    let real_address = match address {
      IO_BEG..=IO_END => address - IO_BEG,

      _ => panic!("Unsupported MMU flag address {:#06x}", address),
    };

    let current = self.io[real_address];

    (current & mask.into()) > 0
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  macro_rules! instantiate_mmu {
    () => {{
      RealMMU::new(false, vec![0; 1024 * 32])
    }};
  }

  #[test]
  fn read8_readable() {
    let mmu = instantiate_mmu!();

    assert_eq!(mmu.read8(ROM0_BEG), 0);
    assert_eq!(mmu.read8(ROM0_END), 0);

    assert_eq!(mmu.read8(ROMX_BEG), 0);
    assert_eq!(mmu.read8(ROMX_END), 0);

    assert_eq!(mmu.read8(ERAM_BEG), 0);
    assert_eq!(mmu.read8(ERAM_END), 0);

    assert_eq!(mmu.read8(WRAM0_BEG), 0);
    assert_eq!(mmu.read8(WRAM0_END), 0);

    assert_eq!(mmu.read8(WRAMX_BEG), 0);
    assert_eq!(mmu.read8(WRAMX_END), 0);

    assert_eq!(mmu.read8(ZRAM_BEG), 0);
    assert_eq!(mmu.read8(ZRAM_END), 0);
  }

  #[test]
  fn write8_writable() {
    let mut mmu = instantiate_mmu!();

    mmu.write8(WRAM0_BEG, 1);
    assert_eq!(mmu.wram0[0], 1);

    mmu.write8(WRAMX_BEG, 2);
    assert_eq!(mmu.wramx[0], 2);

    mmu.write8(ZRAM_BEG, 3);
    assert_eq!(mmu.zram[0], 3);
  }

  #[test]
  #[should_panic]
  fn write8_rom0() {
    let mut mmu = instantiate_mmu!();

    mmu.write8(ROM0_BEG, 1);
  }

  #[test]
  #[should_panic]
  fn write8_romx() {
    let mut mmu = instantiate_mmu!();

    mmu.write8(ROMX_BEG, 1);
  }

  #[test]
  fn set_flag() {
    let mut mmu = instantiate_mmu!();

    mmu.set_flag(FLAG_BOOT, 0x1);
    assert_eq!(mmu.io[FLAG_BOOT - IO_BEG], 1);

    mmu.unset_flag(FLAG_BOOT, 0x1);
    assert_eq!(mmu.io[FLAG_BOOT - IO_BEG], 0);
  }

  #[test]
  fn get_flag() {
    let mut mmu = instantiate_mmu!();
    assert_eq!(mmu.get_flag(FLAG_BOOT, 0x1), false);
    mmu.set_flag(FLAG_BOOT, 0x1);
    assert_eq!(mmu.get_flag(FLAG_BOOT, 0x1), true);
  }
}
