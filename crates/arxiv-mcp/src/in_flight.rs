//! In-flight download deduplication keyed by bare paper id (SPEC §9.4).

use dashmap::mapref::entry::Entry;
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::watch;

type DownloadWatch = watch::Sender<Option<Result<(), String>>>;

#[derive(Clone)]
pub struct InFlightDownloads {
    map: Arc<DashMap<String, DownloadWatch>>,
}

impl InFlightDownloads {
    pub fn new() -> Self {
        Self {
            map: Arc::new(DashMap::new()),
        }
    }

    pub async fn coalesce<F, Fut>(&self, key: &str, work: F) -> Result<(), String>
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: std::future::Future<Output = Result<(), String>> + Send + 'static,
    {
        let mut rx = match self.map.entry(key.to_string()) {
            Entry::Vacant(slot) => {
                let (tx, rx) = watch::channel(None);
                slot.insert(tx.clone());
                let map = self.map.clone();
                let job_key = key.to_string();
                tokio::spawn(async move {
                    let result = work().await;
                    let _ = tx.send(Some(result.clone()));
                    map.remove(&job_key);
                });
                rx
            }
            Entry::Occupied(slot) => slot.get().subscribe(),
        };

        loop {
            if let Some(result) = rx.borrow().clone() {
                return result;
            }
            if rx.changed().await.is_err() {
                return Err("download task ended unexpectedly".into());
            }
        }
    }
}

impl Default for InFlightDownloads {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[tokio::test]
    async fn concurrent_downloads_share_one_job() {
        let gate = InFlightDownloads::new();
        let calls = Arc::new(AtomicUsize::new(0));
        let mut handles = Vec::new();
        for _ in 0..4 {
            let gate = gate.clone();
            let calls = calls.clone();
            handles.push(tokio::spawn(async move {
                gate.coalesce("1706.03762", move || {
                    let calls = calls.clone();
                    async move {
                        calls.fetch_add(1, Ordering::SeqCst);
                        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                        Ok(())
                    }
                })
                .await
            }));
        }
        for h in handles {
            h.await.unwrap().expect("job ok");
        }
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }
}
