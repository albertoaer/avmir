use core::fmt;

use super::{memory::Memory, program::{InstructionParam, Opcode, Program}};

#[derive(Clone, Debug, Copy)]
pub enum StackValue {
  Int(i64),
  Float(f64)
}

macro_rules! same_type_op {
  ($a: ident $op: tt $b: ident) => {
    match ($a, $b) {
      (StackValue::Int(x), StackValue::Int(y)) => StackValue::Int(x $op y),
      (StackValue::Float(x), StackValue::Float(y)) => StackValue::Float(x $op y),
      _ => panic!("operands must be same type")
    }
  };

  (($output: path => $cast: ty) $a: ident $op: tt $b: ident) => {
    match ($a, $b) {
      (StackValue::Int(x), StackValue::Int(y)) => $output((x $op y) as $cast),
      (StackValue::Float(x), StackValue::Float(y)) => $output((x $op y) as $cast),
      _ => panic!("operands must be same type")
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

#[derive(Clone)]
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

pub struct Process {
  program: Program,
  pc: usize,
  stack: Stack,
  external_unit: Option<usize>
}

impl Process {
  pub fn new(program: Program) -> Self {
    Process {
      program,
      pc: 0,
      stack: Stack::new(),
      external_unit: None
    }
  }

  pub fn external_unit(&self) -> Option<usize> {
    self.external_unit
  }

  pub fn run(&mut self, memory: &mut Memory) -> bool {
    let instruction = &self.program[self.pc];
    self.pc += 1;

    let stack = &mut self.stack;

    macro_rules! arg {
      ($idx: tt) => {
        instruction.$idx.and_then(|x| Some(x.into())).or_else(|| stack.pop())
          .expect(concat!("expecting argument ", $idx, " on line ", line!()))
      };
    }

    match instruction.0 {
      Opcode::Add => {
        let (a, b) = (arg!(1), arg!(2));
        stack.push(same_type_op!(a + b));
      },
      Opcode::Sub => {
        let (a, b) = (arg!(1), arg!(2));
        stack.push(same_type_op!(a - b));
      },
      Opcode::Mul => {
        let (a, b) = (arg!(1), arg!(2));
        stack.push(same_type_op!(a * b));
      },
      Opcode::Div => {
        let (a, b) = (arg!(1), arg!(2));
        stack.push(same_type_op!(a / b));
      },

      Opcode::Gt => {
        let (a, b) = (arg!(1), arg!(2));
        stack.push(same_type_op!((StackValue::Int => i64) a > b));
      }
      Opcode::Ls => {
        let (a, b) = (arg!(1), arg!(2));
        stack.push(same_type_op!((StackValue::Int => i64) a < b));
      }
      Opcode::Gteq =>  {
        let (a, b) = (arg!(1), arg!(2));
        stack.push(same_type_op!((StackValue::Int => i64) a >= b));
      }
      Opcode::Lseq =>  {
        let (a, b) = (arg!(1), arg!(2));
        stack.push(same_type_op!((StackValue::Int => i64) a <= b));
      }
      Opcode::Eq =>  {
        let (a, b) = (arg!(1), arg!(2));
        stack.push(same_type_op!((StackValue::Int => i64) a == b));
      }
      Opcode::Noteq =>  {
        let (a, b) = (arg!(1), arg!(2));
        stack.push(same_type_op!((StackValue::Int => i64) a != b));
      }

      Opcode::Discard => { stack.pop(); },
      Opcode::Clone => if let Some(item) = stack.peek() { stack.push(item) },
      Opcode::Debug => println!("{:?}", stack),
      Opcode::Push => {
        let item = arg!(1);
        stack.push(item)
      },
      Opcode::Int => match arg!(1) {
        StackValue::Int(x) => stack.push(StackValue::Int(x)),
        StackValue::Float(x) => stack.push(StackValue::Int(x as i64)),
      },
      Opcode::Float => match arg!(1) {
        StackValue::Int(x) => stack.push(StackValue::Float(x as f64)),
        StackValue::Float(x) => stack.push(StackValue::Float(x)),
      },
      Opcode::Jump => match (arg!(1), arg!(2)) {
        (StackValue::Int(pc), StackValue::Int(cond)) if pc >= 0 => if cond != 0 {
          self.pc = pc as usize
        },
        _ => panic!("expecting: pc :: int >= 0, cond :: int")
      },
      Opcode::Swap => {
        let (a, b) = (arg!(1), arg!(2));
        stack.push(a);
        stack.push(b);
      },
      Opcode::Over => {
        let (a, b) = (arg!(1), arg!(2));
        stack.push(b);
        stack.push(a);
        stack.push(b);
      },
        
      Opcode::WriteInt64 => match (arg!(1), arg!(2)) {
        (StackValue::Int(address), StackValue::Int(value)) =>
          memory.write_int_64(value, address as usize),
        _ => panic!("expecting: address :: int, value :: int")
      }
      Opcode::ReadInt64 => match arg!(1) {
        StackValue::Int(address) => stack.push(StackValue::Int(memory.read_int_64(address as usize))),
        _ => panic!("expecting: address :: int")
      }
      Opcode::Mount => match arg!(1) {
        StackValue::Int(unit) if unit >= 0 => self.external_unit = Some(unit as usize),
        _ => panic!("expecting: unit :: int >= 0")
      }
      Opcode::Unmount => self.external_unit = None,
    };

    return self.pc < self.program.len();
  }
}