use generated::{EntryDetail,StreamsIdsResponse};
use hyper::Client;
use hyper::header;
use result;
use serde_json;

pub struct Feedly {
  userid: String,
  token: String
}

pub fn new(userid: &str, token: &str) -> Feedly {
  Feedly { userid: userid.to_string(), token: token.to_string() }
}

impl Feedly {
  fn saved_feed(&self) -> String {
    format!("user/{}/tag/global.saved", self.userid)
  }

  fn auth_header(&self) -> header::Authorization<String> {
    header::Authorization(format!("OAuth {}", self.token).to_owned())
  }

  pub fn saved_entry_ids(&self) -> result::Result<Vec<String>> {
    let client = Client::new();
    let url = format!("http://cloud.feedly.com/v3/streams/ids?streamId={}", self.saved_feed());
    let response = try!(client.get(&url)
        .header(self.auth_header())
        .send());
    let response : StreamsIdsResponse = try!(serde_json::from_reader(response));
    Ok(response.ids)
  }

  pub fn detail_for_entries(&self, ids: Vec<String>) -> result::Result<Vec<EntryDetail>> {
    let client = Client::new();
    let url = "http://cloud.feedly.com/v3/entries/.mget";
    let quoted : Vec<String> = ids.into_iter().map(|i| "\"".to_string() + &i + "\"").collect();
    let body = "[".to_string() + &quoted.join(",") + "]";
    let builder = client.post(url);
    let response = try!(builder.body(body.as_bytes()).send());
    let detail : Vec<EntryDetail> = try!(serde_json::from_reader(response));

    Ok(detail)
  }

  pub fn extract_image_url(&self, detail: EntryDetail) -> Option<String> {
    detail.visual.and_then(|v| v.url)
  }
}