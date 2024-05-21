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

struct MachineProcessSupervisor {
  memory: Memory,
  machine: Arc<MachineInternal>,
  external_memory: Option<Arc<RwLock<Memory>>>
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
}

impl Machine {
  pub fn new() -> Self {
    Machine(Arc::new(MachineInternal::new()))
  }

  pub fn launch(&mut self, program: Program) {
    let internal = self.0.clone();
    let mut supervisor = MachineProcessSupervisor{
      memory: Memory::new(1024),
      machine: internal.clone(),
      external_memory: None
    };
    
    internal.active.fetch_add(1, Ordering::Relaxed);
    thread::spawn(move || {

      let mut process = Process::new(program, &mut supervisor);
      while process.run() { }
      
      internal.active.fetch_sub(1, Ordering::Relaxed);
    });
  }

  pub fn wait(&mut self) {
    static DEFAULT_DURATION: Duration = Duration::from_millis(100);
    while self.0.active.load(Ordering::Relaxed) > 0 {
      thread::sleep(DEFAULT_DURATION)
    }
  }
}