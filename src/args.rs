use std::str::FromStr;
use thiserror::Error;

use clap::Parser;

#[derive(Debug, Clone)]
pub enum MemoryInput {
  Virtual {
    size: usize
  },
  FileMap {
    size: usize,
    path: String
  }
}

#[derive(Debug, Error)]
#[error("not a valid memory unit syntax")]
pub struct NotAValidMemoryError;

impl FromStr for MemoryInput {
  type Err = NotAValidMemoryError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    if let Ok(size) = s.parse() {
      return Ok(Self::Virtual { size });
    }

    let mut content = s.split(":");
    match (content.next().and_then(|x| x.parse().ok()), content.next()) {
      (Some(size), Some(path)) => Ok(Self::FileMap { size, path: path.into() }),
      _ => Err(NotAValidMemoryError)
    }
  }
}

#[derive(Parser)]
#[command(version, about)]
pub struct Args {
  #[arg(short)]
  pub memory: Vec<MemoryInput>,

  #[arg(short)]
  pub library: Vec<String>,

  #[arg()]
  pub files: Vec<String>
}