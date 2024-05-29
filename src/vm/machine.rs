use std::{sync::{Arc, Condvar, Mutex, RwLock}, thread::{self, JoinHandle}};

use super::{
  ffi::{invoke_ffi, invoke_ffi_memory, invoke_ffi_trap, FFILoader},
  memory::{Memory, MemoryHandler},
  process::{ProcesSupervisor, Process, PUBLIC_REGISTERS_COUNT},
  program::Program, stack::StackValue
};

struct MachineInternal {
  active: (Mutex<usize>, Condvar),
  buffers: Vec<Arc<RwLock<dyn Memory>>>,
  ffi: Vec<FFILoader>
}

impl MachineInternal {
  fn new() -> Self {
    MachineInternal {
      active: (Mutex::new(0), Condvar::new()),
      buffers: vec![],
      ffi: vec![]
    }
  }

  pub fn add_memory(&mut self, memory: impl Memory + 'static) {
    self.buffers.push(Arc::new(RwLock::new(memory)))
  }

  pub fn add_ffi_loader(&mut self, loader: FFILoader) {
    self.ffi.push(loader)
  }
}

struct MachineProcessSupervisor {
  machine: Arc<MachineInternal>,
  memory: Vec<u8>,
  external_memory: Option<Arc<RwLock<dyn Memory>>>
}

impl MachineProcessSupervisor {
  pub fn new(machine: Arc<MachineInternal>, memory: Vec<u8>) -> Self {
    MachineProcessSupervisor {
      machine,
      memory,
      external_memory: None
    }
  }
}

impl ProcesSupervisor for MachineProcessSupervisor {
  fn set_memory(&mut self, unit: Option<usize>) {
    self.external_memory = match unit {
      Some(idx) => Some(self.machine.buffers[idx].clone()),
      None => None,
    }
  }

  fn get_memory(&mut self) -> MemoryHandler {
    self.external_memory.as_ref()
      .map(|x| MemoryHandler::MemoryLock(x.clone()))
      .unwrap_or(MemoryHandler::MemoryRef(&mut self.memory))
  }

  fn fork(&self, process: Process) {
    launch(self.machine.clone(), process);
  }
  
  fn invoke_ffi(&mut self, symbol: &[u8], process: &mut Process) -> Option<StackValue> {
    let registers = &mut process.registers[0..PUBLIC_REGISTERS_COUNT].try_into().unwrap();

    if process.get_flag_invoke_trap() { // ffi invoking a trap
      let machine = self.machine.clone();

      unsafe {
        invoke_ffi_trap(&machine.ffi, symbol, process, self)
      }.unwrap()
    } else if process.get_flag_share_memory() { // ffi sharing memory
      unsafe {
        match &self.external_memory {
          Some(external) => invoke_ffi_memory(&self.machine.ffi, symbol, registers, &mut *external.write().unwrap()),
          None => invoke_ffi_memory(&self.machine.ffi, symbol, registers,  &mut self.memory)
        }
      }.unwrap()
    } else { // normal ffi
      unsafe {
        invoke_ffi(&self.machine.ffi, symbol, registers)
      }.unwrap()
    }
  }
}

fn launch(machine: Arc<MachineInternal>, mut process: Process) -> Option<JoinHandle<Process>> {
  if process.is_finished() {
    return None
  }

  *machine.active.0.lock().unwrap() += 1;

  Some(thread::spawn(move || {
    let mut supervisor = MachineProcessSupervisor::new(machine, process.program.memory());

    process.run_until_finish(&mut supervisor);
    *supervisor.machine.active.0.lock().unwrap() -= 1;
    supervisor.machine.active.1.notify_all();
    process
  }))
}

pub struct Machine(Arc<MachineInternal>);

impl Machine {
  pub fn new() -> Self {
    Machine(Arc::new(MachineInternal::new()))
  }

  fn with_content(internal: MachineInternal) -> Self {
    Machine(Arc::new(internal))
  }

  pub fn launch(&mut self, program: Program) {
    launch(self.0.clone(), program.into());
  }

  pub fn wait(&mut self) {
    let (count, process_ended) = &self.0.active;
    let mut count_lock = count.lock().unwrap();
    while *count_lock > 0 {
      count_lock = process_ended.wait(count_lock).unwrap();
    }
  }
}

pub struct MachineBuilder(MachineInternal);

impl MachineBuilder {
  pub fn new() -> Self {
    MachineBuilder(MachineInternal::new())
  }

  pub fn add_memory(mut self, memory: impl Memory + 'static) -> Self {
    self.0.add_memory(memory);
    self
  }

  pub fn add_ffi_loader(mut self, loader: FFILoader) -> Self {
    self.0.add_ffi_loader(loader);
    self
  }

  pub fn build(self) -> Machine {
    Machine::with_content(self.0)
  }
}