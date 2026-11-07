use std::time::Duration;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::atom::{AtomFeed, PaperRecord};
use crate::policy::{sanitize_returned_text, strict_sanitize_from_env};
use crate::search_query::SearchParams;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedAtom {
    pub xml: String,
    pub feed: AtomFeed,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CacheKey(pub String);

impl CacheKey {
    pub fn search(params: &SearchParams) -> Self {
        Self(crate::search_query::cache_key_for_search(params))
    }

    pub fn abstract_id(paper_id: &str) -> Self {
        Self(crate::search_query::cache_key_for_abstract(paper_id))
    }
}

#[async_trait]
pub trait SearchCache: Send + Sync {
    async fn get(&self, key: &CacheKey) -> Option<CachedAtom>;
    async fn set(&self, key: CacheKey, value: CachedAtom, ttl: Duration);
}

pub fn paper_to_json_map(
    paper: &PaperRecord,
    abstract_mode: crate::search_query::AbstractMode,
) -> serde_json::Value {
    use crate::content::{snippet_abstract, ABSTRACT_SNIPPET_CHARS};
    let clean_abstract =
        sanitize_returned_text(&paper.abstract_text, strict_sanitize_from_env()).text;
    let mut obj = serde_json::json!({
        "id": paper.id,
        "versioned_id": paper.versioned_id,
        "title": paper.title,
        "authors": paper.authors,
        "categories": paper.categories,
        "published": paper.published,
        "url": paper.url,
        "resource_uri": paper.resource_uri,
    });
    match abstract_mode {
        crate::search_query::AbstractMode::None => {}
        crate::search_query::AbstractMode::Snippet => {
            obj["abstract"] = serde_json::Value::String(snippet_abstract(
                &clean_abstract,
                ABSTRACT_SNIPPET_CHARS,
            ));
        }
        crate::search_query::AbstractMode::Full => {
            obj["abstract"] = serde_json::Value::String(clean_abstract);
        }
    }
    obj
}
