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
  ($supervisor: ident msg_type($msg_type_name: tt) write($stack_value: path => $cast: ty)) => {
    match (arg!(1), arg!(2)) {
      (StackValue::Int(address), $stack_value(value)) =>
        $supervisor.memory_mut(|memory| memory.write(address as usize, &(value as $cast).to_le_bytes())),
      _ => panic!(concat!("expecting: address :: int, value :: ", stringify!($msg_type_name)))
    }
  };

  ($supervisor: ident read($stack_value: path, $read_type: ty => $cast: ty)) => {
    match arg!(1) {
      StackValue::Int(address) =>
        $stack_value(
          $supervisor.memory(|memory| <$read_type>::from_le_bytes(
            memory.read(address as usize, std::mem::size_of::<$read_type>()).try_into().unwrap()
          )) as $cast
        ),
      _ => panic!("expecting: address :: int")
    }
  };
}

pub trait ProcesSupervisor {
  fn set_memory(&mut self, unit: Option<usize>);
  fn memory<T>(&self, effect: impl FnOnce(&dyn Memory) -> T) -> T;
  fn memory_mut<T>(&mut self, effect: impl FnOnce(&mut dyn Memory) -> T) -> T;
  fn fork(&self, process: Process);
  fn invoke_ffi(&mut self, symbol: &[u8], process: &mut Process) -> Option<StackValue>;
}

pub type Registers = [StackValue; 10];

#[derive(Clone)]
pub struct Process {
  pub program: Program,
  pub pc: usize,
  pub stack: Stack,
  pub registers: Registers
}

impl Process {
  pub fn new(program: Program) -> Self {
    Process {
      program,
      pc: 0,
      stack: Stack::new(),
      registers: [StackValue::Int(0); 10]
    }
  }

  pub fn is_finished(&self) -> bool {
    self.pc >= self.program.instructions.len()
  }

  pub fn run_until_finish(&mut self, supervisor: &mut impl ProcesSupervisor) {
    while let Some(&instruction) = self.program.instructions.get(self.pc) {
      self.pc += 1;
      self.run_instruction(supervisor, instruction)
    }
  }

