mod opcodes;
mod registers;

use registers::Registers;

struct CPU {
  registers: Registers,
}

impl CPU {
  pub fn new() -> CPU {
    CPU {
      registers: Registers::new(),
    }
  }

  pub fn exec(&self, instr: opcodes::Instruction) {
    use opcodes::Instruction::*;

    // match instr {
    //   I_NOP => (),
    // }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn new_cpu() {
    let cpu = CPU::new();

    assert_eq!(cpu.registers.read16(registers::Register16::AF), 0);
  }
}
