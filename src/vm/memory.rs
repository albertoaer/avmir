use std::ops::DerefMut;

pub trait Memory: Send + Sync {
  fn write_int_64(&mut self, int: i64, offset: usize);
  fn read_int_64(&self, offset: usize) -> i64;
}

impl<T> Memory for T where T : DerefMut<Target = [u8]> + Send + Sync {
  fn write_int_64(&mut self, int: i64, offset: usize) {
    self[offset..8].copy_from_slice(&int.to_le_bytes());
  }

  fn read_int_64(&self, offset: usize) -> i64 {
    i64::from_le_bytes(self[offset..8].try_into().unwrap())
  }
}