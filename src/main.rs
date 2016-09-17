extern crate clap;
extern crate hyper;
extern crate regex;
extern crate serde_json;

mod args;
mod config;
mod feedly;
mod generated;
mod result;

use config::ConfigFile;
use feedly::Feedly;
use generated::EntryDetail;
use hyper::Client;
use result::{FdownError, Result};
use std::ffi::OsStr;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::{Path, PathBuf};

fn download_image(url: &String) -> Result<(Vec<u8>)> {
  let client = Client::new();
  let mut response = try!(client.get(url).send());
  let mut buf: Vec<u8> = Vec::new();
  try!(response.read_to_end(&mut buf));
  Ok(buf)
}

fn add_number_suffix(stem: &OsStr, num: usize) -> Result<String> {
  if let Some(stem) = stem.to_str() {
    return Ok(format!("{}_({})", stem, num));
  }
  Err(result::FdownError::BadFormat(format!("cannot append number to stem: {}",
                                            stem.to_string_lossy())))
}

fn file_new_path(path: PathBuf) -> Result<PathBuf> {
  let mut num = 0;
  let extension = path.extension().unwrap_or(OsStr::new(""));
  if let Some(stem) = path.file_stem() {
    loop {
      num += 1;
      let new_name = try!(add_number_suffix(stem, num));
      let new_path = path.with_file_name(new_name).with_extension(extension);
      if !new_path.exists() {
        return Ok(new_path);
      }
    }
  }
  Err(FdownError::BadFormat(format!("unable to find file stem in url: {}",
                                    path.to_string_lossy())))
}

fn filepath_for_url(url: &String) -> Result<PathBuf> {
  // Get the likely filename from the url.
  // TODO: check to ensure that the filename has an extension.
  if let Some(slash_index) = url.rfind('/') {
    let filename = Path::new(&url[slash_index + 1..]);
    let path = Path::new("/Users/gmadrid/Dropbox/ATestDir").join(filename);
    if path.exists() {
      return file_new_path(path);
    }
    return Ok(path);
  }
  Err(result::FdownError::BadFormat(format!("unable to extract filename from url: {}", url)))
}

fn unsave_entries(entries: &Vec<&EntryDetail>, feedly: &Feedly) -> Result<()> {
  feedly.unsave_entries(entries)
}

fn write_entry(entry: &EntryDetail) -> Result<()> {
  if let Some(url) = Feedly::extract_image_url(entry) {
    let url = Feedly::tumblr_filter(url);
    let image_bytes = try!(download_image(&url));
    let path = try!(filepath_for_url(&url));
    let mut file = try!(File::create(path));
    try!(file.write(&image_bytes));
    return Ok(());
  }
  Err(result::FdownError::MissingUrl(entry.id.clone()))
}

fn list_subs(feedly: &Feedly) -> Result<()> {
  let subs = try!(feedly.subscriptions());
  for sub in subs {
    // TODO: print something better.
    let title = sub.title.unwrap_or(sub.id);
    let first_cat = sub.categories.first();
    match first_cat {
      Some(cat) => {
        let cat_title = cat.label.as_ref().unwrap_or(&cat.id);
        println!("{}: {}", cat_title, title);
      }
      None => println!("{}", title),
    }
  }
  Ok(())
}

fn filter_for_category(category: Option<&str>,
                       feedly: &Feedly)
    -> Result<Box<Fn(&EntryDetail) -> bool>> {
  if let Some(category) = category {
    let subs = try!(feedly.subscriptions());
    let stream_ids: Vec<String> = subs.iter()
      .filter(|sub| {
        let categories = &sub.categories;
        categories.iter().any(|cat| cat.label.as_ref().map_or(false, |label| label == category))
      })
      .map(|sub| sub.id.to_string())
      .collect();

    let closure = move |entry: &EntryDetail| -> bool {
      if let Some(ref origin) = entry.origin {
        return stream_ids.iter().any(|stream_id| stream_id == &origin.stream_id);
      }
      false
    };
    return Ok(Box::new(closure));
  }
  Ok(Box::new(|_| true))
}

fn get_entries(filter_func: &Fn(&EntryDetail) -> bool,
               count: usize,
               feedly: &Feedly)
    -> Result<Vec<EntryDetail>> {
  // TODO: give this a count param
  // TODO: keep continuing until you have count entries
  let ids = try!(feedly.saved_entry_ids(count));
  let entries = try!(feedly.detail_for_entries(ids));
  let res: Vec<EntryDetail> = entries.into_iter().filter(filter_func).collect();
  Ok(res)
}

fn real_main() -> Result<()> {
  let args = try!(args::Args::parse());
  let config = try!(ConfigFile::new(args.config_file_location()));

  let userid = try!(config.required_string("userid"));
  let token = try!(config.required_string("token"));

  let feedly = Feedly::new(userid, token);

  if args.list_subs() {
    return list_subs(&feedly);
  }

  let filter = try!(filter_for_category(args.filter_category(), &feedly));
  let entries = try!(get_entries(filter.as_ref(), args.number_of_entries(), &feedly));
  let mut successful_entries: Vec<&EntryDetail> = Vec::with_capacity(entries.len());
  for (i, entry) in entries.iter().enumerate() {
    println!("Processing entry {}.", i);
    match write_entry(entry) {
      Ok(_) => successful_entries.push(entry),
      Err(e) => return Err(e),
    }
  }
  if args.should_unsave() {
    try!(unsave_entries(&successful_entries, &feedly));
  }

  Ok(())
}

fn main() {
  match real_main() {
    Ok(_) => (),
    Err(err) => {
      match err {
        // Clap gets special attention. ('-h' for example is better handled by clap::Error::exit())
        result::FdownError::Clap(ce) => clap::Error::exit(&ce),
        _ => println!("{:?}", err),
      }
    }
  }
}
