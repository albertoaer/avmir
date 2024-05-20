#[derive(Clone, Debug, Copy)]
pub enum Opcode {
  Add,
  Sub,
  Mul,
  Div,
  Discard,
  Clone
}

#[derive(Clone, Debug, Copy)]
pub enum InstructionParam {
  Int(i64),
  Float(f64),
}

#[derive(Clone, Debug, Copy)]
pub struct Instruction(pub Opcode, pub Option<InstructionParam>, pub Option<InstructionParam>);

impl Instruction {
  fn new(opcode: Opcode) -> Instruction {
    Instruction(opcode, None, None)
  }

  fn new_args(opcode: Opcode, first: Option<InstructionParam>, second: Option<InstructionParam>) -> Instruction {
    Instruction(opcode, first, second)
  }
}

pub type Program = Vec<Instruction>;