use crate::opcodes::Instruction;
use crate::registers::Registers;

pub struct CPU {
  registers: Registers,
}

impl CPU {
  pub fn new() -> CPU {
    CPU {
      registers: Registers::new(),
    }
  }

  pub fn exec(&self, _instr: Instruction) {}
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::registers::Register16;

  #[test]
  fn new_cpu() {
    let cpu = CPU::new();

    assert_eq!(cpu.registers.read16(Register16::AF), 0);
  }
}
