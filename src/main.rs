extern crate clap;
extern crate hyper;
extern crate regex;
extern crate serde_json;

mod args;
mod config;
mod feedly;
mod generated;
mod result;

use feedly::{Feedly};
use generated::EntryDetail;
use hyper::Client;
use std::ffi::OsStr;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::{Path,PathBuf};
use std::process::Command;

fn open_url(url: &String) -> result::Result<()> {
  let mut cmd = Command::new("open");
  cmd.arg(url);
  try!(cmd.spawn());
  Ok(())
}

fn download_image(url: &String) -> result::Result<(Vec<u8>)> {
  let client = Client::new();
  let mut response = try!(client.get(url).send());
  let mut buf : Vec<u8> = Vec::new();
  try!(response.read_to_end(&mut buf));
  Ok(buf)
}

fn add_number_suffix(stem: &OsStr, num: usize) -> result::Result<String> {
  if let Some(stem) = stem.to_str() {
    return Ok(format!("{}_({})", stem, num))
  }
  Err(result::FdownError::BadFormat(format!("cannot append number to stem: {}", stem.to_string_lossy())))
}

fn file_new_path(path: PathBuf) -> result::Result<PathBuf> {
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
  Err(result::FdownError::BadFormat(format!("unable to find file stem in url: {}", path.to_string_lossy())))
}

fn filepath_for_url(url: &String) -> result::Result<PathBuf> {
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

fn unsave_entry(entry: &EntryDetail) -> result::Result<()> {
  // TODO: do this.
  Ok(())
}

fn process_entry(entry: &EntryDetail) -> result::Result<()> {
  // For each, 
  // 1) get the url
  // 2) if it's a tumblr url, get the big version
  // 3) download the image 
  // 4) save to Dropbox (with noclobber)
  // 5) unsave
  if let Some(url) = Feedly::extract_image_url(entry) {
    let url = Feedly::tumblr_filter(url);
    let image_bytes = try!(download_image(&url));
    let path = try!(filepath_for_url(&url));
    let mut foo = try!(File::create(path));
    try!(foo.write(&image_bytes));
    try!(unsave_entry(entry));
  }
  Ok(())
}

fn main() {
  let args = args::parse();
  let config = config::new(args.config_file_location()).unwrap();

  let userid = config.required_string("userid");
  let token = config.required_string("token");
  let feedly = feedly::new(userid, token);
  let ids = feedly.saved_entry_ids().unwrap();

  // let ids = vec!(
  //   "cfBX1FTyBgWMD47LB+mDBO8xvSRPMYW+Yf70hpffjGI=_156bd0d2737:20fa60f:45cbc242".to_string(),
  //   "cfBX1FTyBgWMD47LB+mDBO8xvSRPMYW+Yf70hpffjGI=_1570b3456ad:b7c503e:e3157ec0".to_string(),
  //   "cfBX1FTyBgWMD47LB+mDBO8xvSRPMYW+Yf70hpffjGI=_156b6ba88b8:767b5a:e0992bbc".to_string()
  //   );
  let details = feedly.detail_for_entries(ids).unwrap();
  for detail in details.iter() {
    process_entry(detail).map_err(|err| println!("{:?}", err));
  }
}
