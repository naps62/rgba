extern crate image;
extern crate rand;

use std::sync::Arc;

use crate::{buffer, mmu};
use buffer::Buffer;
use image::Rgba;
use mmu::{addrs::Addr, addrs::LCDControlReg, MMU};

const TILEMAP_0_OFFSET: u32 = 0x1800;
const TILEMAP_1_OFFSET: u32 = 0x1C00;

pub const VRAM_BEG: usize = 0x8000;

pub fn renderscan<M: MMU>(buffer: &Arc<Buffer>, mmu: &mut M) {
  let line = mmu.read8(Addr::CurrentScanLine) as u32;
  let scroll_x = mmu.read8(Addr::ScrollX) as u32;
  let scroll_y = mmu.read8(Addr::ScrollY) as u32;
  let bg_map = mmu.get_flag(Addr::LCDControl, LCDControlReg::BGTileMap);
  // let bg_set = mmu.get_flag(Addr::LCDControl, LCDControlReg::BGTileSet);

  let mut map_offset = if bg_map {
    TILEMAP_1_OFFSET
  } else {
    TILEMAP_0_OFFSET
  };
  map_offset = map_offset + ((line + scroll_y) & 255) >> 3;

  let mut line_offset = scroll_x >> 3;

  let _y = (line + scroll_y) & 7;
  let mut x = scroll_x & 7;

  let _canvas_offset = line * 160 * 4;

  // todo
  // if (tile 1 && tile < 128) {tile_index +=256}
  // todo check if map 1 is to be used
  // if bg_set && tile_index < 128 {
  //   tile_index += 256
  // }

  let mut tile_addr = VRAM_BEG + (map_offset + line_offset) as usize;
  let mut tile_row = mmu.read16(tile_addr);
  if tile_row != 0x0 {
    println!("{:b}", tile_row);
  }

  for screen_x in 0..160 {
    buffer.put_pixel(screen_x, line as u32, Rgba::<u8>([127, 0, 0, 255]));

    x = x + 1;

    if x == 8 {
      x = 0;
      line_offset = (line_offset + 1) & 31;
      tile_addr = VRAM_BEG + (map_offset + line_offset) as usize;
      tile_row = mmu.read16(tile_addr);
      if tile_row != 0x0 {
        println!("{:b}", tile_row);
      }

      // todo
      // if (tile 1 && tile < 128) {tile_index +=256}
    }
  }
}
