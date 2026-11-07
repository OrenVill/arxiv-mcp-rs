//! Upstream (blazickjp) on-disk layout — detect only; never read automatically (D2).

use std::path::{Path, PathBuf};

/// Default papers directory used by `arxiv-mcp-server` (Python).
pub fn legacy_python_storage_path() -> PathBuf {
    super::config::dirs_home().join(".arxiv-mcp-server").join("papers")
}

/// True when *root* looks like the upstream flat cache (`*.md` at top level).
pub fn legacy_flat_store_detected(root: &Path) -> bool {
    let Ok(entries) = std::fs::read_dir(root) else {
        return false;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() && path.extension().is_some_and(|e| e == "md") {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn detects_flat_md() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("2404.19756.md"), b"# x").unwrap();
        assert!(legacy_flat_store_detected(dir.path()));
    }

    #[test]
    fn empty_dir_not_legacy() {
        let dir = tempdir().unwrap();
        assert!(!legacy_flat_store_detected(dir.path()));
    }
}
