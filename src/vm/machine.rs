use std::{sync::{atomic::{AtomicUsize, Ordering}, Arc, RwLock}, thread, time::Duration};

use super::{memory::Memory, process::{ProcesSupervisor, Process}, program::Program};

struct MachineInternal {
  active: Arc<AtomicUsize>,
  buffers: Vec<Arc<RwLock<Memory>>>
}

impl MachineInternal {
  fn new() -> Self {
    MachineInternal {
      active: Arc::new(AtomicUsize::new(0)),
      buffers: vec![
        Arc::new(RwLock::new(Memory::new(1024)))
      ]
    }
  }
}

pub struct Machine(Arc<MachineInternal>);

#[derive(Clone)]
struct MachineProcessSupervisor {
  machine: Arc<MachineInternal>,
  memory: Memory,
  external_memory: Option<Arc<RwLock<Memory>>>
}

impl MachineProcessSupervisor {
  pub fn new(machine: Arc<MachineInternal>, memory: Memory) -> Self {
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

  fn memory<T>(&self, effect: impl FnOnce(&Memory) -> T) -> T {
    match &self.external_memory {
      Some(external) => effect(&external.read().unwrap()),
      None => effect(&self.memory)
    }
  }

  fn memory_mut<T>(&mut self, effect: impl FnOnce(&mut Memory) -> T) -> T {
    match &self.external_memory {
      Some(external) => effect(&mut external.write().unwrap()),
      None => effect(&mut self.memory)
    }
  }
  
  fn fork(&self, process: Process) {
    self.clone().launch(process)
  }
}

impl Machine {
  pub fn new() -> Self {
    Machine(Arc::new(MachineInternal::new()))
  }

  pub fn launch(&mut self, program: Program) {
    MachineProcessSupervisor::new(self.0.clone(), Memory::new(1024))
      .launch(Process::new(program));
  }

  pub fn wait(&mut self) {
    static DEFAULT_DURATION: Duration = Duration::from_millis(100);
    while self.0.active.load(Ordering::Relaxed) > 0 {
      thread::sleep(DEFAULT_DURATION)
    }
  }
}