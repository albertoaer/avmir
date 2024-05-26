use libloading::{library_filename, Symbol};

use super::{memory::Memory, process::{ProcesSupervisor, Process, Registers}};

pub trait FFITrap {
  fn name(&self) -> &'static str;
  fn handle(&self, process: &mut Process);
}

pub trait FFIFunction {
  fn name(&self) -> &'static str;
  fn call(&self, memory: &mut dyn Memory, registers: Registers);
}

pub enum FFI {
  Trap(Box<dyn FFITrap>),
  Function(Box<dyn FFIFunction>)
}

impl FFI {
  pub fn handle(&self, supervisor: &mut impl ProcesSupervisor, process: &mut Process) {
    match self {
      Self::Trap(trap) => trap.handle(process),
      Self::Function(function) => supervisor.memory_mut(|memory| function.call(memory, process.registers())),
    }
  }
}

pub struct FFILoader(libloading::Library);

impl FFILoader {
  pub unsafe fn new(path: impl AsRef<str>) -> Result<Self, libloading::Error> {
    Ok(Self(libloading::Library::new(library_filename(path.as_ref()))?))
  }

  pub unsafe fn load_ffi_trap(&self, name: &[u8]) -> FFI {
    let symbol: Symbol<fn() -> *mut dyn FFITrap> = self.0.get(name).expect("expecting symbol");
    FFI::Trap(Box::from_raw(symbol()))
  }

  pub unsafe fn load_ffi_function(&self, name: &[u8]) -> FFI {
    let symbol: Symbol<fn() -> *mut dyn FFIFunction> = self.0.get(name).expect("expecting symbol");
    FFI::Function(Box::from_raw(symbol()))
  }
}