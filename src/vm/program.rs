use strum_macros::EnumString;

#[derive(Clone, Debug, Copy, EnumString)]
pub enum Opcode {
  Add,
  Sub,
  Mul,
  Div,

  Gt,
  Ls,
  Gteq,
  Lseq,
  Eq,
  Noteq,

  Discard,
  Clone,
  Debug,
  Push,
  Int,
  Float,
  Jump,
  Swap,
  Over,

  WriteInt64,
  ReadInt64,
  Mount,
  Unmount
}

#[derive(Clone, Debug, Copy)]
pub enum InstructionParam {
  Int(i64),
  Float(f64),
}

#[derive(Clone, Debug, Copy)]
pub struct Instruction(pub Opcode, pub Option<InstructionParam>, pub Option<InstructionParam>);

impl Instruction {
  pub fn new(opcode: Opcode) -> Self {
    Instruction(opcode, None, None)
  }

  pub fn new_args(opcode: Opcode, first: Option<InstructionParam>, second: Option<InstructionParam>) -> Instruction {
    Instruction(opcode, first, second)
  }
}

pub type Program = Vec<Instruction>;