use clap::{Arg, App, ArgMatches};

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
  pub fn parse() -> Args<'a> {
    Args { matches: parse_cmd_line() }
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

fn parse_cmd_line<'a>() -> ArgMatches<'a> {
  App::new("fdown")
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
          .requires(CATEGORY))
      .get_matches()
}
