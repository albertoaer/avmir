use std::str::FromStr;

use thiserror::Error;

use crate::vm::program::{Instruction, InstructionParam, Opcode, Program};

use super::Parser;

pub struct Simple;

#[derive(Error, Debug)]
pub enum InternalSimpleParserError {
  #[error("operand invalid syntax")]
  OperandInvalidSyntax,

  #[error("bad line syntax: {0}")]
  BadLineSyntax(String),

  #[error("opcode not found")]
  OpcodeNotFound(#[from] <Opcode as FromStr>::Err)
}

fn parse_operand(item: &str) -> Result<InstructionParam, InternalSimpleParserError> {
  if let Ok(int) = item.parse() {
    Ok(InstructionParam::Int(int))
  } else if let Ok(float) = item.parse() {
    Ok(InstructionParam::Float(float))
  } else {
    Err(InternalSimpleParserError::OperandInvalidSyntax)
  }
}

fn parse_instruction(line: &str) -> Result<Instruction, InternalSimpleParserError> {
  let items: Vec<_> = line.split(' ').collect();
  Ok(match items.as_slice() {
    &[a] => Instruction::new(Opcode::from_str(a)?),
    &[a, b] => Instruction::new_args(Opcode::from_str(a)?, Some(parse_operand(b)?), None),
    &[a, "_", b] => Instruction::new_args(
      Opcode::from_str(a)?, None, Some(parse_operand(b)?)
    ),
    &[a, b, c] => Instruction::new_args(
      Opcode::from_str(a)?, Some(parse_operand(b)?), Some(parse_operand(c)?)
    ),
    _ => return Err(InternalSimpleParserError::BadLineSyntax(line.to_owned()))
  })
}

#[derive(Error, Debug)]
#[error("Error [LINE: {0}] :: {1}")]
pub struct SimpleParserError(usize, InternalSimpleParserError);

impl Parser for Simple {
  type Err = SimpleParserError;

  fn parse(source: impl AsRef<str>) -> Result<Program, Self::Err> {
    let mut program = Program::new();
    for (idx, line) in source.as_ref().lines().enumerate() {
      program.push(parse_instruction(line).map_err(|err| SimpleParserError(idx + 1, err))?);
    }
    Ok(program)
  }
}