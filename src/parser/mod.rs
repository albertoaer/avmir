use crate::vm::program::Program;

pub mod simple;

pub trait Parser {
  type Err;

  fn parse(source: impl AsRef<str>) -> Result<Program, Self::Err>;
}