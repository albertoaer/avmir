use std::ops::DerefMut;

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