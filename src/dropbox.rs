use generated::DropboxUploadAPI;
use hyper::{Client, header};
use hyper::header::ContentType;
use hyper::mime::{Mime, SubLevel, TopLevel};
use result::Result;
use serde_json;

header!{ (DropboxAPIArg, "Dropbox-API-Arg") => [String] }

const UPLOAD_URL : &'static str = "https://content.dropboxapi.com/2/files/upload";
const ADD_UPLOAD_MODE : &'static str = "add";

#[derive(Debug)]
pub struct Dropbox {
  token: String,
}

impl Dropbox {
  pub fn new(token: &str) -> Dropbox {
    Dropbox { token: token.to_string() }
  }

  fn auth_header(&self) -> header::Authorization<String> {
    header::Authorization(format!("Bearer {}", self.token).to_owned())
  }

  fn api_header(&self, api: &DropboxUploadAPI) -> DropboxAPIArg {
    DropboxAPIArg(serde_json::to_string(&api).unwrap())
  }

  pub fn upload(&self, path: &str, contents: &[u8]) -> Result<()> {
    let api = DropboxUploadAPI {
      path: path,
      mode: ADD_UPLOAD_MODE,
      autorename: true,
      mute: false,
    };
    try!(Client::new().post(UPLOAD_URL)
      .body(contents)
      .header(ContentType(Mime(TopLevel::Application, SubLevel::OctetStream, vec![])))
      .header(self.auth_header())
      .header(self.api_header(&api))
      .send());
    Ok(())
  }
}