  pub fn run_instruction(&mut self, supervisor: &mut impl ProcesSupervisor, instruction: Instruction) {
    macro_rules! arg {
      ($idx: tt) => {
        instruction.$idx.and_then(|x| Some(x.into())).or_else(|| self.stack.pop())
          .expect(concat!("expecting argument ", $idx, " on line ", line!()))
      };
    }

    match instruction.0 {
      Opcode::Noop => (),

      Opcode::Add => {
        let (a, b) = (arg!(1), arg!(2));
        self.stack.push(same_type_op!(a + b));
      },
      Opcode::Sub => {
        let (a, b) = (arg!(1), arg!(2));
        self.stack.push(same_type_op!(a - b));
      },
      Opcode::Mul => {
        let (a, b) = (arg!(1), arg!(2));
        self.stack.push(same_type_op!(a * b));
      },
      Opcode::Div => {
        let (a, b) = (arg!(1), arg!(2));
        self.stack.push(same_type_op!(a / b));
      },

      Opcode::Gt => {
        let (a, b) = (arg!(1), arg!(2));
        self.stack.push(same_type_op!((StackValue::Int => i64) a > b));
      }
      Opcode::Ls => {
        let (a, b) = (arg!(1), arg!(2));
        self.stack.push(same_type_op!((StackValue::Int => i64) a < b));
      }
      Opcode::Gteq =>  {
        let (a, b) = (arg!(1), arg!(2));
        self.stack.push(same_type_op!((StackValue::Int => i64) a >= b));
      }
      Opcode::Lseq =>  {
        let (a, b) = (arg!(1), arg!(2));
        self.stack.push(same_type_op!((StackValue::Int => i64) a <= b));
      }
      Opcode::Eq =>  {
        let (a, b) = (arg!(1), arg!(2));
        self.stack.push(same_type_op!((StackValue::Int => i64) a == b));
      }
      Opcode::Noteq =>  {
        let (a, b) = (arg!(1), arg!(2));
        self.stack.push(same_type_op!((StackValue::Int => i64) a != b));
      }

      Opcode::Discard => { self.stack.pop(); }
      Opcode::Clone => if let Some(item) = self.stack.peek() { self.stack.push(item) }
      Opcode::Debug => println!("{:?}", self.stack),
      Opcode::Push => {
        let item = arg!(1);
        self.stack.push(item)
      }
      Opcode::Int => match arg!(1) {
        StackValue::Int(x) => self.stack.push(StackValue::Int(x)),
        StackValue::Float(x) => self.stack.push(StackValue::Int(x as i64)),
      }
      Opcode::Float => match arg!(1) {
        StackValue::Int(x) => self.stack.push(StackValue::Float(x as f64)),
        StackValue::Float(x) => self.stack.push(StackValue::Float(x)),
      }
      Opcode::Jump => match (arg!(1), arg!(2)) {
        (StackValue::Int(pc), StackValue::Int(cond)) => if cond != 0 {
          self.pc = pc as usize
        },
        _ => panic!("expecting: pc :: int, cond :: int")
      }
      Opcode::Swap => {
        let (a, b) = (arg!(1), arg!(2));
        self.stack.push(a);
        self.stack.push(b);
      },
      Opcode::Over => {
        let (a, b) = (arg!(1), arg!(2));
        self.stack.push(b);
        self.stack.push(a);
        self.stack.push(b);
      }

      Opcode::Reg => match arg!(1) {
        StackValue::Int(reg @ 0..=9) => self.stack.push(self.registers[reg as usize]),
        _ => panic!("expecting: reg :: int in [0, 10)")
      }
      Opcode::RegSet => match (arg!(1), arg!(2)) {
        (StackValue::Int(reg @ 0..=9), value) => self.registers[reg as usize] = value,
        _ => panic!("expecting: reg :: int in [0, 10)")
      }

      Opcode::WriteInt64 => mem!(supervisor msg_type(int) write(StackValue::Int => i64)),
      Opcode::ReadInt64 => {
        let value = mem!(supervisor read(StackValue::Int, i64 => i64));
        self.stack.push(value);
      }

      Opcode::WriteInt32 => mem!(supervisor msg_type(int) write(StackValue::Int => i32)),
      Opcode::ReadInt32 => {
        let value = mem!(supervisor read(StackValue::Int, i32 => i64));
        self.stack.push(value);
      }

      Opcode::WriteInt16 => mem!(supervisor msg_type(int) write(StackValue::Int => i16)),
      Opcode::ReadInt16 => {
        let value = mem!(supervisor read(StackValue::Int, i16 => i64));
        self.stack.push(value);
      }

      Opcode::WriteInt8 => mem!(supervisor msg_type(int) write(StackValue::Int => i8)),
      Opcode::ReadInt8 => {
        let value = mem!(supervisor read(StackValue::Int, i8 => i64));
        self.stack.push(value);
      }

      Opcode::WriteFloat64 => mem!(supervisor msg_type(float) write(StackValue::Float => f64)),
      Opcode::ReadFloat64 => {
        let value = mem!(supervisor read(StackValue::Float, f64 => f64));
        self.stack.push(value);
      }

      Opcode::WriteFloat32 => mem!(supervisor msg_type(float) write(StackValue::Float => f32)),
      Opcode::ReadFloat32 => {
        let value = mem!(supervisor read(StackValue::Float, f32 => f64));
        self.stack.push(value);
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
      Opcode::Invoke => match (arg!(1), arg!(2)) {
        (StackValue::Int(address), StackValue::Int(size)) => {
          let outcome: Vec<u8> = supervisor.memory(|memory| memory.read(address as usize, size as usize).into());
          if let Some(value) = supervisor.invoke_ffi(&outcome, self) {
            self.stack.push(value);
          }
        },
        _ => panic!("expecting: address :: int, size :: int")
      },
    };
  }
}

impl From<Program> for Process {
  fn from(program: Program) -> Self {
    Process::new(program)
  }
}