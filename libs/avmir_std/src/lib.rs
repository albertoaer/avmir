use avmir::vm::{process::Registers, stack::StackValue};

/// hello world function to know everything worked
#[no_mangle]
fn std_hello_world() -> Option<StackValue> {
  println!("Hello World from FFI!");
  None
}

/// a example function to sum all the registers
#[no_mangle]
fn std_sum_registers(regs: Registers) -> Option<StackValue> {
  Some(StackValue::Int(regs.iter().map(|x| match x {
    StackValue::Int(x) => *x,
    StackValue::Float(x) => *x as i64,
  }).sum()))
}