use generated::{EntryDetail,MarkerRequestBody,StreamsIdsResponse,SubscriptionDetail};
use hyper;
use hyper::Client;
use hyper::header;
use regex::Regex;
use result;
use serde_json;

pub type Feedly = FeedlyInternal<HyperClientWrapper>;

pub struct FeedlyInternal<T> where T: HttpMockableClient {
  userid: String,
  token: String,
  client: T
}

impl<T> FeedlyInternal<T> where T: HttpMockableClient {
  pub fn new(userid: &str, token: &str) -> FeedlyInternal<HyperClientWrapper> {
    FeedlyInternal::new_with_client(userid, token, HyperClientWrapper{} )
  }

  fn new_with_client(userid: &str, token: &str, client: T) -> FeedlyInternal<T> {
    FeedlyInternal { userid: userid.to_string(), token: token.to_string(), client: client }
  }

  fn saved_feed(&self) -> String {
    format!("user/{}/tag/global.saved", self.userid)
  }

  fn auth_header(&self) -> header::Authorization<String> {
    header::Authorization(format!("OAuth {}", self.token).to_owned())
  }

  pub fn saved_entry_ids(&self, count: usize) -> result::Result<Vec<String>> {  
    let url = format!("http://cloud.feedly.com/v3/streams/ids?streamId={}&count={}", self.saved_feed(), count);
    let response = try!(self.client.get(url.as_str(), Some(self.auth_header())));
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
    try!(self.client.post(url, Some(self.auth_header()), body.as_slice()));
    Ok(())
  }

  pub fn subscriptions(&self) -> result::Result<Vec<SubscriptionDetail>> {
    let response = try!(self.client.get("http://cloud.feedly.com/v3/subscriptions", Some(self.auth_header())));
    let detail : Vec<SubscriptionDetail> = try!(serde_json::from_reader(response));
    Ok(detail)
  }

  pub fn detail_for_entries(&self, ids: Vec<String>) -> result::Result<Vec<EntryDetail>> {
    let url = "http://cloud.feedly.com/v3/entries/.mget";
    let quoted : Vec<String> = ids.into_iter().map(|i| "\"".to_string() + &i + "\"").collect();
    let body = "[".to_string() + &quoted.join(",") + "]";

    let response = try!(self.client.post(url, None, body.as_bytes()));
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

pub trait HttpMockableClient {
  fn get(&self, url: &str, authHeader: Option<header::Authorization<String>>) 
      -> hyper::error::Result<hyper::client::Response>;
  fn post(&self, url: &str, authHeader: Option<header::Authorization<String>>, body: &[u8]) 
      -> hyper::error::Result<hyper::client::Response>;
}

pub struct HyperClientWrapper {}

impl HttpMockableClient for HyperClientWrapper {
  fn get(&self, url: &str, auth_header: Option<header::Authorization<String>>) -> hyper::error::Result<hyper::client::Response> {
    let client = Client::new();
    let mut builder = client.get(url);
    match auth_header {
      Some(h) => builder = builder.header(h),
      None => {}
    }
    builder.send()
  }

  fn post(&self, url: &str, auth_header: Option<header::Authorization<String>>, body: &[u8]) -> hyper::error::Result<hyper::client::Response> {
    let client = Client::new();
    let mut builder = client.post(url).body(body);
    match auth_header {
      Some(h) => builder = builder.header(h),
      None => {}
    }
    builder.send()
  }
}

#[cfg(test)]
mod tests {

  struct NullClient;
//  impl HttpMockableClient for NullClient {

//  }

  use super::*;

  const TEST_USERID : &'static str = "test_userid";
  const TEST_TOKEN : &'static str = "test_token";

  #[test]
  fn saved_feed() {
//    assert_equal(FeedlyInternal::saved_feed(TEST_USERID), "seasrt");
  }

}
