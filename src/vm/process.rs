use super::program::{InstructionParam, Program};

#[derive(Clone, Debug, Copy)]
pub enum StackValue {
  Int(i64),
  Float(f64)
}

macro_rules! same_type {
  ($op: tt => $a: expr, $b: expr) => {
    match ($a, $b) {
      (StackValue::Int(x), StackValue::Int(y)) => StackValue::Int(x $op y),
      (StackValue::Float(x), StackValue::Float(y)) => StackValue::Float(x $op y),
      _ => panic!("Unexpected case")
    }
  };
}

impl From<InstructionParam> for StackValue {
  fn from(value: InstructionParam) -> Self {
    match value {
      InstructionParam::Int(x) => Self::Int(x),
      InstructionParam::Float(x) => Self::Float(x),
    }
  }
}

#[derive(Clone, Debug)]
pub struct Stack {
  items: [StackValue; 32],
  offset: u8
}

impl Stack {
  pub fn new() -> Self {
    Stack {
      items: [StackValue::Int(0); 32],
      offset: 0, 
    }
  }

  pub fn pop(&mut self) -> Option<StackValue> {
    if self.offset == 0 {
      None
    } else {
      self.offset -= 1;
      Some(self.items[self.offset as usize])
    }
  }
  
  pub fn peek(&mut self) -> Option<StackValue> {
    if self.offset == 0 {
      None
    } else {
      Some(self.items[(self.offset - 1) as usize])
    }
  }

  pub fn push(&mut self, value: StackValue) {
    if self.offset == 32 {
      panic!("stack overflow")
    }
    self.items[self.offset as usize] = value
  }
}

pub struct Process {
  program: Program,
  pc: usize,
  stack: Stack
}

impl Process {
  pub fn new(program: Program) -> Self {
    Process {
      program,
      pc: 0,
      stack: Stack::new()
    }
  }

  pub fn run(&mut self) -> bool {
    let instruction = &self.program[self.pc];

    let stack = &mut self.stack;

    let first = |stack: &mut Stack| instruction.1.and_then(|x| Some(x.into())).or_else(|| stack.pop())
      .expect("expected first operand");
    let second = |stack: &mut Stack| instruction.1.and_then(|x| Some(x.into())).or_else(|| stack.pop())
      .expect("expected second operand");

    match instruction.0 {
      super::program::Opcode::Add => {
        let (a, b) = (first(stack), second(stack));
        stack.push(same_type!(+ => a, b));
      },
      super::program::Opcode::Sub => {
        let (a, b) = (first(stack), second(stack));
        stack.push(same_type!(- => a, b));
      },
      super::program::Opcode::Mul => {
        let (a, b) = (first(stack), second(stack));
        stack.push(same_type!(* => a, b));
      },
      super::program::Opcode::Div => {
        let (a, b) = (first(stack), second(stack));
        stack.push(same_type!(/ => a, b));
      },
      super::program::Opcode::Discard => { stack.pop(); },
      super::program::Opcode::Clone => if let Some(item) = stack.peek() { stack.push(item) },
    };

    return self.pc >= self.program.len();
  }
}