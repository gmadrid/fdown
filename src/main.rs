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
use std::fs::File;
use std::io::Read;
use std::io::Write;
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

    // let client = Client::new();
    // let url = format!("http://cloud.feedly.com/v3/streams/ids?streamId={}", self.saved_feed());
    // let response = try!(client.get(&url)
    //     .header(self.auth_header())
    //     .send());
    // let response : StreamsIdsResponse = try!(serde_json::from_reader(response));


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
    let mut foo = try!(File::create("/tmp/myimage.jpg"));
    foo.write(&image_bytes);
  }
  Ok(())
}

fn main() {
  let args = args::parse();
  let config = config::new(args.config_file_location()).unwrap();

  let userid = config.required_string("userid");
  let token = config.required_string("token");
  let feedly = feedly::new(userid, token);
  // let ids = feedly.saved_entry_ids().unwrap();

  let ids = vec!(
    "cfBX1FTyBgWMD47LB+mDBO8xvSRPMYW+Yf70hpffjGI=_156bd0d2737:20fa60f:45cbc242".to_string(),
    "cfBX1FTyBgWMD47LB+mDBO8xvSRPMYW+Yf70hpffjGI=_1570b3456ad:b7c503e:e3157ec0".to_string(),
    "cfBX1FTyBgWMD47LB+mDBO8xvSRPMYW+Yf70hpffjGI=_156b6ba88b8:767b5a:e0992bbc".to_string()
    );
  let details = feedly.detail_for_entries(ids).unwrap();
  for detail in details.iter() {
    process_entry(detail);
  }
}
