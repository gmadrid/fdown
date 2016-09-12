use clap::{Arg, App, ArgMatches};

static CONFIG: &'static str = "config";
static DEFAULT_CONFIG: &'static str = "~/.fdown";

pub struct Args<'a> {
  matches: ArgMatches<'a>
}

impl<'a> Args<'a> {
  pub fn config_file_location(&self) -> &str {
    // Return something else and error if file not found
    return self.matches.value_of(CONFIG).unwrap_or(DEFAULT_CONFIG);
  }
}

pub fn parse<'a>() -> Args<'a> {
  Args { matches: parse_cmd_line() }
}

fn parse_cmd_line<'a>() -> ArgMatches<'a> {
  App::new("fdown")
      .version("0.0.1")
      .author("George Madrid (gmadrid@gmail.com)")
      .arg(Arg::with_name(CONFIG)
          .long(CONFIG)
          .takes_value(true)
          .help("Location of the config file"))
      .get_matches()
}
