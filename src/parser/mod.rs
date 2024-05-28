use std::borrow::BorrowMut;

use crate::vm::program::Program;

#[cfg_attr(not(somefeature), path = "simple_v1.rs")]
pub mod v1;
#[cfg_attr(not(somefeature), path = "simple_v2.rs")]
pub mod v2;

pub trait Parser {
  type Err;

  fn parse(target: impl BorrowMut<Program>, source: impl AsRef<str>) -> Result<(), Self::Err>;
}