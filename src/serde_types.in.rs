#[derive(Debug,Deserialize)]
pub struct StreamsIdsResponse {
  continuation: String,
  pub ids: Vec<String>
}

#[derive(Debug,Deserialize,PartialEq)]
pub struct EntryDetailVisual {
  pub url: Option<String>,
  #[serde(rename="contentType")]
  pub content_type: Option<String>
}

#[derive(Debug,Deserialize,PartialEq)]
pub struct EntryDetailOrigin {
  #[serde(rename="streamId")]
  pub stream_id: String,
  pub title: Option<String>
}

#[derive(Debug,Deserialize,PartialEq)]
pub struct EntryDetail {
  pub id: String,
  pub fingerprint: Option<String>,
  pub visual: Option<EntryDetailVisual>,
  pub origin: Option<EntryDetailOrigin>
}

#[derive(Debug,Deserialize)]
pub struct SubscriptionDetailCategory {
  pub id: String,
  pub label: Option<String>
}

#[derive(Debug,Deserialize)]
pub struct SubscriptionDetail {
  pub id: String,
  pub website: Option<String>,
  pub title: Option<String>,
  pub categories: Vec<SubscriptionDetailCategory>
}

#[derive(Debug,Serialize)]
pub struct MarkerRequestBody {
  pub action: String,
  #[serde(rename="type")]
  pub type_field: String,
  #[serde(rename="entryIds")]  
  pub entry_ids: Vec<String>
}