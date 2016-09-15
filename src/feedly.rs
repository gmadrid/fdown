use generated::{EntryDetail,MarkerRequestBody,StreamsIdsResponse,SubscriptionDetail};
use hyper;
use hyper::Client;
use hyper::header;
use regex::Regex;
use result;
use serde_json;

pub struct FeedlyInternal<'f, T> where T: HttpMockableClient {
  userid: String,
  token: String,
  client: &'f T
}

impl<'f, T> FeedlyInternal<'f, T> where T: HttpMockableClient {
  pub fn new<'a>(userid: &str, token: &str) -> FeedlyInternal<'a, T> {
    FeedlyInternal { userid: userid.to_string(), token: token.to_string() }
  }

  fn saved_feed(&self) -> String {
    format!("user/{}/tag/global.saved", self.userid)
  }

  fn auth_header(&self) -> header::Authorization<String> {
    header::Authorization(format!("OAuth {}", self.token).to_owned())
  }

  pub fn saved_entry_ids(&self, count: usize) -> result::Result<Vec<String>> {  
    let url = format!("http://cloud.feedly.com/v3/streams/ids?streamId={}&count={}", self.saved_feed(), count);
    let response = try!(HyperClientWrapper{}.get(self, url.as_str(), true));
    let ids_response : StreamsIdsResponse = try!(serde_json::from_reader(response));
    Ok(ids_response.ids)
  }

  pub fn unsave_entries(&self, entries: &Vec<&EntryDetail>) -> result::Result<()> {
    let url = "http://cloud.feedly.com/v3/markers";
    let entry_ids : Vec<String> = entries.iter().map(|e| e.id.clone()).collect();
    let body_struct = MarkerRequestBody{ 
        action: "markAsUnsaved".to_string(), 
        type_field: "entries".to_string(), 
        entry_ids: entry_ids};
    let body : Vec<u8> = try!(serde_json::to_vec(&body_struct));
    try!(HyperClientWrapper{}.post(self, url, true, body.as_slice()));
    Ok(())
  }

  pub fn subscriptions(&self) -> result::Result<Vec<SubscriptionDetail>> {
    let response = try!(HyperClientWrapper{}.get(self, "http://cloud.feedly.com/v3/subscriptions", true));
    let detail : Vec<SubscriptionDetail> = try!(serde_json::from_reader(response));
    Ok(detail)
  }

  pub fn detail_for_entries(&self, ids: Vec<String>) -> result::Result<Vec<EntryDetail>> {
    let url = "http://cloud.feedly.com/v3/entries/.mget";
    let quoted : Vec<String> = ids.into_iter().map(|i| "\"".to_string() + &i + "\"").collect();
    let body = "[".to_string() + &quoted.join(",") + "]";

    let response = try!(HyperClientWrapper{}.post(self, url, false, body.as_bytes()));
    let detail : Vec<EntryDetail> = try!(serde_json::from_reader(response));

    Ok(detail)
  }

  pub fn extract_image_url<'a>(detail: &'a EntryDetail) -> Option<&'a String> {
    if let Some(ref visual) = detail.visual {
      if let Some(ref url) = visual.url {
        return Some(url);
      }
    }
    None
  }

  pub fn tumblr_filter(url: &str) -> String {
    // TODO: seems wasteful to create this regex every time.
    let regex = Regex::new(r"_(\d+)(\.[:alnum:]+)$").unwrap();
    return regex.replace(url, "_1280$2");
  }
}

trait HttpMockableClient {
  fn get(&self, feedly: &FeedlyInternal<Self>, url: &str, auth: bool) -> hyper::error::Result<hyper::client::Response>;
  fn post(&self, feedly: &FeedlyInternal<Self>, url: &str, auth: bool, body: &[u8]) -> hyper::error::Result<hyper::client::Response>;
}

struct HyperClientWrapper {}

impl HttpMockableClient for HyperClientWrapper {
  fn get(&self, feedly: &FeedlyInternal<Self>, url: &str, auth: bool) -> hyper::error::Result<hyper::client::Response> {
    let client = Client::new();
    let mut builder = client.get(url);
    if auth {
      builder = builder.header(feedly.auth_header());
    }
    builder.send()
  }

  fn post(&self, feedly: &FeedlyInternal<Self>, url: &str, auth: bool, body: &[u8]) -> hyper::error::Result<hyper::client::Response> {
    let client = Client::new();
    let mut builder = client.post(url).body(body);
    if auth {
      builder = builder.header(feedly.auth_header());
    }
    builder.send()
  }
}
