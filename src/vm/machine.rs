use std::{sync::{Arc, Condvar, Mutex, RwLock}, thread};

use super::{memory::Memory, process::{ProcesSupervisor, Process}, program::Program};

struct MachineInternal {
  active: (Mutex<usize>, Condvar),
  buffers: Vec<Arc<RwLock<dyn Memory>>>
}

impl MachineInternal {
  fn new() -> Self {
    MachineInternal {
      active: (Mutex::new(0), Condvar::new()),
      buffers: vec![]
    }
  }

  pub fn add_memory(&mut self, memory: impl Memory + 'static) {
    self.buffers.push(Arc::new(RwLock::new(memory)))
  }
}

#[derive(Clone)]
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

  pub fn launch(mut self, mut process: Process) {
    if process.is_finished() {
      return
    }

    *self.machine.active.0.lock().unwrap() += 1;

    thread::spawn(move || {
      process.run_until_finish(&mut self);
      *self.machine.active.0.lock().unwrap() -= 1;
      self.machine.active.1.notify_all();
    });
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
    self.clone().launch(process)
  }
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
    MachineProcessSupervisor::new(self.0.clone(), program.memory())
      .launch(program.into());
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

  pub fn build(self) -> Machine {
    Machine::with_content(self.0)
  }
}