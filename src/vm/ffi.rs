use libloading::{library_filename, Symbol};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FFIError {
  #[error("unable to find symbol")]
  NotFound,
  #[error("{0}")]
  SymbolError(#[from] libloading::Error)
}

pub trait FFIInvoke<F, A, R> {
  unsafe fn invoke_ffi(&self, symbol: &[u8], args: A) -> Result<R, FFIError> where F: Fn(A) -> R;
}

#[derive(Debug)]
pub struct FFILoader(libloading::Library);

impl FFILoader {
  pub unsafe fn new(path: impl AsRef<str>) -> Result<Self, FFIError> {
    Ok(Self(libloading::Library::new(library_filename(path.as_ref()))?))
  }
}

impl<F, A, R> FFIInvoke<F, A, R> for FFILoader {
  unsafe fn invoke_ffi(&self, symbol: &[u8], args: A) -> Result<R, FFIError> where F: Fn(A) -> R {
    let symbol: Symbol<F> = self.0.get(symbol)?;
    Ok(symbol(args))
  }
}

impl<F, A, R> FFIInvoke<F, A, R> for Vec<FFILoader> where A : Clone {
  unsafe fn invoke_ffi(&self, symbol: &[u8], args: A) -> Result<R, FFIError> where F: Fn(A) -> R {
    for loader in self.iter() {
      if let Ok(result) = <FFILoader as FFIInvoke<F, A, R>>::invoke_ffi(loader, symbol, args.clone()) {
        return Ok(result)
      }
    }
    Err(FFIError::NotFound)
  }
}