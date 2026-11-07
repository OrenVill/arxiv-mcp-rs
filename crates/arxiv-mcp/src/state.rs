use std::sync::Arc;

use arxiv_core::{ArxivGateway, SearchCache};
use arxiv_fetch::{AllowlistedClient, ProcessArxivGateway};
use arxiv_store::LocalFsStore;

use crate::cache::MemorySearchCache;

/// Immutable, shareable process state (stateless handlers; no per-client fields).
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<arxiv_core::Config>,
    pub store: Arc<LocalFsStore>,
    pub gateway: Arc<dyn ArxivGateway>,
    pub cache: Arc<dyn SearchCache>,
}

impl AppState {
    pub fn new(config: arxiv_core::Config) -> anyhow::Result<Self> {
        let store = LocalFsStore::new(config.storage_path.clone());
        let allowlist = AllowlistedClient::default();
        let gateway = ProcessArxivGateway::new(allowlist)?;
        Ok(Self {
            config: Arc::new(config),
            store: Arc::new(store),
            gateway: Arc::new(gateway),
            cache: Arc::new(MemorySearchCache::new()),
        })
    }
}
