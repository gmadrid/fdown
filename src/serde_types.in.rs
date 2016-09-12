#[derive(Debug,Deserialize)]
pub struct StreamsIdsResponse {
  continuation: String,
  pub ids: Vec<String>
}

#[derive(Debug,Deserialize)]
pub struct EntryDetailVisual {
  pub url: Option<String>,
  #[serde(rename="contentType")]
  pub content_type: String
}

#[derive(Debug,Deserialize)]
pub struct EntryDetail {
  pub id: String,
  pub fingerprint: String,
  pub visual: Option<EntryDetailVisual>
}

