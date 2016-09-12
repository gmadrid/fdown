extern crate clap;
extern crate hyper;
extern crate serde_json;

mod args;
mod config;
mod feedly;
mod generated;
mod result;

use std::process::Command;

fn open_url(url: String) -> result::Result<()> {
  let mut cmd = Command::new("open");
  cmd.arg(url);
  try!(cmd.spawn());
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
    "cfBX1FTyBgWMD47LB+mDBO8xvSRPMYW+Yf70hpffjGI=_1570b3456ad:b7c503e:e3157ec0".to_string()
    );
  let details = feedly.detail_for_entries(ids).unwrap();
  for detail in details {
    feedly.extract_image_url(detail).map(open_url);
  }
}
