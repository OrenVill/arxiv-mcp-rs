use std::path::{Path, PathBuf};

use arxiv_core::ArxivId;
use async_trait::async_trait;
use tokio::fs;

use crate::{StoreBackend, StoreError};

#[derive(Debug, Clone)]
pub struct LocalFsStore {
    root: PathBuf,
}

impl LocalFsStore {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn paper_dir(&self, id: &ArxivId) -> PathBuf {
        self.root.join("papers").join(id.filesystem_stem())
    }
}

#[async_trait]
impl StoreBackend for LocalFsStore {
    async fn store_version(&self) -> u32 {
        1
    }

    async fn ensure_root(&self) -> Result<(), StoreError> {
        fs::create_dir_all(self.root.join("papers")).await?;
        let version_file = self.root.join("store-version");
        if !fs::try_exists(&version_file).await? {
            fs::write(&version_file, b"1").await?;
        }
        Ok(())
    }

    async fn has_paper(&self, id: &ArxivId) -> Result<bool, StoreError> {
        let content = self.paper_dir(id).join("content.md");
        Ok(fs::try_exists(content).await?)
    }
}
