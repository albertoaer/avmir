use std::{borrow::BorrowMut, str::FromStr};

use thiserror::Error;

use crate::vm::program::{Instruction, InstructionParam, Opcode, Program};

use super::Parser;

pub struct Simple;

#[derive(Error, Debug)]
pub enum InternalSimpleParserError {
  #[error("operand invalid syntax")]
  OperandInvalidSyntax,

  #[error("bad memory item: {0}")]
  BadMemoryItem(usize),

  #[error("bad line syntax: {0}")]
  BadLineSyntax(String),

  #[error("opcode not found")]
  OpcodeNotFound(#[from] <Opcode as FromStr>::Err)
}

fn parse_operand(item: &str, program: &Program) -> Result<Option<InstructionParam>, InternalSimpleParserError> {
  // get memory item address, size or next address
  if item.starts_with("$") || item.starts_with("@") || item.starts_with("^") {
    return if let Ok(idx) = item[1..].parse() {
      match (item.chars().next().unwrap(), program.static_data_meta.get(idx)) {
        ('$', Some((address, _))) => Ok(Some(InstructionParam::Int(*address as i64))),
        ('@', Some((_, size))) => Ok(Some(InstructionParam::Int(*size as i64))),
        (_, Some((address, size))) => Ok(Some(InstructionParam::Int((*address + *size) as i64))),
        (_, None) => Err(InternalSimpleParserError::BadMemoryItem(idx))
      }
    } else {
      Err(InternalSimpleParserError::OperandInvalidSyntax)
    }
  }

  // default behaviour
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

fn parse_instruction(line: &str, program: &Program) -> Result<Instruction, InternalSimpleParserError> {
  let items: Vec<_> = line.split(' ').filter(|x| !x.is_empty()).collect();
  Ok(match items.as_slice() {
    &[a] => Instruction::new(Opcode::from_str(a)?),
    &[a, b] => Instruction::with_args(Opcode::from_str(a)?, parse_operand(b, program)?, None),
    &[a, b, c] => Instruction::with_args(
      Opcode::from_str(a)?, parse_operand(b, program)?, parse_operand(c, program)?
    ),
    _ => return Err(InternalSimpleParserError::BadLineSyntax(line.to_owned()))
  })
}

#[derive(Error, Debug)]
#[error("Error [LINE: {0}] :: {1}")]
pub struct SimpleParserError(usize, InternalSimpleParserError);

impl Parser for Simple {
  type Err = SimpleParserError;

  fn parse(mut target: impl BorrowMut<Program>, source: impl AsRef<str>) -> Result<(), Self::Err> {
    let program = target.borrow_mut();

    for (idx, line) in source.as_ref().lines().map(|l| l.trim()).enumerate() {
      if line.is_empty() || line.starts_with(";") {
        continue;
      }
      
      if line.starts_with("#") {
        let begin = program.static_data.len();
        program.static_data.extend_from_slice(&line.as_bytes()[1..]);
        let end = program.static_data.len();
        program.static_data_meta.push((begin, end - begin));
        continue
      }

      program.instructions.push(parse_instruction(line, &program).map_err(|err| SimpleParserError(idx + 1, err))?);
    }
    Ok(())
  }
}