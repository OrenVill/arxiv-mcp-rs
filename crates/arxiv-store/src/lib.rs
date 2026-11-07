mod io_util;
mod local;
mod manifest;

pub use local::LocalFsStore;
pub use manifest::{FetchedVia, ManifestV1};

use arxiv_core::ArxivId;
use async_trait::async_trait;
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct StoredPaper {
    pub manifest: ManifestV1,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct StoreRecord {
    pub manifest: ManifestV1,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct ListPaperItem {
    pub id: String,
    pub title: Option<String>,
    pub authors: Option<Vec<String>>,
    pub published: Option<String>,
    pub arxiv_version: Option<String>,
    pub versioned_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ListResponse {
    pub papers: Vec<ListPaperItem>,
}

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("not found: {0}")]
    NotFound(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("quota exceeded")]
    QuotaExceeded,
    #[error("symlink not allowed: {0}")]
    SymlinkRejected(String),
    #[error("invalid manifest: {0}")]
    InvalidManifest(String),
    #[error("invalid paper id in manifest: {0}")]
    InvalidPaperId(String),
    #[error("content exceeds per-paper byte cap")]
    ContentTooLarge,
}

#[async_trait]
pub trait StoreBackend: Send + Sync {
    async fn store_version(&self) -> u32;
    async fn ensure_root(&self) -> Result<(), StoreError>;
    async fn has_paper(&self, id: &ArxivId) -> Result<bool, StoreError>;
    async fn read_paper(&self, bare_id: &ArxivId) -> Result<StoredPaper, StoreError>;
    async fn write_paper(&self, record: &StoreRecord) -> Result<(), StoreError>;
    async fn list_papers(&self, compact: bool) -> Result<ListResponse, StoreError>;
}
