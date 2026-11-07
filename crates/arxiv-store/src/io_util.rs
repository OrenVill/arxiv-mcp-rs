use std::path::{Path, PathBuf};

use tokio::io::AsyncWriteExt;

use crate::StoreError;

pub(crate) async fn reject_symlink(path: &Path) -> Result<(), StoreError> {
    match tokio::fs::symlink_metadata(path).await {
        Ok(meta) if meta.file_type().is_symlink() => {
            Err(StoreError::SymlinkRejected(path.display().to_string()))
        }
        Ok(_) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(StoreError::from(e)),
    }
}

pub(crate) async fn atomic_write(path: &Path, data: &[u8]) -> Result<(), StoreError> {
    reject_symlink(path).await?;

    let parent = path
        .parent()
        .ok_or_else(|| StoreError::Io(std::io::Error::other("path has no parent")))?;
    reject_symlink(parent).await?;

    let file_name = path
        .file_name()
        .ok_or_else(|| StoreError::Io(std::io::Error::other("path has no file name")))?;
    let tmp_path: PathBuf = parent.join(format!(".{}.tmp", file_name.to_string_lossy()));

    let write_result = async {
        let mut file = tokio::fs::File::create(&tmp_path).await?;
        file.write_all(data).await?;
        file.sync_all().await?;
        tokio::fs::rename(&tmp_path, path).await?;
        Ok::<(), std::io::Error>(())
    }
    .await;

    if write_result.is_err() {
        let _ = tokio::fs::remove_file(&tmp_path).await;
    }
    write_result.map_err(StoreError::from)
}
