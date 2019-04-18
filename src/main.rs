mod cpu;

use cpu::opcodes::{Arg::*, Instruction::*};
use cpu::registers::Register8;
use cpu::CPU;

fn main() {
  let cpu = CPU::new();

  cpu.exec(I_LD(R8(Register8::A), R8(Register8::B)));

  println!("Hello")
}
