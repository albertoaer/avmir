//! This is the version 2 of the simple parser.
//! 
//! It's designed to provide tags in order to write memory access and jumps easier,
//! 
//! A line must contain only either: a comment, a chunk to be written in memory or a instruction
//! 
//! If no tag is provided, for example "#memory" ":push 0", the index of the line would be used as tag (starting in 0)
//! 
//! Numeric tags are not compatible with v1, so both parsers are not interchangeable

use std::{borrow::BorrowMut, collections::HashMap, str::FromStr};

use thiserror::Error;

use crate::vm::program::{Instruction, InstructionParam, Opcode, Program};

use super::Parser;

pub struct Simple;

#[derive(Error, Debug)]
pub enum InternalSimpleParserError {
  #[error("operand invalid syntax")]
  OperandInvalidSyntax,

  #[error("bad tag: {0}, not found")]
  BadTagNotFound(String),

  #[error("bad tag: {0}, only $ is valid for instruction tag")]
  InstructionBadTag(String),

  #[error("bad line syntax: {0}")]
  BadLineSyntax(String),

  #[error("opcode not found")]
  OpcodeNotFound(#[from] <Opcode as FromStr>::Err)
}

#[derive(Error, Debug)]
#[error("Error [LINE: {0}] :: {1}")]
pub struct SimpleParserError(usize, InternalSimpleParserError);

enum Tag {
  Memory {
    address: usize,
    size: usize,
  },
  Instruction {
    line: usize
  }
}

struct ParserV2<'a> {
  program: &'a mut Program,
  tags: HashMap<String, Tag>, // tag => command
}

impl<'a> ParserV2<'a> {
  pub fn consume_tags_and_memory(&mut self, source: &mut Vec<(usize, String)>) -> Result<(), SimpleParserError> {
    source.retain(|(_, x)| match x.trim().chars().next() {
      Some(';') | None => false,
      _ => true
    }); // remove empty lines and comments

    let mut instruction_counter = 0;

    for (idx, line) in source.iter_mut() {
      *line = line.trim_start().into();

      if let Some(memory_idx) = line.find("#") {
        let tag: String = match line[..memory_idx].trim() {
          x if x.len() > 0 => x.into(),
          _ => idx.to_string()
        };

        let begin = self.program.static_data.len();
        self.program.static_data.extend_from_slice(&line[(memory_idx + 1)..].as_bytes());
        let end = self.program.static_data.len();
        self.program.static_data_meta.push((begin, end - begin)); // TODO?: remove static_data_meta

        self.tags.insert(tag, Tag::Memory { address: begin, size: end - begin });
        
        *line = String::new();

        continue // skip so the instruction counter does not increment
      }

      if let Some(line_tag_idx) = line.find(":") {
        let tag: String = match line[..line_tag_idx].trim() {
          x if x.len() > 0 => x.into(),
          _ => idx.to_string()
        };

        self.tags.insert(tag, Tag::Instruction { line: instruction_counter });

        *line = line[(line_tag_idx+1)..].into();
      }

      // here we are assume this is a instruction, otherwise consume_instructions will return an error
      instruction_counter += 1; 
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
    // handle tag
    if item.starts_with("$") || item.starts_with("@") || item.starts_with("^") {
      let tag = &item[1..];
      return match (item.chars().next().unwrap(), self.tags.get(tag)) {
        ('$', Some(&Tag::Memory { address, .. })) => Ok(Some(InstructionParam::Int(address as i64))),
        ('@', Some(&Tag::Memory { size, .. })) => Ok(Some(InstructionParam::Int(size as i64))),
        ('^', Some(&Tag::Memory { address, size })) => Ok(Some(InstructionParam::Int((address + size) as i64))),
        ('$', Some(&Tag::Instruction { line })) => Ok(Some(InstructionParam::Int(line as i64))),
        ('@' | '^', Some(&Tag::Instruction { .. })) => Err(InternalSimpleParserError::InstructionBadTag(tag.into())),
        (_, None) => Err(InternalSimpleParserError::BadTagNotFound(tag.into())),
        _ => Err(InternalSimpleParserError::BadTagNotFound(tag.into())) // should never trigger, char not in $, @, ^
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
      tags: HashMap::new()
    };

    let mut source: Vec<_> = source.as_ref().lines().enumerate().map(|(idx, line)| (idx, line.to_string())).collect();

    parser.consume_tags_and_memory(&mut source)?;

    parser.consume_instructions(&mut source)?;

    Ok(())
  }
}