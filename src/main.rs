use std::{env::args, fs};

use crate::{parser::{Parser, simple}, vm::{machine::Machine, process::Process}};

mod vm;
mod parser;

fn main() {
    let mut machine: Machine = Machine::new();

    for file in args().skip(1) {
        let content = fs::read_to_string(file).expect("expecting file content");
        let program = match simple::Simple::parse(content) {
            Ok(program) => program,
            Err(err) => panic!("{}", err)
        };
        let process = Process::new(program);
        machine.launch(process);
    }

    machine.wait();
}
