use super::{stack::{Stack, StackValue}, memory::Memory, program::{Opcode, Program}};

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
  fn memory<T>(&mut self, effect: impl FnOnce(&mut Memory) -> T) -> T;
}

pub struct Process<'a, T> {
  program: Program,
  pc: usize,
  stack: Stack,
  supervisor: &'a mut T
}

impl<'a, T: ProcesSupervisor> Process<'a, T> {
  pub fn new(program: Program, supervisor: &'a mut T) -> Self {
    Process {
      program,
      pc: 0,
      stack: Stack::new(),
      supervisor
    }
  }

  pub fn run(&mut self) -> bool {
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
          self.supervisor.memory(|memory| memory.write_int_64(value, address as usize)),
        _ => panic!("expecting: address :: int, value :: int")
      }
      Opcode::ReadInt64 => match arg!(1) {
        StackValue::Int(address) =>
          stack.push(StackValue::Int(self.supervisor.memory(|memory| memory.read_int_64(address as usize)))),
        _ => panic!("expecting: address :: int")
      }
      Opcode::Mount => match arg!(1) {
        StackValue::Int(unit) if unit >= 0 => self.supervisor.set_memory(Some(unit as usize)),
        _ => panic!("expecting: unit :: int >= 0")
      }
      Opcode::Unmount => self.supervisor.set_memory(None),
    };

    return self.pc < self.program.len();
  }
}