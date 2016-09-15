use clap::{Arg, App, ArgMatches};
use result;
use std::env;
use std::ffi::OsString;

static CONFIG: &'static str = "config";
static COUNT: &'static str = "count";
static CATEGORY: &'static str = "category";
static DEFAULT_CONFIG: &'static str = "~/.fdown";
static SUBS: &'static str = "subs";
static UNSAVE: &'static str = "unsave";

pub struct Args<'a> {
  matches: ArgMatches<'a>
}

impl<'a> Args<'a> {
  pub fn parse() -> result::Result<Args<'a>> {
    Args::parse_from(env::args_os())
  }

  fn parse_from<I, T>(itr: I) -> result::Result<Args<'a>>
      where I: IntoIterator<Item=T>, T: Into<OsString> {
    let matches = try!(parse_cmd_line_from(itr));
    Ok(Args { matches: matches })
  }

  pub fn config_file_location(&self) -> &str {
    // Return something else and error if file not found
    return self.matches.value_of(CONFIG).unwrap_or(DEFAULT_CONFIG);
  }

  pub fn list_subs(&self) -> bool {
    self.matches.occurrences_of(SUBS) > 0
  }

  pub fn should_unsave(&self) -> bool {
    self.matches.occurrences_of(UNSAVE) > 0
  }

  pub fn filter_category(&self) -> Option<&str> {
    self.matches.value_of(CATEGORY)
  }

  pub fn number_of_entries(&self) -> usize {
    self.matches.value_of(COUNT).unwrap_or("20").parse::<usize>().unwrap()
  }
}

fn parse_cmd_line_from<'a, I, T>(itr: I) -> result::Result<ArgMatches<'a>>
    where I: IntoIterator<Item=T>, T: Into<OsString> {
  let builder = App::new("fdown")
      .version("0.0.1")
      .author("George Madrid (gmadrid@gmail.com)")
      .arg(Arg::with_name(CATEGORY)
          .short("C")
          .long(CATEGORY)
          .help("Only process entries in this category")
          .takes_value(true))
      .arg(Arg::with_name(CONFIG)
          .long(CONFIG)
          .takes_value(true)
          .help("Location of the config file"))
      .arg(Arg::with_name(COUNT)
          .long(COUNT)
          .takes_value(true)
          .help("Number of entries to download"))
      .arg(Arg::with_name(SUBS)
          .long(SUBS)
          .help("List the subscriptions"))
      .arg(Arg::with_name(UNSAVE)
          .short("U")
          .long(UNSAVE)
          .help("Unsave the entry after saving it.")
          .requires(CATEGORY));

    builder.get_matches_from_safe(itr).map_err(result::FdownError::from)
}

#[cfg(test)]
mod tests {
  use super::*;

  fn args_from<'a, 'b, 'c>(lst: &'a[&'b str]) -> Args<'c> {
    Args::parse_from(lst.iter()).unwrap()
  } 

  #[test]
  fn count() {
    // Test default
    let args = args_from(&["foo"]);
    assert_eq!(20, args.number_of_entries());

    let args = args_from(&["foo", "--count", "57"]);
    assert_eq!(57, args.number_of_entries());
  }

  #[test]
  #[should_panic]
  fn count_missing() {
    args_from(&["foo", "--count"]);
  }

  #[test]
  fn filter_category() {
    let args = args_from(&["foo"]);
    assert_eq!(None, args.filter_category());

    let args = args_from(&["foo", "-C", "quux"]);
    assert_eq!("quux", args.filter_category().unwrap());

    let args = args_from(&["foo", "--category", "bam"]);
    assert_eq!("bam", args.filter_category().unwrap());
  }

  #[test]
  #[should_panic]
  fn filter_category_missing() {
    args_from(&["foo", "-C"]);
  }

  #[test]
  fn subs() {
    let args = args_from(&["foo", "--subs"]);
    assert_eq!(true, args.list_subs());

    let args = args_from(&["foo"]);
    assert_eq!(false, args.list_subs());
  }

  #[test]
  fn unsave() {
    let args = args_from(&["foo", "-C", "cat"]);
    assert_eq!(false, args.should_unsave());

    let args = args_from(&["foo", "-C", "cat", "-U"]);
    assert_eq!(true, args.should_unsave());

    let args = args_from(&["foo", "-C", "cat", "--unsave"]);
    assert_eq!(true, args.should_unsave());
  }

  #[test]
  #[should_panic]
  fn unsave_no_cat() {
    args_from(&["foo", "-U"]);
  }  

  #[test]
  fn config_file_location() {
    let args = Args::parse_from(["foo", "--config", "foobar"].iter()).unwrap();
    assert_eq!("foobar", args.config_file_location());

    // Test default
    let args = Args::parse_from(["foo"].iter()).unwrap();
    assert_eq!("~/.fdown", args.config_file_location());    
  }

  #[should_panic]
  #[test]
  fn config_file_location_missing() {
    Args::parse_from(["foo", "--config"].iter()).unwrap();
  }
}