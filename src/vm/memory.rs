#[derive(Debug, Clone)]
pub struct Memory {
  raw: Vec<u8>
}

impl Memory {
  pub fn new(size: usize) -> Self {
    Memory {
      raw: vec![0; size]
    }
  }

  pub fn with_content(mut content: Vec<u8>, size: usize) -> Self {
    if content.len() < size {
      content.resize(size, 0)
    }
    Memory {
      raw: content
    }
  }

  pub fn size(&self) -> usize {
    self.raw.len()
  }

  pub fn resize(&mut self, size: usize) {
    self.raw.resize(size, 0)
  }

  pub fn write_int_64(&mut self, int: i64, offset: usize) {
    self.raw[offset..8].copy_from_slice(&int.to_le_bytes());
  }

  pub fn read_int_64(&self, offset: usize) -> i64 {
    i64::from_le_bytes(self.raw[offset..8].try_into().unwrap())
  }
}