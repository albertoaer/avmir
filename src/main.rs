use std::{fs::{self, OpenOptions}, io};

use clap::Parser as ArgsParser;
use memmap2::MmapOptions;
use thiserror::Error;
use vm::machine::MachineBuilder;

use crate::{parser::{simple, Parser}, vm::machine::Machine};

mod vm;
mod parser;
mod args;

#[derive(Debug, Error)]
enum RuntimeError {
  #[error("io error: {0}")]
  Fs(#[from] io::Error)
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

  Ok(builder)
}

fn main() -> Result<(), RuntimeError> {
  let args = args::Args::parse();
  let machine_builder = MachineBuilder::new();
  let mut machine: Machine = config_machine(&args, machine_builder)?.build();

  for file in args.files {
    let content = fs::read_to_string(file).expect("expecting file content");
    let program = match simple::Simple::parse(content) {
      Ok(program) => program,
      Err(err) => panic!("{}", err)
    };
    machine.launch(program);
  }

  machine.wait();

  Ok(())
}
