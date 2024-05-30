use std::{thread, time::Duration};

use super::{instruction::ProcessInstruction, memory::MemoryHandler, program::{Instruction, Opcode, Program}, stack::{Stack, StackValue}};

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
    match arg!(both) {
      (StackValue::Int(address), $stack_value(value)) =>
        $supervisor.get_memory().memory_mut(|memory| memory.write(address as usize, &(value as $cast).to_le_bytes())),
      _ => panic!(concat!("expecting: address :: int, value :: ", stringify!($msg_type_name)))
    }
  };

  ($supervisor: ident read($stack_value: path, $read_type: ty => $cast: ty)) => {
    match arg!(first) {
      StackValue::Int(address) =>
        $stack_value(
          $supervisor.get_memory().memory(|memory| <$read_type>::from_le_bytes(
            memory.read(address as usize, std::mem::size_of::<$read_type>()).try_into().unwrap()
          )) as $cast
        ),
      _ => panic!("expecting: address :: int")
    }
  };
}

pub trait ProcesSupervisor {
  fn set_memory(&mut self, unit: Option<usize>);
  fn get_memory(&mut self) -> MemoryHandler;
  fn fork(&self, process: Process);
  fn invoke_ffi(&mut self, symbol: &[u8], process: &mut Process) -> Option<StackValue>;
}

pub const PUBLIC_REGISTERS_COUNT: usize = 10;
pub const SPECIAL_REGISTERS_COUNT: usize = 4;
pub const PRIVATE_REGISTERS_COUNT: usize = 10;
pub const PROCESS_REGISTERS_COUNT: usize = PUBLIC_REGISTERS_COUNT + SPECIAL_REGISTERS_COUNT + PRIVATE_REGISTERS_COUNT;

pub const REGISTER_FLAG_SHARE_MEMORY: usize = 0;
pub const REGISTER_FLAG_INVOKE_TRAP: usize = 1;

pub type PublicRegisters = [StackValue; PUBLIC_REGISTERS_COUNT];
pub type ProcessRegisters = [StackValue; PROCESS_REGISTERS_COUNT];

#[derive(Clone)]
pub struct Process {
  pub program: Program,
  pub instructions: Vec<ProcessInstruction>,
  pub pc: usize,
  pub stack: Stack,
  pub registers: ProcessRegisters,
  pub invoke_target: Vec<u8>
}

impl Process {
  pub fn new(program: Program) -> Self {
    let instructions = program.instructions.iter().map(|x| (*x).into()).collect();

    Process {
      program,
      instructions,
      pc: 0,
      stack: Stack::new(),
      registers: [StackValue::Int(0); PROCESS_REGISTERS_COUNT],
      invoke_target: vec![]
    }
  }

  pub fn is_finished(&self) -> bool {
    self.pc >= self.program.instructions.len()
  }

  pub fn get_flag_share_memory(&self) -> bool {
    let value: usize = self.registers[
      PUBLIC_REGISTERS_COUNT + REGISTER_FLAG_SHARE_MEMORY
    ].into();
    value != 0
  }

  pub fn get_flag_invoke_trap(&self) -> bool {
    let value: usize = self.registers[
      PUBLIC_REGISTERS_COUNT + REGISTER_FLAG_INVOKE_TRAP
    ].into();
    value != 0
  }

  pub fn get_current_instruction(&self) -> Option<&Instruction> {
    self.program.instructions.get(self.pc)
  }

  pub fn run_next(&mut self, supervisor: &mut dyn ProcesSupervisor) -> bool {
    if let Some(&instruction) = self.instructions.get(self.pc) {
      self.pc += 1;
      self.run_instruction(supervisor, instruction);
      true
    } else {
      false
    }
  }

  pub fn run_until_finish(&mut self, supervisor: &mut dyn ProcesSupervisor) {
    while let Some(&instruction) = self.instructions.get(self.pc) {
      self.pc += 1;
      self.run_instruction(supervisor, instruction)
    }
  }

