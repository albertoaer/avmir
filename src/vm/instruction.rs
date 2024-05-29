use super::{program::{Instruction, Opcode}, stack::StackValue};

#[derive(Copy, Clone)]
pub struct ProcessInstruction {
  pub opcode: Opcode,
  pub operands: (Option<StackValue>, Option<StackValue>),
}

impl From<Instruction> for ProcessInstruction {
  fn from(value: Instruction) -> Self {
    ProcessInstruction {
      opcode: value.0,
      operands: (value.1.map(|x| x.into()), value.2.map(|x| x.into()))
    }
  }
}