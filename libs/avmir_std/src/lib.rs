use avmir::vm::{memory::Memory, process::{ProcesSupervisor, Process, PublicRegisters}, stack::StackValue};

/// hello world function to know everything worked
#[no_mangle]
fn std_hello_world() -> Option<StackValue> {
  println!("Hello World from FFI!");
  None
}

/// a example function to sum all the registers
/// registers: from 0 to 9
#[no_mangle]
fn std_sum_registers(regs: &PublicRegisters) -> Option<StackValue> {
  Some(StackValue::Int(regs.iter().map(|x| Into::<i64>::into(*x)).sum()))
}

/// print a chunk of memory
#[no_mangle]
fn std_print(regs: &PublicRegisters, memory: &mut dyn Memory) -> Option<StackValue> {
  print!("{}", String::from_utf8_lossy(&memory.read(regs[0].into(), regs[1].into())));
  None
}

/// print a chunk of memory with line end
#[no_mangle]
fn std_println(regs: &PublicRegisters, memory: &mut dyn Memory) -> Option<StackValue> {
  println!("{}", String::from_utf8_lossy(&memory.read(regs[0].into(), regs[1].into())));
  None
}

/// print the first register
#[no_mangle]
fn std_reg_print(regs: &PublicRegisters) -> Option<StackValue> {
  print!("{}", regs[0]);
  None
}

/// print the first register with line end
#[no_mangle]
fn std_reg_println(regs: &PublicRegisters) -> Option<StackValue> {
  println!("{}", regs[0]);
  None
}

/// debug trap while handle the program showing the current instruction until it finishes
#[no_mangle]
fn std_trap_debug(process: &mut Process, supervisor: &mut dyn ProcesSupervisor) -> Option<StackValue> {
  loop {
    if let Some(intruction) = process.get_current_instruction() {
      println!("RUNNING: {}", intruction);
    }

    if !process.run_next(supervisor) {
      break
    }
  }
  println!("DONE!");
  None
}