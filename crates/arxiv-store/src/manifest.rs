use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FetchedVia {
    Html,
    Pdf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestV1 {
    pub schema_version: u32,
    pub paper_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arxiv_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub versioned_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub authors: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub published: Option<String>,
    pub converter_version: u32,
    pub fetched_via: FetchedVia,
    pub content_sha256: String,
    pub fetched_at: String,
    pub untrusted: bool,
}

impl ManifestV1 {
    pub fn new(paper_id: impl Into<String>) -> Self {
        Self {
            schema_version: 1,
            paper_id: paper_id.into(),
            arxiv_version: None,
            versioned_id: None,
            title: None,
            authors: Vec::new(),
            published: None,
            converter_version: 1,
            fetched_via: FetchedVia::Html,
            content_sha256: String::new(),
            fetched_at: String::new(),
            untrusted: true,
        }
    }
}
