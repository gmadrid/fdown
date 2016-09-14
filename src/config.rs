use std::collections::hash_map::HashMap;
use std::env::home_dir;
use std::fs::{File};
use std::io::{BufRead,BufReader};
use std::path::{Component,Path,PathBuf};
use result::{self,FdownError};

#[derive(Debug)]
pub struct ConfigFile {
  values: HashMap<String,String>
}

fn twiddle(filename: &str) -> PathBuf {
  let file_path = Path::new(filename);
  let mut components = file_path.components();
  if let Some(first) = components.nth(0) {
    if let Component::Normal(twiddle) = first {
      if twiddle == "~" {
        if let Some(mut home) = home_dir() {
          // TODO: return an error if you can't get the home dir.
          home.push(components.as_path());
          return home;
        } 
      }
    }
  }

  // TODO: maybe make this return nothing when no twiddle expansion has happened.
  file_path.to_path_buf()
}

pub fn new(filename: &str) -> result::Result<ConfigFile> {
  let f = try!(File::open(twiddle(filename)));
  let mut hash = HashMap::new();
  let f = BufReader::new(f);

  for line in f.lines() {
    if let Ok(line) = line {
      let (k, v) = try!(split_line(&line));
      hash.insert(k.to_string(), v.to_string());
    }
  }
  Ok(ConfigFile{values: hash})
}

impl ConfigFile {
  pub fn required_string(&self, k: &str) -> result::Result<&String> {
    self.values.get(k).ok_or(
        FdownError::BadConfig(format!("Required config value, {}, missing", k)))
  }
}

fn split_line<'a>(line: &'a str) -> result::Result<(&'a str, &'a str)> {
  if let Some(pos) = line.find('=') {
    let key = line[..pos].trim();
    let value = line[pos+1..].trim();
    return Ok((key, value))
  }

  Err(FdownError::BadConfig(format!("Missing '=' in config file: \"{}\"", line)))
}