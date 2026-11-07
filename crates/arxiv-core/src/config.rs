use std::path::PathBuf;

/// Process configuration (immutable, shared via `Arc`).
#[derive(Debug, Clone)]
pub struct Config {
    pub storage_path: PathBuf,
    pub default_max_chars: usize,
    pub contact_email: Option<String>,
    pub quota_mb: Option<u64>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            storage_path: default_storage_path(),
            default_max_chars: 12_000,
            contact_email: None,
            quota_mb: None,
        }
    }
}

pub fn default_storage_path() -> PathBuf {
    dirs_home().join(".arxiv-mcp-rs").join("papers")
}

pub(crate) fn dirs_home() -> PathBuf {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}
