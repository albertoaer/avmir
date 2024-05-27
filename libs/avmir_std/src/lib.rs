use avmir::vm::{memory::Memory, process::PublicRegisters, stack::StackValue};

/// hello world function to know everything worked
#[no_mangle]
fn std_hello_world() -> Option<StackValue> {
  println!("Hello World from FFI!");
  None
}

/// a example function to sum all the registers
#[no_mangle]
fn std_sum_registers(regs: PublicRegisters) -> Option<StackValue> {
  Some(StackValue::Int(regs.iter().map(|x| match x {
    StackValue::Int(x) => *x,
    StackValue::Float(x) => *x as i64,
  }).sum()))
}

/// print a chunk of memory
#[no_mangle]
fn std_print(regs: PublicRegisters, memory: &mut dyn Memory) -> Option<StackValue> {
  println!("{}", String::from_utf8_lossy(&memory.read(regs[0].into(), regs[1].into())));
  None
}