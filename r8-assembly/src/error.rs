use std::{
  fmt::{Debug, Display},
  num::ParseIntError,
};

use super::tokenizer::OwnedLine;

pub enum Error {
  IOErr(std::io::Error),
  DuplicateLabel(String, usize),
  UndefinedLabel(String, usize),
  InvalidAddress(u16, usize),
  InvalidNumber(ParseIntError, String, usize),
  InvalidRegister(u16, usize),
  InvalidByte(u16, usize),
  InvalidNibble(u16, usize),
  InvalidToken(String, usize),
  InvalidLine(OwnedLine),
}

impl Error {
  pub fn warp<T>(self) -> Result<T, Self> {
    Err(self)
  }
}

impl From<std::io::Error> for Error {
  fn from(err: std::io::Error) -> Self {
    Error::IOErr(err)
  }
}

impl Display for Error {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Error::IOErr(err) => write!(f, "IO Error: {}", err),
      Error::DuplicateLabel(name, line) => write!(f, "Duplicate label {} at line {}", name, line),
      Error::UndefinedLabel(name, line) => write!(f, "Undefined label {} at line {}", name, line),
      Error::InvalidAddress(addr, line) => write!(f, "Invalid address {} at line {}", addr, line),
      Error::InvalidNumber(msg, num, line) => write!(
        f,
        "Invalid number {} at line {} wit error {}",
        num, line, msg
      ),
      Error::InvalidRegister(reg, line) => write!(f, "Invalid register {} at line {}", reg, line),
      Error::InvalidToken(token, line) => write!(f, "Invalid token {} at line {}", token, line),
      Error::InvalidByte(byte, line) => write!(
        f,
        "Invalid byte at line {}: {} is bigger than 0xFF",
        line, byte
      ),
      Error::InvalidNibble(nibble, line) => write!(
        f,
        "Invalid nibble at line {}: {} is bigger than 0xF",
        line, nibble
      ),
      Error::InvalidLine(line) => write!(f, "Invalid line: {:?}", line),
    }
  }
}

impl Debug for Error {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    Display::fmt(self, f)
  }
}

impl std::error::Error for Error {}
