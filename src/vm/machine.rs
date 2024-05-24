use std::{sync::{atomic::{AtomicUsize, Ordering}, Arc, RwLock}, thread, time::Duration};

use super::{memory::{Memory, MemoryBuffer}, process::{ProcesSupervisor, Process}, program::Program};

struct MachineInternal {
  active: AtomicUsize,
  buffers: Vec<Arc<RwLock<dyn Memory>>>
}

impl MachineInternal {
  fn new() -> Self {
    MachineInternal {
      active: AtomicUsize::new(0),
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
  memory: MemoryBuffer,
  external_memory: Option<Arc<RwLock<dyn Memory>>>
}

impl MachineProcessSupervisor {
  pub fn new(machine: Arc<MachineInternal>, memory: MemoryBuffer) -> Self {
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

    self.machine.active.fetch_add(1, Ordering::Relaxed);

    thread::spawn(move || {
      while !process.run(&mut self) { }
      
      self.machine.active.fetch_sub(1, Ordering::Relaxed);
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
    MachineProcessSupervisor::new(self.0.clone(), MemoryBuffer::with_content(program.static_data, 1024))
      .launch(Process::new(program.instructions));
  }

  pub fn wait(&mut self) {
    static DEFAULT_DURATION: Duration = Duration::from_millis(100);
    while self.0.active.load(Ordering::Relaxed) > 0 {
      thread::sleep(DEFAULT_DURATION)
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