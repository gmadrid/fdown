use clap;
use hyper;
use serde_json;
use std::io;
use std::num;
use std::result;

#[derive(Debug)]
pub enum FdownError {
  BadConfig(String),
  BadFormat(String),
  Clap(clap::Error),
  Hyper(hyper::Error),
  Io(io::Error),
  MissingUrl(String),
  ParseIntError(num::ParseIntError),
  SerdeJson(serde_json::Error)
}

pub type Result<T> = result::Result<T,FdownError>;

impl From<clap::Error> for FdownError {
  fn from(err: clap::Error) -> FdownError {
    FdownError::Clap(err)
  }
}

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

impl From<num::ParseIntError> for FdownError {
  fn from(err: num::ParseIntError) -> FdownError {
    FdownError::ParseIntError(err)
  }
}

impl From<serde_json::Error> for FdownError {
  fn from(err: serde_json::Error) -> FdownError {
    FdownError::SerdeJson(err)
  }
}