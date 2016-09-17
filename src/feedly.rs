use generated::{EntryDetail, MarkerRequestBody, StreamsIdsResponse, SubscriptionDetail};
use hyper;
use hyper::Client;
use hyper::header;
use regex::Regex;
use result::{FdownError, Result};
use serde_json;
use std::io::Read;

pub type Feedly = FeedlyInternal<HyperClientWrapper>;

pub struct FeedlyInternal<T>
  where T: HttpMockableClient
{
  userid: String,
  token: String,
  client: T,
}

impl<T> FeedlyInternal<T>
  where T: HttpMockableClient
{
  pub fn new(userid: &str, token: &str) -> FeedlyInternal<HyperClientWrapper> {
    FeedlyInternal::<HyperClientWrapper>::new_with_client(userid, token, HyperClientWrapper {})
  }

  fn new_with_client<C>(userid: &str, token: &str, client: C) -> FeedlyInternal<C>
    where C: HttpMockableClient {
    FeedlyInternal {
      userid: userid.to_string(),
      token: token.to_string(),
      client: client,
    }
  }

  fn saved_feed(&self) -> String {
    format!("user/{}/tag/global.saved", self.userid)
  }

  fn auth_header(&self) -> header::Authorization<String> {
    header::Authorization(format!("OAuth {}", self.token).to_owned())
  }

  pub fn saved_entry_ids(&self, count: usize) -> Result<Vec<String>> {
    let url = format!("http://cloud.feedly.com/v3/streams/ids?streamId={}&count={}",
                      self.saved_feed(),
                      count);
    let response = try!(self.client.get(url.as_str(), Some(self.auth_header())));
    let ids_response: StreamsIdsResponse = try!(serde_json::from_reader(response));
    Ok(ids_response.ids)
  }

  pub fn unsave_entries(&self, entries: &Vec<&EntryDetail>) -> Result<()> {
    let url = "http://cloud.feedly.com/v3/markers";
    let entry_ids: Vec<String> = entries.iter().map(|e| e.id.clone()).collect();
    let body_struct = MarkerRequestBody {
      action: "markAsUnsaved".to_string(),
      type_field: "entries".to_string(),
      entry_ids: entry_ids,
    };
    let body: Vec<u8> = try!(serde_json::to_vec(&body_struct));
    try!(self.client.post(url, Some(self.auth_header()), body.as_slice()));
    Ok(())
  }

  pub fn subscriptions(&self) -> Result<Vec<SubscriptionDetail>> {
    let response = try!(self.client.get("http://cloud.feedly.com/v3/subscriptions",
                                        Some(self.auth_header())));
    let detail: Vec<SubscriptionDetail> = try!(serde_json::from_reader(response));
    Ok(detail)
  }

  pub fn detail_for_entries(&self, ids: Vec<String>) -> Result<Vec<EntryDetail>> {
    let url = "http://cloud.feedly.com/v3/entries/.mget";
    let quoted: Vec<String> = ids.into_iter().map(|i| "\"".to_string() + &i + "\"").collect();
    let body = "[".to_string() + &quoted.join(",") + "]";

    let response = try!(self.client.post(url, None, body.as_bytes()));
    let detail: Vec<EntryDetail> = try!(serde_json::from_reader(response));

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

  fn get(&self, url: &str, authHeader: Option<header::Authorization<String>>) -> Result<Self::R>;
  fn post(&self,
          url: &str,
          authHeader: Option<header::Authorization<String>>,
          body: &[u8])
      -> Result<Self::R>;
}

pub struct HyperClientWrapper {}

impl HttpMockableClient for HyperClientWrapper {
  type R = hyper::client::Response;

  fn get(&self, url: &str, auth_header: Option<header::Authorization<String>>) -> Result<Self::R> {
    let client = Client::new();
    let mut builder = client.get(url);
    match auth_header {
      Some(h) => builder = builder.header(h),
      None => {}
    }
    builder.send().map_err(|e| FdownError::from(e))
  }

  fn post(&self,
          url: &str,
          auth_header: Option<header::Authorization<String>>,
          body: &[u8])
      -> Result<Self::R> {
    let client = Client::new();
    let mut builder = client.post(url).body(body);
    match auth_header {
      Some(h) => builder = builder.header(h),
      None => {}
    }
    builder.send().map_err(|e| FdownError::from(e))
  }
}

#[cfg(test)]
mod tests {
  use generated::*;
  use hyper::header;
  use result::{FdownError, Result};
  use std::cell::{Cell, RefCell};
  use std::convert::From;
  use std::io::Cursor;
  use super::*;

  const TEST_USERID: &'static str = "test_userid";
  const TEST_TOKEN: &'static str = "test_token";

  type MockFeedly<'a> = FeedlyInternal<NullClient<'a>>;

  struct NullClient<'a> {
    responses: Vec<&'a str>,
    url: RefCell<Option<String>>,
    has_auth: Cell<bool>,
    body: RefCell<Option<Vec<u8>>>,
  }
  impl<'a> NullClient<'a> {
    fn check_url(&self, url: &str) {
      assert_eq!(url, self.url.borrow().as_ref().unwrap());
    }
    fn check_has_auth(&self, val: bool) {
      assert_eq!(val, self.has_auth.get());
    }
    fn check_has_no_body(&self) {
      assert_eq!(true, self.body.borrow().as_ref().is_none());
    }
    fn check_body(&self, body_str: &str) {
      let b = self.body.borrow();
      let vec_bytes = b.as_ref().unwrap().as_slice();
      let sent_body = String::from_utf8_lossy(vec_bytes);
      assert_eq!(body_str, sent_body);
    }
  }
  impl<'a> HttpMockableClient for NullClient<'a> {
    type R = Cursor<Vec<u8>>;

    // TODO: combine get/set code into single thing.
    fn get(&self,
           url: &str,
           auth_header: Option<header::Authorization<String>>)
        -> Result<Self::R> {
      if self.responses.len() < 1 {
        return Err(FdownError::TestError);
      }
      // Save away our arguments for verification.
      *self.url.borrow_mut() = Some(url.to_string());
      self.has_auth.set(auth_header.is_some());

      let bytes = self.responses.get(0).unwrap().as_bytes();
      let vec: Vec<u8> = From::from(bytes);
      let cursor = Cursor::new(vec);

      Ok(cursor)
    }
    fn post(&self,
            url: &str,
            auth_header: Option<header::Authorization<String>>,
            body: &[u8])
        -> Result<Self::R> {
      if self.responses.len() < 1 {
        return Err(FdownError::TestError);
      }

      // Save away our arguments for verification.
      *self.url.borrow_mut() = Some(url.to_string());
      self.has_auth.set(auth_header.is_some());
      *self.body.borrow_mut() = Some(Vec::from(body));

      let bytes = self.responses.get(0).unwrap().as_bytes();
      let vec: Vec<u8> = From::from(bytes);
      let cursor = Cursor::new(vec);

      Ok(cursor)
    }
  }

  fn null_client<'a>(responses: Vec<&'a str>) -> MockFeedly<'a> {
    Feedly::new_with_client(TEST_USERID,
                            TEST_TOKEN,
                            NullClient {
                              responses: responses,
                              url: RefCell::new(None),
                              has_auth: Cell::new(false),
                              body: RefCell::new(None),
                            })
  }

  #[test]
  fn saved_feed() {
    assert_eq!(null_client(vec![]).saved_feed(),
               "user/test_userid/tag/global.saved");
  }

  #[test]
  fn auth_header() {
    let header::Authorization(s) = null_client(vec![]).auth_header();
    assert_eq!("OAuth test_token", s);
  }

  #[test]
  fn saved_entry_ids() {
    let resp = "{ \"ids\": [ \"id1\", \"id2\", \"id3\" ],
                  \"continuation\": \"continuation\" }";
    let feedly = null_client(vec![resp]);
    let ids = feedly.saved_entry_ids(5).unwrap();
    feedly.client.check_has_auth(true);
    feedly.client
      .check_url("http://cloud.feedly.com/v3/streams/ids?streamId=user/test_userid/tag/global.\
                  saved&count=5");
    feedly.client.check_has_no_body();
    assert_eq!(vec!["id1", "id2", "id3"], ids);
  }

  #[test]
  fn saved_entry_ids_bad_http() {
    let feedly = null_client(vec![]);
    feedly.saved_entry_ids(5).unwrap_err();
  }

  #[test]
  fn saved_entry_ids_bad_json() {
    let resp = "{ ids: [ \"id1\", \"id2\", \"id3\" ], \"continuation\": \"continuation\" }";
    let feedly = null_client(vec![resp]);
    feedly.saved_entry_ids(5).unwrap_err();
  }

  #[test]
  fn entry_detail_empty() {
    let feedly = null_client(vec!["[]"]);
    let entries = feedly.detail_for_entries(vec![]).unwrap();
    feedly.client.check_has_auth(false);
    feedly.client.check_url("http://cloud.feedly.com/v3/entries/.mget");
    feedly.client.check_body("[]");
    assert_eq!(0, entries.len());
  }

  #[test]
  fn entry_detail() {
    let feedly = null_client(vec!["[{ \"id\": \"id1\" }, { \"id\": \"id2\" }, { \"id\": \"id3\" \
                                   }]"]);
    let entries =
      feedly.detail_for_entries(vec!["id1".to_string(), "id2".to_string(), "id3".to_string()])
        .unwrap();
    feedly.client.check_url("http://cloud.feedly.com/v3/entries/.mget");
    feedly.client.check_has_auth(false);
    feedly.client.check_body("[\"id1\",\"id2\",\"id3\"]");

    assert_eq!(3, entries.len());
    let foo: Vec<EntryDetail> = vec![EntryDetail {
                                       id: "id1".to_string(),
                                       fingerprint: None,
                                       origin: None,
                                       visual: None,
                                     },
                                     EntryDetail {
                                       id: "id2".to_string(),
                                       fingerprint: None,
                                       origin: None,
                                       visual: None,
                                     },
                                     EntryDetail {
                                       id: "id3".to_string(),
                                       fingerprint: None,
                                       origin: None,
                                       visual: None,
                                     }];
    assert_eq!(foo, entries);
  }

  #[test]
  fn entry_detail_bad_http() {
    let feedly = null_client(vec![]);
    feedly.detail_for_entries(vec!["id1".to_string()]).unwrap_err();
  }

  #[test]
  fn entry_detail_bad_json() {
    let resp = "this is a bad json string";
    let feedly = null_client(vec![resp]);
    feedly.detail_for_entries(vec!["id1".to_string()]).unwrap_err();
  }
}
