use std::{sync::{Arc, Condvar, Mutex, RwLock}, thread::{self, JoinHandle}};

use super::{ffi::{FFIInvoke, FFILoader}, memory::Memory, process::{ProcesSupervisor, Process, Registers}, program::Program, stack::StackValue};

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

  fn memory<T>(&self, effect: impl FnOnce(&dyn Memory) -> T) -> T {
    match &self.external_memory {
      Some(external) => effect(& *external.read().unwrap()),
      None => effect(&self.memory)
    }
  }

  fn memory_mut<T>(&mut self, effect: impl FnOnce(&mut dyn Memory) -> T) -> T {
    match &self.external_memory {
      Some(external) => effect(&mut *external.write().unwrap()),
      None => effect(&mut self.memory)
    }
  }

  fn fork(&self, process: Process) {
    launch(self.machine.clone(), process);
  }
  
  fn invoke_ffi(&mut self, symbol: &[u8], process: &mut Process) -> Option<StackValue> {
    unsafe {
      FFIInvoke::<fn (Registers) -> Option<StackValue>, _, _>::invoke_ffi(&self.machine.ffi, symbol, process.registers)
    }.unwrap()
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