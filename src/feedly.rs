use generated::{EntryDetail,MarkerRequestBody,StreamsIdsResponse,SubscriptionDetail};
use hyper;
use hyper::Client;
use hyper::header;
use regex::Regex;
use result;
use serde_json;
use std::io::Read;

pub type Feedly = FeedlyInternal<HyperClientWrapper>;

pub struct FeedlyInternal<T> where T: HttpMockableClient {
  userid: String,
  token: String,
  client: T
}

impl<T> FeedlyInternal<T> where T: HttpMockableClient {
  pub fn new(userid: &str, token: &str) -> FeedlyInternal<HyperClientWrapper> {
    FeedlyInternal::<HyperClientWrapper>::new_with_client(userid, token, HyperClientWrapper{})
  }

  fn new_with_client<C>(userid: &str, token: &str, client: C) 
      -> FeedlyInternal<C> where C: HttpMockableClient {
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
  type R: Read;

  fn get(&self, url: &str, authHeader: Option<header::Authorization<String>>) 
      -> result::Result<Self::R>;
  fn post(&self, url: &str, authHeader: Option<header::Authorization<String>>, body: &[u8]) 
      -> result::Result<Self::R>;
}

pub struct HyperClientWrapper {}

impl HttpMockableClient for HyperClientWrapper {
  type R = hyper::client::Response;

  fn get(&self, url: &str, auth_header: Option<header::Authorization<String>>) -> result::Result<Self::R> {
    let client = Client::new();
    let mut builder = client.get(url);
    match auth_header {
      Some(h) => builder = builder.header(h),
      None => {}
    }
    builder.send().map_err(|e| result::FdownError::from(e))
  }

  fn post(&self, url: &str, auth_header: Option<header::Authorization<String>>, body: &[u8]) -> result::Result<Self::R> {
    let client = Client::new();
    let mut builder = client.post(url).body(body);
    match auth_header {
      Some(h) => builder = builder.header(h),
      None => {}
    }
    builder.send().map_err(|e| result::FdownError::from(e))
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use hyper::header;
  use result;
  use std::cell::{Cell,RefCell};
  use std::convert::From;
  use std::io::Cursor;

  const TEST_USERID : &'static str = "test_userid";
  const TEST_TOKEN : &'static str = "test_token";

  type MockFeedly<'a> = FeedlyInternal<NullClient<'a>>;

  struct NullClient<'a> {
    responses: Vec<&'a str>,
    url: RefCell<Option<String>>,
    has_auth: Cell<bool>
  }
  impl<'a> NullClient<'a> {
    fn check_url(&self, url: &str) {
      assert_eq!(url, self.url.borrow().as_ref().unwrap());
    }
    fn check_has_auth(&self, val: bool) {
      assert_eq!(val, self.has_auth.get());
    }
  }
  impl<'a> HttpMockableClient for NullClient<'a> {
    type R = Cursor<Vec<u8>>;

    fn get(&self, url: &str, auth_header: Option<header::Authorization<String>>) 
        -> result::Result<Self::R> {
      if self.responses.len() < 1 {
        return Err(result::FdownError::TestError);
      }
      let bytes = self.responses.get(0).unwrap().as_bytes();
      let vec : Vec<u8> = From::from(bytes);
      let cursor = Cursor::new(vec);
      *self.url.borrow_mut() = Some(url.to_string());
      self.has_auth.set(auth_header.is_some());
      Ok(cursor)
    }
    fn post(&self, url: &str, auth_header: Option<header::Authorization<String>>, body: &[u8]) 
        -> result::Result<Self::R> {
      unimplemented!();
    }
  }

  fn null_client<'a>(responses: Vec<&'a str>) -> MockFeedly<'a> {
    Feedly::new_with_client(TEST_USERID, TEST_TOKEN, NullClient{ 
        responses: responses, url: RefCell::new(None), has_auth: Cell::new(false)
    })
  }

  #[test]
  fn saved_feed() {
    assert_eq!(null_client(vec!()).saved_feed(), "user/test_userid/tag/global.saved");
  }

  #[test]
  fn auth_header() {
    let header::Authorization(s) = null_client(vec!()).auth_header();
    assert_eq!("OAuth test_token", s);
  }

  #[test]
  fn saved_entry_ids() {
    let resp = "{ \"ids\": [ \"id1\", \"id2\", \"id3\" ], \"continuation\": \"continuation\" }";
    let feedly = null_client(vec!(resp));
    let ids = feedly.saved_entry_ids(5).unwrap();
    feedly.client.check_has_auth(true);
    feedly.client.check_url("http://cloud.feedly.com/v3/streams/ids?streamId=user/test_userid/tag/global.saved&count=5");
    assert_eq!(3, ids.len());
  }

  #[test]
  fn saved_entry_ids_bad_http() {
    let feedly = null_client(vec!());
    feedly.saved_entry_ids(5).unwrap_err();
  }

  #[test]
  fn saved_entry_ids_bad_json() {
    let resp = "{ ids: [ \"id1\", \"id2\", \"id3\" ], \"continuation\": \"continuation\" }";
    let feedly = null_client(vec!(resp));
    feedly.saved_entry_ids(5).unwrap_err();
  }
}
