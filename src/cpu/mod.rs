mod memory;
pub mod opcodes;
pub mod registers;

use opcodes::Instruction;
use ram::RAM;
use registers::{Register16, Registers};

pub struct CPU {
  registers: Registers,
  ram: Memory,
}

impl CPU {
  pub fn new() -> CPU {
    CPU {
      registers: Registers::new(),
      ram: Memory::new(8 * 1024),
    }
  }

  pub fn exec(&self, _instr: Instruction) {
    let current_pc = self.registers.read16(Register16::PC);

    let new_pc = current_pc;

    self.registers.write16(Register16::PC, new_pc);
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use registers::Register16;

  #[test]
  fn new_cpu() {
    let cpu = CPU::new();

    assert_eq!(cpu.registers.read16(Register16::AF), 0);
  }
}
