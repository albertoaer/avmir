use std::ops::DerefMut;

pub trait Memory: Send + Sync {
  fn write_int_64(&mut self, int: i64, offset: usize);
  fn read_int_64(&self, offset: usize) -> i64;
}

#[derive(Debug, Clone)]
pub struct MemoryBuffer {
  raw: Vec<u8>
}

impl MemoryBuffer {
  pub fn new(size: usize) -> Self {
    MemoryBuffer {
      raw: vec![0; size]
    }
  }

  pub fn with_content(mut content: Vec<u8>, size: usize) -> Self {
    if content.len() < size {
      content.resize(size, 0)
    }
    MemoryBuffer {
      raw: content
    }
  }

  pub fn size(&self) -> usize {
    self.raw.len()
  }

  pub fn resize(&mut self, size: usize) {
    self.raw.resize(size, 0)
  }
}

impl Memory for MemoryBuffer {
  fn write_int_64(&mut self, int: i64, offset: usize) {
    self.raw[offset..8].copy_from_slice(&int.to_le_bytes());
  }

  fn read_int_64(&self, offset: usize) -> i64 {
    i64::from_le_bytes(self.raw[offset..8].try_into().unwrap())
  }
}

impl<T> Memory for T where T : DerefMut<Target = [u8]> + Send + Sync {
  fn write_int_64(&mut self, int: i64, offset: usize) {
    self[offset..8].copy_from_slice(&int.to_le_bytes());
  }

  fn read_int_64(&self, offset: usize) -> i64 {
    i64::from_le_bytes(self[offset..8].try_into().unwrap())
  }
}