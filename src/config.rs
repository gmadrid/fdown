use result::{FdownError, Result};
use std::collections::hash_map::HashMap;
use std::env::home_dir;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Component, Path, PathBuf};

#[derive(Debug)]
pub struct ConfigFile {
  values: HashMap<String, String>,
}

impl ConfigFile {
  pub fn new(filename: &str) -> Result<ConfigFile> {
    let f = try!(File::open(twiddle(filename, BaseHomedirProvider {}).unwrap_or(PathBuf::from(filename))));
    let reader = BufReader::new(f);

    ConfigFile::new_with_bufread(reader)
  }

  fn new_with_bufread<T>(reader: T) -> Result<ConfigFile>
    where T: BufRead {
    let mut hash = HashMap::new();

    for line in reader.lines() {
      if let Ok(line) = line {
        let trimmed = line.trim();
        if trimmed.starts_with("#") || trimmed.len() == 0 {
          continue;
        }
        let (k, v) = try!(split_line_at_first_equals(&line));
        hash.insert(k.to_string(), v.to_string());
      }
    }
    Ok(ConfigFile { values: hash })
  }

  pub fn required_string(&self, k: &str) -> Result<&String> {
    self.values
      .get(k)
      .ok_or(FdownError::BadConfig(format!("Required config value, {}, missing", k)))
  }
}

fn twiddle<T>(filename: &str, home_dir_provider: T) -> Option<PathBuf>
  where T: HasHomedir {
  let file_path = Path::new(filename);
  let mut components = file_path.components();
  if let Some(first) = components.nth(0) {
    if let Component::Normal(twiddle) = first {
      if twiddle == "~" {
        let mut home = home_dir_provider.home_dir();
        home.push(components.as_path());
        return Some(home);
      }
    }
  }
  None
}

fn split_line_at_first_equals(line: &str) -> Result<(&str, &str)> {
  if let Some(pos) = line.find('=') {
    let (head, tail) = line.split_at(pos);
    return Ok((head.trim(), &tail[1..].trim()));
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
  use std::path::PathBuf;
  use super::{HasHomedir, split_line_at_first_equals};

  #[test]
  fn simple_reader() {
    let reader = "foo=bar\nquux=bam".as_bytes();
    let config = super::ConfigFile::new_with_bufread(reader).unwrap();
    assert_eq!("bar", config.required_string("foo").unwrap());
    assert_eq!("bam", config.required_string("quux").unwrap());
  }

  #[test]
  fn comments_blank_lines() {
    let reader = "\nquux=bam\n# a comment\nfoo=bar\n".as_bytes();
    let config = super::ConfigFile::new_with_bufread(reader).unwrap();
    assert_eq!("bar", config.required_string("foo").unwrap());
    assert_eq!("bam", config.required_string("quux").unwrap());
  }

  #[test]
  fn split_line_basic() {
    assert_eq!(("foo", "bar"), split_line_at_first_equals("foo=bar").unwrap());
    assert_eq!(("foo", "bar"), split_line_at_first_equals("foo = bar").unwrap());
    assert_eq!(("foo", "bar"), split_line_at_first_equals("foo =bar").unwrap());
    assert_eq!(("foo", "bar"), split_line_at_first_equals("foo= bar").unwrap());
    assert_eq!(("foo", "bar"), split_line_at_first_equals(" foo=bar ").unwrap());
    assert_eq!(("foo", "bar"), split_line_at_first_equals("   foo =  bar    ").unwrap());

    assert_eq!(("foo", ""), split_line_at_first_equals("foo=").unwrap());
  }

  #[test]
  #[should_panic]
  fn split_line_with_no_equals() {
    split_line_at_first_equals("foo bar").unwrap();
  }

  struct TestHomeDirProvider {}
  impl HasHomedir for TestHomeDirProvider {
    fn home_dir(&self) -> PathBuf {
      return PathBuf::from("/foo/bar/home".to_string());
    }
  }

  struct FailingHomeDirProvider {}
  impl HasHomedir for FailingHomeDirProvider {
    fn home_dir(&self) -> PathBuf {
      assert!(false);
      PathBuf::from("/never/returns/this")
    }
  }

  #[test]
  fn twiddle() {
    assert_eq!(None, super::twiddle("/quux", TestHomeDirProvider {}));
    assert_eq!(Some(PathBuf::from("/foo/bar/home/quux")),
               super::twiddle("~/quux", TestHomeDirProvider {}));
    assert_eq!(None, super::twiddle("~quux", TestHomeDirProvider {}));
    assert_eq!(None, super::twiddle("/foo/~/bar", TestHomeDirProvider {}));
  }

  #[test]
  #[should_panic]
  fn twiddle_no_home_dir() {
    super::twiddle("~/quux", FailingHomeDirProvider {});
  }

  #[test]
  fn twiddle_no_home_dir_with_absolute_path() {
    super::twiddle("/quux", FailingHomeDirProvider {});
  }
}
