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

fn parse_operand(item: &str) -> Result<Option<InstructionParam>, InternalSimpleParserError> {
  if item == "_" {
    Ok(None)
  } else if let Ok(int) = item.parse() {
    Ok(Some(InstructionParam::Int(int)))
  } else if let Ok(float) = item.parse() {
    Ok(Some(InstructionParam::Float(float)))
  } else {
    Err(InternalSimpleParserError::OperandInvalidSyntax)
  }
}

fn parse_instruction(line: &str) -> Result<Instruction, InternalSimpleParserError> {
  let items: Vec<_> = line.split(' ').filter(|x| !x.is_empty()).collect();
  Ok(match items.as_slice() {
    &[a] => Instruction::new(Opcode::from_str(a)?),
    &[a, b] => Instruction::new_args(Opcode::from_str(a)?, parse_operand(b)?, None),
    &[a, b, c] => Instruction::new_args(
      Opcode::from_str(a)?, parse_operand(b)?, parse_operand(c)?
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
    for (idx, line) in source.as_ref().lines().map(|l| l.trim()).enumerate() {
      if line.is_empty() {
        continue;
      }
      program.push(parse_instruction(line).map_err(|err| SimpleParserError(idx + 1, err))?);
    }
    Ok(program)
  }
}