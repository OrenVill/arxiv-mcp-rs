use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use arxiv_core::ArxivId;
use async_trait::async_trait;
use sha2::{Digest, Sha256};
use tokio::fs;
use tokio::sync::{Mutex, RwLock};

use crate::io_util::{atomic_write, reject_symlink};
use crate::manifest::ManifestV1;
use crate::{ListPaperItem, ListResponse, StoreBackend, StoreError, StoreRecord, StoredPaper};

#[derive(Debug, Clone)]
pub struct LocalFsStore {
    root: PathBuf,
    quota_mb: Option<u64>,
    max_content_bytes: usize,
    paper_locks: Arc<Mutex<HashMap<String, Arc<RwLock<()>>>>>,
}

impl LocalFsStore {
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            quota_mb: None,
            max_content_bytes: 32 * 1024 * 1024,
            paper_locks: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn with_quota_mb(mut self, quota_mb: Option<u64>) -> Self {
        self.quota_mb = quota_mb;
        self
    }

    pub fn with_max_content_bytes(mut self, max_content_bytes: usize) -> Self {
        self.max_content_bytes = max_content_bytes;
        self
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn paper_dir(&self, id: &ArxivId) -> PathBuf {
        self.root.join("papers").join(id.filesystem_stem())
    }

    async fn paper_lock(&self, bare_key: &str) -> Arc<RwLock<()>> {
        let mut map = self.paper_locks.lock().await;
        map.entry(bare_key.to_string())
            .or_insert_with(|| Arc::new(RwLock::new(())))
            .clone()
    }

    fn content_sha256(content: &[u8]) -> String {
        let digest = Sha256::digest(content);
        digest.iter().map(|b| format!("{b:02x}")).collect()
    }

    async fn dir_file_bytes(dir: &Path) -> Result<u64, StoreError> {
        let mut total = 0u64;
        let mut rd = fs::read_dir(dir).await?;
        while let Some(entry) = rd.next_entry().await? {
            let ft = entry.file_type().await?;
            if ft.is_symlink() {
                return Err(StoreError::SymlinkRejected(
                    entry.path().display().to_string(),
                ));
            }
            if ft.is_file() {
                total += entry.metadata().await?.len();
            }
        }
        Ok(total)
    }

    async fn store_bytes_used(&self) -> Result<u64, StoreError> {
        let papers = self.root.join("papers");
        if !fs::try_exists(&papers).await? {
            return Ok(0);
        }
        reject_symlink(&papers).await?;

        let mut total = 0u64;
        let mut rd = fs::read_dir(&papers).await?;
        while let Some(entry) = rd.next_entry().await? {
            let ft = entry.file_type().await?;
            if ft.is_symlink() {
                return Err(StoreError::SymlinkRejected(
                    entry.path().display().to_string(),
                ));
            }
            if ft.is_dir() {
                total += Self::dir_file_bytes(&entry.path()).await?;
            }
        }
        Ok(total)
    }

    async fn read_manifest_and_content(
        &self,
        dir: &Path,
    ) -> Result<(ManifestV1, String), StoreError> {
        reject_symlink(dir).await?;
        let manifest_path = dir.join("manifest.json");
        let content_path = dir.join("content.md");

        for path in [&manifest_path, &content_path] {
            if !fs::try_exists(path).await? {
                return Err(StoreError::NotFound(path.display().to_string()));
            }
            reject_symlink(path).await?;
        }

        let manifest_bytes = fs::read(&manifest_path).await?;
        let manifest: ManifestV1 = serde_json::from_slice(&manifest_bytes)
            .map_err(|e| StoreError::InvalidManifest(e.to_string()))?;
        let content = String::from_utf8(fs::read(&content_path).await?)
            .map_err(|e| StoreError::InvalidManifest(e.to_string()))?;

        let expected = Self::content_sha256(content.as_bytes());
        if manifest.content_sha256 != expected {
            return Err(StoreError::InvalidManifest(
                "content_sha256 does not match content.md".into(),
            ));
        }

        Ok((manifest, content))
    }
}

#[async_trait]
impl StoreBackend for LocalFsStore {
    async fn store_version(&self) -> u32 {
        1
    }

    async fn ensure_root(&self) -> Result<(), StoreError> {
        reject_symlink(&self.root).await?;
        fs::create_dir_all(self.root.join("papers")).await?;
        let version_file = self.root.join("store-version");
        if !fs::try_exists(&version_file).await? {
            atomic_write(&version_file, b"1").await?;
        }
        Ok(())
    }

    async fn has_paper(&self, id: &ArxivId) -> Result<bool, StoreError> {
        let content = self.paper_dir(id).join("content.md");
        if !fs::try_exists(&content).await? {
            return Ok(false);
        }
        reject_symlink(&content).await?;
        Ok(true)
    }

