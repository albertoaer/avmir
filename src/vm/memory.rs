use std::{ops::DerefMut, sync::{Arc, RwLock}};

pub trait Memory: Send + Sync {
  fn write(&mut self, offset: usize, data: &[u8]);
  fn read(&self, offset: usize, size: usize) -> &[u8];
}

impl<T> Memory for T where T : DerefMut<Target = [u8]> + Send + Sync {
  fn write(&mut self, offset: usize, data: &[u8]) {
    self[offset..(offset+data.len())].copy_from_slice(data)
  }

  fn read(&self, offset: usize, size: usize) -> &[u8] {
    &self[offset..(offset+size)]
  }
}

pub enum MemoryHandler<'a> {
  MemoryRef(&'a mut dyn Memory),
  MemoryLock(Arc<RwLock<dyn Memory>>)
}

impl<'a> MemoryHandler<'a> {
  pub fn memory<T>(&self, effect: impl FnOnce(&dyn Memory) -> T) -> T {
    match self {
      Self::MemoryRef(memory) => effect(*memory),
      Self::MemoryLock(lock) => effect(& *lock.read().unwrap())
    }
  }

  pub fn memory_mut<T>(&mut self, effect: impl FnOnce(&mut dyn Memory) -> T) -> T {
    match self {
      Self::MemoryRef(memory) => effect(*memory),
      Self::MemoryLock(lock) => effect(&mut *lock.write().unwrap())
    }
  }
}