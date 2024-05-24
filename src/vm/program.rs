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
  Unmount,
  Fork
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

  pub fn with_args(opcode: Opcode, first: Option<InstructionParam>, second: Option<InstructionParam>) -> Instruction {
    Instruction(opcode, first, second)
  }
}

const DEFAULT_PROGRAM_MEMORY: usize = 1024;

#[derive(Debug, Clone)]
pub struct Program {
  pub instructions: Vec<Instruction>,
  pub static_data: Vec<u8>,
  pub static_data_meta: Vec<(usize, usize)>,
  pub required_memory: usize
}

impl Program {
  pub fn new() -> Program {
    Program {
      instructions: Vec::new(),
      static_data: Vec::new(),
      static_data_meta: Vec::new(),
      required_memory: DEFAULT_PROGRAM_MEMORY
    }
  }

  pub fn memory(&self) -> Vec<u8> {
    let mut memory = self.static_data.clone();
    if memory.len() < self.required_memory {
      memory.resize(self.required_memory, 0);
    }
    memory
  }
}