use std::borrow::BorrowMut;

use crate::vm::program::Program;

#[path = "simple_v1.rs"]
#[deprecated(since = "0.1.1", note = "use v2 instead")]
pub mod v1;

#[path = "simple_v2.rs"]
pub mod v2;

pub trait Parser {
  type Err;

  fn parse(target: impl BorrowMut<Program>, source: impl AsRef<str>) -> Result<(), Self::Err>;
}