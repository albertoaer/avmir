use std::{borrow::BorrowMut, sync::{atomic::{AtomicUsize, Ordering}, Arc, Mutex, MutexGuard}, thread, time::Duration};

use super::{memory::Memory, process::Process, program::Program};

struct MachineInternal {
  active: Arc<AtomicUsize>,
  buffers: Vec<Arc<Mutex<Memory>>>
}

impl MachineInternal {
  fn new() -> Self {
    MachineInternal {
      active: Arc::new(AtomicUsize::new(0)),
      buffers: vec![
        Arc::new(Mutex::new(Memory::new(1024)))
      ]
    }
  }
}

pub struct Machine(Arc<MachineInternal>);

impl Machine {
  pub fn new() -> Self {
    Machine(Arc::new(MachineInternal::new()))
  }

  pub fn launch(&mut self, program: Program) {
    let internal = self.0.clone();
    let mut process = Process::new(program);

    internal.active.fetch_add(1, Ordering::Relaxed);
    thread::spawn(move || {
      let memory = Mutex::new(Memory::new(1024));
      let mut mounted_memory: MutexGuard<Memory> = memory.lock().unwrap();
      let mut mounted_unit = None;
      while process.run(mounted_memory.borrow_mut()) {
        if mounted_unit != process.external_unit() {
          mounted_unit = process.external_unit();
          mounted_memory = match mounted_unit {
            Some(idx) if idx < internal.buffers.len() => internal.buffers[idx].lock().unwrap(),
            None => memory.lock().unwrap(),
            _ => panic!("unit does not exists")
          }
        }
      }
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