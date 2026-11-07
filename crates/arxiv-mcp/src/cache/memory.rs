use std::collections::HashMap;
use std::time::{Duration, Instant};

use arxiv_core::{CachedAtom, CacheKey, SearchCache};
use async_trait::async_trait;
use tokio::sync::RwLock;

const DEFAULT_SEARCH_TTL: Duration = Duration::from_secs(300);
const DEFAULT_ABSTRACT_TTL: Duration = Duration::from_secs(600);

struct CacheEntry {
    value: CachedAtom,
    expires_at: Instant,
}

#[derive(Default)]
pub struct MemorySearchCache {
    inner: RwLock<HashMap<String, CacheEntry>>,
}

impl MemorySearchCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn default_search_ttl() -> Duration {
        DEFAULT_SEARCH_TTL
    }

    pub fn default_abstract_ttl() -> Duration {
        DEFAULT_ABSTRACT_TTL
    }
}

#[async_trait]
impl SearchCache for MemorySearchCache {
    async fn get(&self, key: &CacheKey) -> Option<CachedAtom> {
        let guard = self.inner.read().await;
        let entry = guard.get(&key.0)?;
        if entry.expires_at <= Instant::now() {
            return None;
        }
        Some(entry.value.clone())
    }

    async fn set(&self, key: CacheKey, value: CachedAtom, ttl: Duration) {
        let mut guard = self.inner.write().await;
        guard.insert(
            key.0,
            CacheEntry {
                value,
                expires_at: Instant::now() + ttl,
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arxiv_core::parse_atom_feed;

    #[tokio::test]
    async fn stores_and_returns_before_ttl() {
        let cache = MemorySearchCache::new();
        let key = CacheKey("k".into());
        let feed = parse_atom_feed(include_str!(
            "../../../arxiv-core/tests/fixtures/search_atom.xml"
        ))
        .unwrap();
        let cached = CachedAtom {
            xml: "x".into(),
            feed,
        };
        cache
            .set(key.clone(), cached.clone(), Duration::from_secs(60))
            .await;
        let got = cache.get(&key).await.expect("hit");
        assert_eq!(got.feed.total_results, cached.feed.total_results);
    }
}
