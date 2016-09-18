use generated::DropboxUploadAPI;
use hyper::{Client, header};
use hyper::header::ContentType;
use hyper::mime::{Mime, SubLevel, TopLevel};
use result::Result;
use serde_json;
use std::io::Read;

header!{ (DropboxAPIArg, "Dropbox-API-Arg") => [String] }

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
    let header_value: Vec<u8> = serde_json::to_vec(&api).unwrap();
    let header_string = String::from_utf8(header_value).unwrap();
    let escaped = header_string;//.replace("\"", "\\\"");
    DropboxAPIArg(escaped)
  }

  pub fn upload(&self, path: &str, contents: &[u8]) -> Result<()> {
    let api = DropboxUploadAPI {
      path: path.to_string(),
      mode: "add".to_string(),
      autorename: true,
      mute: false,
    };
    let url = "https://content.dropboxapi.com/2/files/upload";
    let client = Client::new();
    let request = client.post(url)
      .body(contents)
      .header(ContentType(Mime(TopLevel::Application, SubLevel::OctetStream, vec![])))
      .header(self.auth_header())
      .header(self.api_header(&api));
    let mut response = try!(request.send());
    let mut foo: String = String::new();
    response.read_to_string(&mut foo);
    Ok(())
  }
}
