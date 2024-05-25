use super::{memory::Memory, program::{Opcode, Program}, stack::{Stack, StackValue}};

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

pub trait ProcesSupervisor {
  fn set_memory(&mut self, unit: Option<usize>);
  fn memory<T>(&self, effect: impl FnOnce(&dyn Memory) -> T) -> T;
  fn memory_mut<T>(&mut self, effect: impl FnOnce(&mut dyn Memory) -> T) -> T;
  fn fork(&self, process: Process);
}

#[derive(Clone)]
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

  pub fn is_finished(&self) -> bool {
    self.pc >= self.program.instructions.len()
  }

  pub fn run(&mut self, supervisor: &mut impl ProcesSupervisor) -> bool {
    let instruction = &self.program.instructions[self.pc];
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
          supervisor.memory_mut(|memory| memory.write_int_64(value, address as usize)),
        _ => panic!("expecting: address :: int, value :: int")
      }
      Opcode::ReadInt64 => match arg!(1) {
        StackValue::Int(address) =>
          stack.push(StackValue::Int(supervisor.memory(|memory| memory.read_int_64(address as usize)))),
        _ => panic!("expecting: address :: int")
      }
      Opcode::Mount => match arg!(1) {
        StackValue::Int(unit) if unit >= 0 => supervisor.set_memory(Some(unit as usize)),
        _ => panic!("expecting: unit :: int >= 0")
      }
      Opcode::Unmount => supervisor.set_memory(None),
      Opcode::Fork => match arg!(1) {
        StackValue::Int(pc_offset) => {
          let mut cloned = self.clone();
          cloned.pc = (cloned.pc as i64 + pc_offset) as usize;
          supervisor.fork(cloned)
        },
        _ => panic!("expecting: pc_offset :: int")
      }
    };

    self.is_finished()
  }
}

impl From<Program> for Process {
  fn from(program: Program) -> Self {
    Process::new(program)
  }
}