    async fn read_paper(&self, bare_id: &ArxivId) -> Result<StoredPaper, StoreError> {
        let bare_key = bare_id.bare_id();
        let lock = self.paper_lock(&bare_key).await;
        let _guard = lock.read().await;

        let dir = self.paper_dir(bare_id);
        if !fs::try_exists(&dir).await? {
            return Err(StoreError::NotFound(bare_key));
        }

        let (manifest, content) = self.read_manifest_and_content(&dir).await?;
        Ok(StoredPaper { manifest, content })
    }

    async fn write_paper(&self, record: &StoreRecord) -> Result<(), StoreError> {
        let bare = ArxivId::parse(&record.manifest.paper_id).map_err(|e| {
            StoreError::InvalidPaperId(format!("{}: {e}", record.manifest.paper_id))
        })?;
        let bare_key = bare.bare_id();
        let lock = self.paper_lock(&bare_key).await;
        let _guard = lock.write().await;

        let content_bytes = record.content.as_bytes();
        if content_bytes.len() > self.max_content_bytes {
            return Err(StoreError::ContentTooLarge);
        }

        let dir = self.paper_dir(&bare);
        let exists = fs::try_exists(&dir).await?;
        if exists {
            reject_symlink(&dir).await?;
        } else {
            fs::create_dir_all(&dir).await?;
            reject_symlink(&dir).await?;
        }

        let old_size = if exists {
            Self::dir_file_bytes(&dir).await?
        } else {
            0
        };

        let mut manifest = record.manifest.clone();
        if manifest.schema_version != 1 {
            return Err(StoreError::InvalidManifest(format!(
                "unsupported schema_version {}",
                manifest.schema_version
            )));
        }
        manifest.paper_id = bare.bare_id();
        manifest.content_sha256 = Self::content_sha256(content_bytes);
        manifest.untrusted = true;

        let manifest_json = serde_json::to_vec_pretty(&manifest)
            .map_err(|e| StoreError::InvalidManifest(e.to_string()))?;
        let new_size = u64::try_from(content_bytes.len()).unwrap_or(u64::MAX)
            + u64::try_from(manifest_json.len()).unwrap_or(u64::MAX);

        if let Some(mb) = self.quota_mb {
            let limit = mb.saturating_mul(1024 * 1024);
            let used = self.store_bytes_used().await?;
            if used.saturating_sub(old_size).saturating_add(new_size) > limit {
                return Err(StoreError::QuotaExceeded);
            }
        }

        atomic_write(&dir.join("content.md"), content_bytes).await?;
        atomic_write(&dir.join("manifest.json"), &manifest_json).await?;

        Ok(())
    }

