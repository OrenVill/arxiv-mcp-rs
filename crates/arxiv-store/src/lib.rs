mod local;
mod manifest;

pub use local::LocalFsStore;
pub use manifest::ManifestV1;

use arxiv_core::ArxivId;
use async_trait::async_trait;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("not found: {0}")]
    NotFound(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("quota exceeded")]
    QuotaExceeded,
}

#[async_trait]
pub trait StoreBackend: Send + Sync {
    async fn store_version(&self) -> u32;
    async fn ensure_root(&self) -> Result<(), StoreError>;
    async fn has_paper(&self, id: &ArxivId) -> Result<bool, StoreError>;
}
