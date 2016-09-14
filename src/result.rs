use hyper;
use serde_json;
use std::io;
use std::result;

#[derive(Debug)]
pub enum FdownError {
  BadConfig(String),
  BadFormat(String),
  Hyper(hyper::Error),
  Io(io::Error),
  SerdeJson(serde_json::Error)
}

pub type Result<T> = result::Result<T,FdownError>;

impl From<hyper::Error> for FdownError {
  fn from(err: hyper::Error) -> FdownError {
    FdownError::Hyper(err)
  }
}

impl From<io::Error> for FdownError {
  fn from(err: io::Error) -> FdownError {
    FdownError::Io(err)
  }
}

impl From<serde_json::Error> for FdownError {
  fn from(err: serde_json::Error) -> FdownError {
    FdownError::SerdeJson(err)
  }
}