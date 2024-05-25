use super::{memory::Memory, program::{Instruction, Opcode, Program}, stack::{Stack, StackValue}};

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

macro_rules! mem {
  ($supervisor: ident msg_type($msg_type_name: tt) write($stack_value: path => $cast: ty) $func: tt) => {
    match (arg!(1), arg!(2)) {
      (StackValue::Int(address), $stack_value(value)) =>
        $supervisor.memory_mut(|memory| memory.$func(value as $cast, address as usize)),
      _ => panic!(concat!("expecting: address :: int, value :: ", stringify!($msg_type_name)))
    }
  };

  ($supervisor: ident read($stack_value: path, $cast: ty) $func: tt) => {
    match arg!(1) {
      StackValue::Int(address) =>
        $stack_value($supervisor.memory(|memory| memory.$func(address as usize)) as $cast),
      _ => panic!("expecting: address :: int")
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

  pub fn run_until_finish(&mut self, supervisor: &mut impl ProcesSupervisor) {
    while let Some(&instruction) = self.program.instructions.get(self.pc) {
      self.pc += 1;
      self.run_current_instruction(supervisor, instruction)
    }
  }

  pub fn run_current_instruction(&mut self, supervisor: &mut impl ProcesSupervisor, instruction: Instruction) {
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
        
      Opcode::WriteInt64 => mem!(supervisor msg_type(int) write(StackValue::Int => i64) write_int_64),
      Opcode::ReadInt64 => {
        let value = mem!(supervisor read(StackValue::Int, i64) read_int_64);
        stack.push(value);
      }

      Opcode::WriteInt32 => mem!(supervisor msg_type(int) write(StackValue::Int => i32) write_int_32),
      Opcode::ReadInt32 => {
        let value = mem!(supervisor read(StackValue::Int, i64) read_int_32);
        stack.push(value);
      }

      Opcode::WriteInt16 => mem!(supervisor msg_type(int) write(StackValue::Int => i16) write_int_16),
      Opcode::ReadInt16 => {
        let value = mem!(supervisor read(StackValue::Int, i64) read_int_16);
        stack.push(value);
      }

      Opcode::WriteInt8 => mem!(supervisor msg_type(int) write(StackValue::Int => i8) write_int_8),
      Opcode::ReadInt8 => {
        let value = mem!(supervisor read(StackValue::Int, i64) read_int_8);
        stack.push(value);
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
  }
}

impl From<Program> for Process {
  fn from(program: Program) -> Self {
    Process::new(program)
  }
}