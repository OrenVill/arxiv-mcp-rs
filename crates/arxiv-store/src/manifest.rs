use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestV1 {
    pub schema_version: u32,
    pub paper_id: String,
    pub untrusted: bool,
}

impl ManifestV1 {
    pub fn new(paper_id: impl Into<String>) -> Self {
        Self {
            schema_version: 1,
            paper_id: paper_id.into(),
            untrusted: true,
        }
    }
}
