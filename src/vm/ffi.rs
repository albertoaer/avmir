use libloading::{library_filename, Symbol};
use thiserror::Error;

use super::{memory::Memory, process::{ProcesSupervisor, Process, PublicRegisters}, stack::StackValue};

#[derive(Debug, Error)]
pub enum FFIError {
  #[error("unable to find symbol")]
  NotFound,
  #[error("{0}")]
  SymbolError(#[from] libloading::Error)
}

#[derive(Debug)]
pub struct FFILoader(libloading::Library);

impl FFILoader {
  pub unsafe fn new(path: impl AsRef<str>) -> Result<Self, FFIError> {
    Ok(Self(libloading::Library::new(library_filename(path.as_ref()))?))
  }

  pub unsafe fn invoke_ffi(
    &self, symbol: &[u8], registers: &mut PublicRegisters
  ) -> Result<Option<StackValue>, FFIError> {
    let symbol: Symbol<fn (&mut PublicRegisters) -> Option<StackValue>> = self.0.get(symbol)?;
    Ok(symbol(registers))
  }

  pub unsafe fn invoke_ffi_memory(
    &self, symbol: &[u8], registers: &mut PublicRegisters, memory: &mut dyn Memory
  ) -> Result<Option<StackValue>, FFIError> {
    let symbol: Symbol<fn (&mut PublicRegisters, &mut dyn Memory) -> Option<StackValue>> = self.0.get(symbol)?;
    Ok(symbol(registers, memory))
  }

  pub unsafe fn invoke_ffi_trap(
    &self, symbol: &[u8], process: &mut Process, supervisor: &mut dyn ProcesSupervisor
  ) -> Result<Option<StackValue>, FFIError> {
    let symbol: Symbol<fn (&mut Process, &mut dyn ProcesSupervisor) -> Option<StackValue>> = self.0.get(symbol)?;
    Ok(symbol(process, supervisor))
  }
}

pub unsafe fn invoke_ffi(
  many: &[FFILoader], symbol: &[u8], registers: &mut PublicRegisters
) -> Result<Option<StackValue>, FFIError>{
  for loader in many.iter() {
    if let Ok(output) = loader.invoke_ffi(symbol, registers) {
      return Ok(output)
    }
  }
  return Err(FFIError::NotFound)
}

pub unsafe fn invoke_ffi_memory(
  many: &[FFILoader], symbol: &[u8], registers: &mut PublicRegisters, memory: &mut dyn Memory
) -> Result<Option<StackValue>, FFIError>{
  for loader in many.iter() {
    if let Ok(output) = loader.invoke_ffi_memory(symbol, registers, memory) {
      return Ok(output)
    }
  }
  return Err(FFIError::NotFound)
}

pub unsafe fn invoke_ffi_trap(
  many: &[FFILoader], symbol: &[u8], process: &mut Process, supervisor: &mut dyn ProcesSupervisor
) -> Result<Option<StackValue>, FFIError>{
  for loader in many.iter() {
    if let Ok(output) = loader.invoke_ffi_trap(symbol, process, supervisor) {
      return Ok(output)
    }
  }
  return Err(FFIError::NotFound)
}