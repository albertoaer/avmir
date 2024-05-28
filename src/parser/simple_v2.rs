use std::{borrow::BorrowMut, collections::HashMap, str::FromStr};

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

#[derive(Error, Debug)]
#[error("Error [LINE: {0}] :: {1}")]
pub struct SimpleParserError(usize, InternalSimpleParserError);

struct ParserV2<'a> {
  program: &'a mut Program,
  memory_tags: HashMap<String, usize>, // tag => memory address
  line_tags: HashMap<String, usize>, // tag => command
}

impl<'a> ParserV2<'a> {
  pub fn consume_tags_and_memory(&mut self, source: &mut Vec<(usize, String)>) -> Result<(), SimpleParserError> {
    source.retain(|(_, x)| match x.trim() {
      ";" | "" => false,
      _ => true
    }); // remove empty lines and comments

    for (idx, line) in source.iter_mut().map(|(idx, line)| (idx, line.trim_start())) {
      match (line.find(";"), line.find(":")) {
      }

      if line.starts_with("#") {
        let begin = self.program.static_data.len();
        self.program.static_data.extend_from_slice(&line.as_bytes()[1..]);
        let end = self.program.static_data.len();
        self.program.static_data_meta.push((begin, end - begin));
        continue
      }
    }

    source.retain(|(_, x)| !x.trim().is_empty()); // remove empty lines
    Ok(())
  }

  pub fn consume_instructions(&mut self, source: &mut Vec<(usize, String)>) -> Result<(), SimpleParserError> {
    for (idx, line) in source.iter() {
      self.consume_instruction(line).map_err(|err| SimpleParserError(*idx, err))?;
    }
    Ok(())
  }

  pub fn consume_instruction(&mut self, line: &String) -> Result<(), InternalSimpleParserError> {
    let items: Vec<_> = line.split(' ').filter(|x| !x.is_empty()).collect();
    let instruction = match items.as_slice() {
      &[a] => Instruction::new(Opcode::from_str(a)?),
      &[a, b] => Instruction::with_args(Opcode::from_str(a)?, self.parse_operand(b)?, None),
      &[a, b, c] => Instruction::with_args(
        Opcode::from_str(a)?, self.parse_operand(b)?, self.parse_operand(c)?
      ),
      _ => return Err(InternalSimpleParserError::BadLineSyntax(line.to_owned()))
    };
    self.program.instructions.push(instruction);
    Ok(())
  }

  pub fn parse_operand(&mut self, item: &str) -> Result<Option<InstructionParam>, InternalSimpleParserError> {
    // get memory item address, size or next address
    if item.starts_with("$") || item.starts_with("@") || item.starts_with("^") {
      return if let Ok(idx) = item[1..].parse() {
        match (item.chars().next().unwrap(), self.program.static_data_meta.get(idx)) {
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
}

impl Parser for Simple {
  type Err = SimpleParserError;

  fn parse(mut target: impl BorrowMut<Program>, source: impl AsRef<str>) -> Result<(), Self::Err> {
    let mut parser = ParserV2{
      program: target.borrow_mut(),
      line_tags: HashMap::new(),
      memory_tags: HashMap::new()
    };

    let mut source: Vec<_> = source.as_ref().lines().enumerate().map(|(idx, line)| (idx, line.to_string())).collect();

    parser.consume_tags_and_memory(&mut source)?;

    parser.consume_instructions(&mut source)?;

    Ok(())
  }
}