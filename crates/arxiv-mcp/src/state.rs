use std::sync::Arc;

use arxiv_core::Config;
use arxiv_store::LocalFsStore;

/// Immutable, shareable process state (stateless handlers; no per-client fields).
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub store: Arc<LocalFsStore>,
}

impl AppState {
    pub fn new(config: Config) -> Self {
        let store = LocalFsStore::new(config.storage_path.clone());
        Self {
            config: Arc::new(config),
            store: Arc::new(store),
        }
    }
}
