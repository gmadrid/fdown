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

impl ConfigFile {
  pub fn new(filename: &str) -> result::Result<ConfigFile> {
    let f = try!(File::open(twiddle(filename,  BaseHomedirProvider{})));
    let reader = BufReader::new(f);

    ConfigFile::new_with_bufread(reader)
  }

  fn new_with_bufread<T>(reader: T) 
      -> result::Result<ConfigFile> where T: BufRead {
    let mut hash = HashMap::new();

    for line in reader.lines() {
      if let Ok(line) = line {
        let trimmed = line.trim();
        if trimmed.starts_with("#") || trimmed.len() == 0 {
          continue;
        }
        let (k, v) = try!(split_line(&line));
        hash.insert(k.to_string(), v.to_string());
      }
    }
    Ok(ConfigFile{values: hash})
  }

  pub fn required_string(&self, k: &str) -> result::Result<&String> {
    self.values.get(k).ok_or(
        FdownError::BadConfig(format!("Required config value, {}, missing", k)))
  }
}

fn twiddle<T>(filename: &str, home_dir_provider: T) -> PathBuf where T: HasHomedir {
  let file_path = Path::new(filename);
  let mut components = file_path.components();
  if let Some(first) = components.nth(0) {
    if let Component::Normal(twiddle) = first {
      if twiddle == "~" {
        let mut home = home_dir_provider.home_dir();
        home.push(components.as_path());
        return home;
      }
    }
  }

  // TODO: maybe make this return nothing when no twiddle expansion has happened.
  file_path.to_path_buf()
}

fn split_line<'a>(line: &'a str) -> result::Result<(&'a str, &'a str)> {
  if let Some(pos) = line.find('=') {
    let key = line[..pos].trim();
    let value = line[pos+1..].trim();
    return Ok((key, value))
  }

  Err(FdownError::BadConfig(format!("Missing '=' in config file: \"{}\"", line)))
}

trait HasHomedir {
  fn home_dir(&self) -> PathBuf;
}

struct BaseHomedirProvider;

impl HasHomedir for BaseHomedirProvider {
  fn home_dir(&self) -> PathBuf {
    return home_dir().unwrap();
  }
}


#[cfg(test)]
mod tests {
  use super::{HasHomedir,split_line};
  use std::path::PathBuf;

  #[test]
  fn simple_reader() {
    let reader = "foo=bar\n\
                  quux=bam".as_bytes();
    let config = super::ConfigFile::new_with_bufread(reader).unwrap();
    assert_eq!("bar", config.required_string("foo").unwrap());
    assert_eq!("bam", config.required_string("quux").unwrap());
  }

  #[test]
  fn comments_blank_lines() {
    let reader = "\n\
                  quux=bam\n\
                  # a comment\n\
                  foo=bar\n".as_bytes();
    let config = super::ConfigFile::new_with_bufread(reader).unwrap();
    assert_eq!("bar", config.required_string("foo").unwrap());
    assert_eq!("bam", config.required_string("quux").unwrap());
  }

  #[test]
  fn split_line_basic() {
    assert_eq!(("foo", "bar"), split_line("foo=bar").unwrap());
    assert_eq!(("foo", "bar"), split_line("foo = bar").unwrap());
    assert_eq!(("foo", "bar"), split_line("foo =bar").unwrap());
    assert_eq!(("foo", "bar"), split_line("foo= bar").unwrap());
    assert_eq!(("foo", "bar"), split_line(" foo=bar ").unwrap());
    assert_eq!(("foo", "bar"), split_line("   foo =  bar    ").unwrap());

    assert_eq!(("foo", ""), split_line("foo=").unwrap());
  }

  #[test]
  #[should_panic]
  fn split_line_no_equals() {
    split_line("foo bar").unwrap();
  }

  struct TestHomeDirProvider{}
  impl HasHomedir for TestHomeDirProvider {
    fn home_dir(&self) -> PathBuf {
      return PathBuf::from("/foo/bar/home".to_string())
    }
  }

  struct FailingHomeDirProvider{}
  impl HasHomedir for FailingHomeDirProvider {
    fn home_dir(&self) -> PathBuf {
      println!("{:?}", "QUUX");
      assert!(false);
      println!("{:?}", "QUUX2");
      PathBuf::from("/never/returns/this")
    }
  }

  #[test]
  fn twiddle() {
    assert_eq!(PathBuf::from("/quux"), super::twiddle("/quux", TestHomeDirProvider{}));
    assert_eq!(PathBuf::from("/foo/bar/home/quux"), super::twiddle("~/quux", TestHomeDirProvider{}));
    assert_eq!(PathBuf::from("~quux"), super::twiddle("~quux", TestHomeDirProvider{}));
    assert_eq!(PathBuf::from("/foo/~/bar"), super::twiddle("/foo/~/bar", TestHomeDirProvider{}));
  }

  #[test]
  #[should_panic]
  fn twiddle_no_home_dir() {
    super::twiddle("~/quux", FailingHomeDirProvider{});
  }

  #[test]
  fn twiddle_no_home_dir_with_absolute_path() {
    super::twiddle("/quux", FailingHomeDirProvider{});
  }
}