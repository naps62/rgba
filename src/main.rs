mod cpu;
mod opcodes;
mod registers;

use cpu::CPU;
use opcodes::{Arg::*, Instruction::*};
use registers::Register8;

fn main() {
  let cpu = CPU::new();

  cpu.exec(I_LD(R8(Register8::A), R8(Register8::B)));

  println!("Hello")
}
