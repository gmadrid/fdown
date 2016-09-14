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
pub struct EntryDetailOrigin {
  pub streamId: String,
  pub title: Option<String>
}

#[derive(Debug,Deserialize)]
pub struct EntryDetail {
  pub id: String,
  pub fingerprint: String,
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
