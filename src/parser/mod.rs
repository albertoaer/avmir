use std::borrow::BorrowMut;

use crate::vm::program::Program;

pub mod simple;

pub trait Parser {
  type Err;

  fn parse(target: impl BorrowMut<Program>, source: impl AsRef<str>) -> Result<(), Self::Err>;
}