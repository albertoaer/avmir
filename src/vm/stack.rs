use core::fmt;

use super::program::InstructionParam;


#[derive(Clone, Debug, Copy)]
pub enum StackValue {
  Int(i64),
  Float(f64)
}

impl From<InstructionParam> for StackValue {
  fn from(value: InstructionParam) -> Self {
    match value {
      InstructionParam::Int(x) => Self::Int(x),
      InstructionParam::Float(x) => Self::Float(x),
    }
  }
}

impl fmt::Display for StackValue {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      StackValue::Int(x) => x.fmt(f),
      StackValue::Float(x) => x.fmt(f),
    }
  }
}

macro_rules! stack_value_cast_into {
  ($t: ty) => {
    impl Into<$t> for StackValue {
      fn into(self) -> $t {
        match self {
          Self::Int(x) => x as $t,
          Self::Float(x) => x as $t,
        }
      }
    }
  };
}

stack_value_cast_into!(usize);
stack_value_cast_into!(i64);
stack_value_cast_into!(f64);

#[derive(Copy, Clone)]
pub struct Stack {
  items: [StackValue; 32],
  offset: u8
}

impl fmt::Debug for Stack {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_list().entries(self.items.iter().take(self.offset as usize)).finish()
  }
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

  pub fn pop2(&mut self) -> Option<(StackValue, StackValue)> {
    if self.offset < 2 {
      None
    } else {
      self.offset -= 2;
      Some((self.items[(self.offset + 1) as usize], self.items[self.offset as usize]))
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
    self.items[self.offset as usize] = value;
    self.offset += 1
  }
}