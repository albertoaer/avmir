use std::{env::args, fs};

use vm::{machine::MachineBuilder, memory::MemoryBuffer};

use crate::{parser::{Parser, simple}, vm::machine::Machine};

mod vm;
mod parser;

fn main() {
    let mut machine: Machine = MachineBuilder::new().add_memory(MemoryBuffer::new(1024)).build();

    for file in args().skip(1) {
        let content = fs::read_to_string(file).expect("expecting file content");
        let program = match simple::Simple::parse(content) {
            Ok(program) => program,
            Err(err) => panic!("{}", err)
        };
        machine.launch(program);
    }

    machine.wait();
}
