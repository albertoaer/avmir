use std::{env::args, fs};

use crate::{parser::{Parser, simple}, vm::process::Process};

mod vm;
mod parser;

fn main() {
    let file = args().nth(1).expect("expecting file");
    let content = fs::read_to_string(file).expect("expecting file content");
    println!("{}", content);
    let program = match simple::Simple::parse(content) {
        Ok(program) => program,
        Err(err) => panic!("{}", err)
    };
    println!("{:?}", program);
    let mut process = Process::new(program);
    while process.run() { }
}