  pub fn run_instruction(&mut self, supervisor: &mut dyn ProcesSupervisor, instruction: ProcessInstruction) {
    macro_rules! expect_arg_stack {
      (both) => {
        self.stack.pop2().expect(concat!("expecting arguments 1 & 2 on line ", line!()))
      };
      
      ($idx: tt) => { // idx is just for the error message
        self.stack.pop().expect(concat!("expecting argument ", $idx, " on line ", line!()))
      };
    }

    macro_rules! arg {
      (both) => {
        match instruction.operands {
          (Some(a), Some(b)) => (a, b),
          (Some(a), None) => (a, expect_arg_stack!(2)),
          (None, Some(b)) => (expect_arg_stack!(1), b),
          (None, None) => expect_arg_stack!(both),
        }
      };

      (first) => {
        match instruction.operands.0 {
          Some(value) => value,
          _ => expect_arg_stack!(0)
        }
      };
    }

    match instruction.opcode {
      Opcode::Noop => (),
      Opcode::Debug => println!("{:?}", self.stack),

      Opcode::Add => {
        let (a, b) = arg!(both);
        self.stack.push(same_type_op!(a + b));
      },
      Opcode::Sub => {
        let (a, b) = arg!(both);
        self.stack.push(same_type_op!(a - b));
      },
      Opcode::Mul => {
        let (a, b) = arg!(both);
        self.stack.push(same_type_op!(a * b));
      },
      Opcode::Div => {
        let (a, b) = arg!(both);
        self.stack.push(same_type_op!(a / b));
      },

      Opcode::Gt => {
        let (a, b) = arg!(both);
        self.stack.push(same_type_op!((StackValue::Int => i64) a > b));
      }
      Opcode::Ls => {
        let (a, b) = arg!(both);
        self.stack.push(same_type_op!((StackValue::Int => i64) a < b));
      }
      Opcode::Gteq =>  {
        let (a, b) = arg!(both);
        self.stack.push(same_type_op!((StackValue::Int => i64) a >= b));
      }
      Opcode::Lseq =>  {
        let (a, b) = arg!(both);
        self.stack.push(same_type_op!((StackValue::Int => i64) a <= b));
      }
      Opcode::Eq =>  {
        let (a, b) = arg!(both);
        self.stack.push(same_type_op!((StackValue::Int => i64) a == b));
      }
      Opcode::Noteq =>  {
        let (a, b) = arg!(both);
        self.stack.push(same_type_op!((StackValue::Int => i64) a != b));
      }

      Opcode::Int => {
        let value = arg!(first).into();
        self.stack.push(StackValue::Int(value))
      }
      Opcode::Float => {
        let value = arg!(first).into();
        self.stack.push(StackValue::Float(value))
      }

      Opcode::Discard => { self.stack.pop(); }
      Opcode::Clone => if let Some(item) = self.stack.peek() { self.stack.push(item) }
      Opcode::Push => {
        if let Some(item) = instruction.operands.0 {
          self.stack.push(item)
        }
        if let Some(item) = instruction.operands.1 {
          self.stack.push(item)
        }
      }
      Opcode::Swap => {
        let (a, b) = arg!(both);
        self.stack.push(a);
        self.stack.push(b);
      },
      Opcode::Over => {
        let (a, b) = arg!(both);
        self.stack.push(b);
        self.stack.push(a);
        self.stack.push(b);
      }

      Opcode::Reg => match arg!(first) {
        StackValue::Int(reg) => self.stack.push(self.registers[reg as usize]),
        _ => panic!("expecting: reg :: int")
      }
      Opcode::SetReg => match arg!(both) {
        (StackValue::Int(reg), value) => self.registers[reg as usize] = value,
        _ => panic!("expecting: registry :: int, value :: any")
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

      Opcode::Mount => match arg!(first) {
        StackValue::Int(unit) if unit >= 0 => supervisor.set_memory(Some(unit as usize)),
        _ => panic!("expecting: unit :: int >= 0")
      }
      Opcode::Unmount => supervisor.set_memory(None),
      
      Opcode::Jump => match arg!(both) {
        (StackValue::Int(pc), StackValue::Int(cond)) => if cond != 0 {
          self.pc = pc as usize
        },
        _ => panic!("expecting: pc :: int, cond :: int")
      }
      Opcode::Fork => match arg!(first) {
        StackValue::Int(pc) => {
          let mut cloned = self.clone();
          cloned.pc = pc as usize;
          supervisor.fork(cloned)
        },
        _ => panic!("expecting: pc :: int")
      }
      Opcode::Exit => self.pc = self.instructions.len(),
      Opcode::ThreadSleep => match arg!(first) {
        StackValue::Int(millis) => thread::sleep(Duration::from_millis(millis as u64)),
        _ => panic!("expecting: millis :: int")
      }

      Opcode::PrepareInvoke => match arg!(both) {
        (StackValue::Int(address), StackValue::Int(size)) => {
          self.invoke_target = supervisor.get_memory()
            .memory(|memory| memory.read(address as usize, size as usize).into());
        },
        _ => panic!("expecting: address :: int, size :: int")
      },
      Opcode::Invoke => {
        let invoke_target = self.invoke_target.clone();
        if let Some(value) = supervisor.invoke_ffi(&invoke_target, self) {
          self.stack.push(value);
        }
      },
      Opcode::FastInvoke => match arg!(both) {
        (StackValue::Int(address), StackValue::Int(size)) => {
          let invoke_target: Vec<_> = supervisor.get_memory()
            .memory(|memory| memory.read(address as usize, size as usize).into());
          self.invoke_target = invoke_target.clone();
          if let Some(value) = supervisor.invoke_ffi(&invoke_target, self) {
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