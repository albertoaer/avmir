use std::ops::DerefMut;

pub trait Memory: Send + Sync {
  fn write_int_64(&mut self, int: i64, offset: usize);
  fn read_int_64(&self, offset: usize) -> i64;

  fn write_int_32(&mut self, int: i32, offset: usize);
  fn read_int_32(&self, offset: usize) -> i32;
  
  fn write_int_16(&mut self, int: i16, offset: usize);
  fn read_int_16(&self, offset: usize) -> i16;
  
  fn write_int_8(&mut self, int: i8, offset: usize);
  fn read_int_8(&self, offset: usize) -> i8;

  fn write_float(&mut self, float: f64, offset: usize);
  fn read_float(&self, offset: usize) -> f64;
}

impl<T> Memory for T where T : DerefMut<Target = [u8]> + Send + Sync {
  fn write_int_64(&mut self, int: i64, offset: usize) {
    self[offset..(offset+8)].copy_from_slice(&int.to_le_bytes());
  }

  fn read_int_64(&self, offset: usize) -> i64 {
    i64::from_le_bytes(self[offset..(offset+8)].try_into().unwrap())
  }

  fn write_int_32(&mut self, int: i32, offset: usize) {
    self[offset..(offset+4)].copy_from_slice(&int.to_le_bytes());
  }

  fn read_int_32(&self, offset: usize) -> i32 {
    i32::from_le_bytes(self[offset..(offset+4)].try_into().unwrap())
  }

  fn write_int_16(&mut self, int: i16, offset: usize) {
    self[offset..(offset+2)].copy_from_slice(&int.to_le_bytes());
  }

  fn read_int_16(&self, offset: usize) -> i16 {
    i16::from_le_bytes(self[offset..(offset+2)].try_into().unwrap())
  }

  fn write_int_8(&mut self, int: i8, offset: usize) {
    self[offset..(offset+1)].copy_from_slice(&int.to_le_bytes());
  }
  
  fn read_int_8(&self, offset: usize) -> i8 {
    i8::from_le_bytes(self[offset..(offset+1)].try_into().unwrap())
  }

  fn write_float(&mut self, float: f64, offset: usize) {
    self[offset..(offset+8)].copy_from_slice(&float.to_le_bytes());
  }

  fn read_float(&self, offset: usize) -> f64 {
    f64::from_le_bytes(self[offset..(offset+8)].try_into().unwrap())
  }
}