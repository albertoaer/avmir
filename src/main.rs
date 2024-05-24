use std::{env::args, fs::{self, OpenOptions}, io};

use memmap2::MmapOptions;
use thiserror::Error;
use vm::machine::MachineBuilder;

use crate::{parser::{Parser, simple}, vm::machine::Machine};

mod vm;
mod parser;

#[derive(Debug, Error)]
enum RuntimeError {
  #[error("io error: {0}")]
  Fs(#[from] io::Error)
}

fn main() -> Result<(), RuntimeError> {
  let file = OpenOptions::new().read(true).write(true).create(true).open("./mem.bin")?;
  file.set_len(1024)?;

  let mapped = unsafe { MmapOptions::new().map_mut(&file)? };

  let mut machine: Machine = MachineBuilder::new().add_memory(mapped).build();

  for file in args().skip(1) {
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
