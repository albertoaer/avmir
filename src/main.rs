use std::{fs::{self, OpenOptions}, io};

use clap::Parser as ArgsParser;
use memmap2::MmapOptions;
use thiserror::Error;
use vm::{ffi::{FFIError, FFILoader}, machine::MachineBuilder, program::Program};

use crate::{parser::{v2, Parser}, vm::machine::Machine};

pub mod vm;
pub mod parser;
mod args;

#[derive(Debug, Error)]
enum RuntimeError {
  #[error("io error: {0}")]
  Fs(#[from] io::Error),

  #[error("parse error: {0}")]
  ParsingError(#[from] v2::SimpleParserError),

  #[error("{0}")]
  FFIError(#[from] FFIError)
}

fn config_machine(args: &args::Args, mut builder: MachineBuilder) -> Result<MachineBuilder, RuntimeError> {
  for mem in args.memory.iter() {
    builder = match mem {
      args::MemoryInput::Virtual { size } => builder.add_memory(vec![0; *size]),
      args::MemoryInput::FileMap { size, path } => {
        let file = OpenOptions::new().read(true).write(true).create(true).open(path)?;
        let size = *size as u64;
        if file.metadata()?.len() < size {
          file.set_len(size)?;
        }
        builder.add_memory(unsafe { MmapOptions::new().map_mut(&file)? })
      }
    }
  }

  for lib in args.library.iter() {
    builder = builder.add_ffi_loader(unsafe { FFILoader::new(lib)? })
  }

  Ok(builder)
}

fn main() -> Result<(), RuntimeError> {
  let args = args::Args::parse();
  let machine_builder = MachineBuilder::new();
  let mut machine: Machine = config_machine(&args, machine_builder)?.build();

  args.files.iter().map(|file| -> Result<Program, RuntimeError> {
    let mut program = Program::with_name(&file);
    let content = fs::read_to_string(file)?;
    v2::Simple::parse(&mut program, content)?;
    Ok(program)
  }).collect::<Result<Vec<Program>, _>>()?.into_iter().for_each(|program| machine.launch(program));

  machine.wait();

  Ok(())
}
