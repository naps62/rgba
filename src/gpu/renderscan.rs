extern crate rand;

use super::registers::{read, Reg};
use crate::{buffer, mmu};
use buffer::{random_pixel, Buffer};
use std::sync::Arc;

const TILEMAP_0_OFFSET: u32 = 0x1800;
// const TILEMAP_1_OFFSET: u32 = 0x1C00;

pub const VRAM_BEG: usize = 0x8000;

pub fn renderscan(buffer: &Arc<Buffer>, mmu: &dyn mmu::MMU) {
  let line = read(mmu, Reg::CurrentScanLine) as u32;
  let scroll_x = read(mmu, Reg::ScrollX) as u32;
  let scroll_y = read(mmu, Reg::ScrollY) as u32;

  // todo check if map 1 is to be used
  let map_offset = TILEMAP_0_OFFSET + ((line + scroll_y) & 255) >> 3;

  let mut line_offset = scroll_x >> 3;

  let _y = (line + scroll_y) & 7;
  let mut x = scroll_x & 7;

  let _canvas_offset = line * 160 * 4;

  let mut tile_index = mmu.read8(VRAM_BEG + (map_offset + line_offset) as usize);

  // todo
  // if (tile 1 && tile < 128) {tile_index +=256}

  for screen_x in 0..160 {
    let _tile_pixel = mmu.read16(tile_index as usize);

    buffer.put_pixel(screen_x, line as u32, random_pixel());

    x = x + 1;

    if x == 8 {
      x = 0;
      line_offset = (line_offset + 1) & 31;
      tile_index = mmu.read8(VRAM_BEG + (map_offset + line_offset) as usize);
      // todo
      // if (tile 1 && tile < 128) {tile_index +=256}
    }
  }
}
