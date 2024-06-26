use std::fmt::Display;

use strum_macros::{Display, EnumString};

#[derive(Clone, Debug, Copy, EnumString, Display)]
pub enum Opcode {
  Noop, // does nothing
  Debug, // print the stack

  Add, // +
  Sub, // -
  Mul, // *
  Div, // /

  Gt, // >
  Ls, // <
  Gteq, // >=
  Lseq, // <=
  Eq, // ==
  Noteq, // !=

  Int, // any => int
  Float, // any => float

  Discard, // a =>
  Clone, // a => a a
  Push, // => a
  Swap, // a b => b a
  Over, // a b => a b a

  Reg, // i; reg[i]
  SetReg, // i a; reg[i] = a

  WriteInt64,
  ReadInt64,

  WriteInt32,
  ReadInt32,
  
  WriteInt16,
  ReadInt16,
  
  WriteInt8,
  ReadInt8,

  WriteFloat64,
  ReadFloat64,

  WriteFloat32,
  ReadFloat32,

  Mount, // set the shared memory as active memory
  Unmount, // set the process memory as active memory

  Jump, // a b; pc = a if b != 0
  Fork, // a; spawn a clone process with pc = a
  Exit, // pc = last instruction + 1
  ThreadSleep, // a; sleeps the current thread 'a' milliseconds

  PrepareInvoke, // a b; invoke_target = [a..(a + b)]
  Invoke, // invoke invoke_target
  FastInvoke, // a b; PrepareInvoke + Invoke
  
  Pid // push the process id onto the stack
}

#[derive(Clone, Debug, Copy)]
pub enum InstructionParam {
  Int(i64),
  Float(f64),
}

impl Display for InstructionParam {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      InstructionParam::Int(x) => write!(f, "{}", x),
      InstructionParam::Float(x) => write!(f, "{}", x),
    }
  }
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

impl Display for Instruction {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.0)?;
    match (self.1, self.2) {
      (None, None) => Ok(()),
      (None, Some(b)) => write!(f, " _ {}", b),
      (Some(a), None) => write!(f, " {}", a),
      (Some(a), Some(b)) => write!(f, " {} {}", a, b),
    }
  }
}

const DEFAULT_PROGRAM_MEMORY: usize = 1024;

#[derive(Debug, Clone)]
pub struct Program {
  pub name: String,
  pub instructions: Vec<Instruction>,
  pub static_data: Vec<u8>,
  pub static_data_meta: Vec<(usize, usize)>,
  pub required_memory: usize
}

impl Program {
  pub fn new() -> Program {
    Program {
      name: String::new(),
      instructions: Vec::new(),
      static_data: Vec::new(),
      static_data_meta: Vec::new(),
      required_memory: DEFAULT_PROGRAM_MEMORY
    }
  }

  pub fn with_name(name: impl AsRef<str>) -> Program {
    let mut program = Self::new();
    program.name = name.as_ref().into();
    program
  }

  pub fn memory(&self) -> Vec<u8> {
    let mut memory = self.static_data.clone();
    if memory.len() < self.required_memory {
      memory.resize(self.required_memory, 0);
    }
    memory
  }
}