    async fn list_papers(&self, compact: bool) -> Result<ListResponse, StoreError> {
        let papers_dir = self.root.join("papers");
        if !fs::try_exists(&papers_dir).await? {
            return Ok(ListResponse { papers: Vec::new() });
        }
        reject_symlink(&papers_dir).await?;

        let mut items = Vec::new();
        let mut rd = fs::read_dir(&papers_dir).await?;
        while let Some(entry) = rd.next_entry().await? {
            let ft = entry.file_type().await?;
            if ft.is_symlink() {
                return Err(StoreError::SymlinkRejected(
                    entry.path().display().to_string(),
                ));
            }
            if !ft.is_dir() {
                continue;
            }

            let dir = entry.path();
            let (manifest, _) = match self.read_manifest_and_content(&dir).await {
                Ok(v) => v,
                Err(StoreError::NotFound(_)) | Err(StoreError::InvalidManifest(_)) => continue,
                Err(e) => return Err(e),
            };

            if compact {
                items.push(ListPaperItem {
                    id: manifest.paper_id.clone(),
                    title: None,
                    authors: None,
                    published: None,
                    arxiv_version: None,
                    versioned_id: None,
                });
            } else {
                items.push(ListPaperItem {
                    id: manifest.paper_id.clone(),
                    title: manifest.title.clone(),
                    authors: Some(manifest.authors.clone()),
                    published: manifest.published.clone(),
                    arxiv_version: manifest.arxiv_version.clone(),
                    versioned_id: manifest.versioned_id.clone(),
                });
            }
        }

        items.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(ListResponse { papers: items })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::FetchedVia;

    fn sample_manifest(paper_id: &str, content: &str) -> ManifestV1 {
        let mut m = ManifestV1::new(paper_id);
        m.converter_version = 1;
        m.fetched_via = FetchedVia::Html;
        m.content_sha256 = LocalFsStore::content_sha256(content.as_bytes());
        m.fetched_at = "2020-01-01T00:00:00Z".into();
        m.title = Some("Attention".into());
        m.authors = vec!["A".into(), "B".into()];
        m.published = Some("2017-06-12".into());
        m.arxiv_version = Some("v7".into());
        m.versioned_id = Some("1706.03762v7".into());
        m
    }

    async fn temp_store() -> (tempfile::TempDir, LocalFsStore) {
        let dir = tempfile::tempdir().expect("tempdir");
        let store = LocalFsStore::new(dir.path().to_path_buf());
        store.ensure_root().await.expect("ensure_root");
        (dir, store)
    }

    #[tokio::test]
    async fn write_read_roundtrip() {
        let (_dir, store) = temp_store().await;
        let content = "# Hello\n\nWorld.";
        let manifest = sample_manifest("1706.03762", content);
        store
            .write_paper(&StoreRecord {
                manifest,
                content: content.into(),
            })
            .await
            .expect("write");

        let id = ArxivId::parse("1706.03762").unwrap();
        let paper = store.read_paper(&id).await.expect("read");
        assert_eq!(paper.content, content);
        assert_eq!(paper.manifest.paper_id, "1706.03762");
        assert!(paper.manifest.untrusted);
        assert!(store.has_paper(&id).await.unwrap());
    }

    #[tokio::test]
    async fn list_compact_and_full() {
        let (_dir, store) = temp_store().await;
        let content = "body";
        store
            .write_paper(&StoreRecord {
                manifest: sample_manifest("1706.03762", content),
                content: content.into(),
            })
            .await
            .unwrap();
        store
            .write_paper(&StoreRecord {
                manifest: sample_manifest("2312.00001", content),
                content: content.into(),
            })
            .await
            .unwrap();

        let compact = store.list_papers(true).await.unwrap();
        assert_eq!(compact.papers.len(), 2);
        assert!(compact.papers.iter().all(|p| p.title.is_none()));

        let full = store.list_papers(false).await.unwrap();
        assert_eq!(full.papers.len(), 2);
        assert_eq!(full.papers[0].title.as_deref(), Some("Attention"));
        assert_eq!(
            full.papers[0].authors.as_deref(),
            Some([String::from("A"), String::from("B")].as_slice())
        );
    }

    #[tokio::test]
    async fn quota_rejects_new_write() {
        let dir = tempfile::tempdir().unwrap();
        let store = LocalFsStore::new(dir.path().to_path_buf()).with_quota_mb(Some(1));
        store.ensure_root().await.unwrap();

        let big = "x".repeat(800_000);
        let manifest = sample_manifest("1706.03762", &big);
        store
            .write_paper(&StoreRecord {
                manifest,
                content: big,
            })
            .await
            .unwrap();

        let extra = "y".repeat(300_000);
        let err = store
            .write_paper(&StoreRecord {
                manifest: sample_manifest("2312.00001", &extra),
                content: extra,
            })
            .await
            .unwrap_err();
        assert!(matches!(err, StoreError::QuotaExceeded));
    }

    #[tokio::test]
    async fn rejects_symlink_paper_dir() {
        let dir = tempfile::tempdir().unwrap();
        let store = LocalFsStore::new(dir.path().to_path_buf());
        store.ensure_root().await.unwrap();

        let real = dir.path().join("papers/real_dir");
        fs::create_dir_all(&real).await.unwrap();
        let link = dir.path().join("papers/1706.03762");
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(&real, &link).unwrap();
        }
        #[cfg(not(unix))]
        {
            return;
        }

        let err = store
            .write_paper(&StoreRecord {
                manifest: sample_manifest("1706.03762", "x"),
                content: "x".into(),
            })
            .await
            .unwrap_err();
        assert!(matches!(err, StoreError::SymlinkRejected(_)));
    }

    #[tokio::test]
    async fn atomic_write_leaves_consistent_manifest() {
        let (_dir, store) = temp_store().await;
        let content = "stable";
        store
            .write_paper(&StoreRecord {
                manifest: sample_manifest("1706.03762", content),
                content: content.into(),
            })
            .await
            .unwrap();

        let path = store.paper_dir(&ArxivId::parse("1706.03762").unwrap());
        let raw = fs::read(path.join("manifest.json")).await.unwrap();
        let manifest: ManifestV1 = serde_json::from_slice(&raw).unwrap();
        assert_eq!(manifest.schema_version, 1);
        assert_eq!(manifest.fetched_via, FetchedVia::Html);
        assert!(!manifest.content_sha256.is_empty());
    }

    #[tokio::test]
    async fn read_missing_paper() {
        let (_dir, store) = temp_store().await;
        let id = ArxivId::parse("9999.00001").unwrap();
        let err = store.read_paper(&id).await.unwrap_err();
        assert!(matches!(err, StoreError::NotFound(_)));
    }

    #[tokio::test]
    async fn legacy_id_roundtrip() {
        let (_dir, store) = temp_store().await;
        let content = "text";
        let manifest = sample_manifest("hep-th/9901001", content);
        store
            .write_paper(&StoreRecord {
                manifest,
                content: content.into(),
            })
            .await
            .unwrap();

        let id = ArxivId::parse("hep-th/9901001").unwrap();
        let listed = store.list_papers(false).await.unwrap();
        assert!(listed.papers.iter().any(|p| p.id == id.bare_id()));
        let paper = store.read_paper(&id).await.unwrap();
        assert_eq!(paper.manifest.paper_id, id.bare_id());
    }
}
