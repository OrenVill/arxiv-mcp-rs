mod papers;

pub use papers::{
    download_paper, list_papers, read_paper, DownloadPaperInput, ReadPaperInput,
};

use std::sync::Arc;

use arxiv_core::{
    paper_to_json_map, sanitize_returned_text, strict_sanitize_from_env, ArxivError, ArxivGateway,
    ArxivId, CachedAtom, CacheKey, CONTENT_WARNING, SearchCache, SearchParams, SortBy,
    validate_categories, AbstractMode,
};
use serde::Serialize;
use serde_json::json;

use crate::cache::MemorySearchCache;
use crate::AppState;

const CATEGORY_ERROR: &str =
    "Invalid category provided. Please check arXiv category names.";

pub(crate) fn json_pretty(value: &impl Serialize) -> String {
    serde_json::to_string_pretty(value).unwrap_or_else(|_| "{}".into())
}

pub(crate) fn error_json(message: impl Into<String>) -> String {
    json_pretty(&json!({ "status": "error", "message": message.into() }))
}

pub(crate) fn rate_limited_json(
    message: String,
    retry_after_seconds: u64,
    http_status: u16,
) -> String {
    json_pretty(&json!({
        "status": "rate_limited",
        "message": message,
        "retry_after_seconds": retry_after_seconds,
        "http_status": http_status,
    }))
}

pub(crate) async fn search_papers(state: &AppState, input: SearchPapersInput) -> String {
    let abstract_mode = match AbstractMode::parse(input.abstract_mode.as_deref()) {
        Ok(m) => m,
        Err(e) => return error_json(e.to_string()),
    };
    let sort_by = match SortBy::parse(input.sort_by.as_deref()) {
        Ok(s) => s,
        Err(e) => return error_json(e.to_string()),
    };
    let categories = input.categories.unwrap_or_default();
    if !categories.is_empty() && validate_categories(&categories).is_err() {
        return error_json(CATEGORY_ERROR);
    }

    let params = SearchParams {
        query: input.query,
        max_results: SearchParams::max_results_clamped(input.max_results),
        start: input.start.unwrap_or(0),
        sort_by,
        date_from: input.date_from,
        date_to: input.date_to,
        categories,
        abstract_mode,
    };

    search_papers_with_gateway(state.gateway.clone(), state.cache.clone(), params).await
}

pub(crate) async fn get_abstract(state: &AppState, paper_id: String) -> String {
    get_abstract_with_gateway(state.gateway.clone(), state.cache.clone(), paper_id).await
}

pub(crate) struct SearchPapersInput {
    pub query: String,
    pub max_results: Option<u32>,
    pub start: Option<u32>,
    pub abstract_mode: Option<String>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub categories: Option<Vec<String>>,
    pub sort_by: Option<String>,
}

pub(crate) async fn search_papers_with_gateway(
    gateway: Arc<dyn ArxivGateway>,
    cache: Arc<dyn SearchCache>,
    params: SearchParams,
) -> String {
    let cache_key = CacheKey::search(&params);
    if let Some(cached) = cache.get(&cache_key).await {
        return build_search_json(&cached.feed, &params);
    }
    match gateway.search(&params).await {
        Ok(feed) => {
            cache
                .set(
                    cache_key,
                    CachedAtom {
                        xml: String::new(),
                        feed: feed.clone(),
                    },
                    MemorySearchCache::default_search_ttl(),
                )
                .await;
            build_search_json(&feed, &params)
        }
        Err(e) => map_arxiv_error(e),
    }
}

pub(crate) async fn get_abstract_with_gateway(
    gateway: Arc<dyn ArxivGateway>,
    cache: Arc<dyn SearchCache>,
    paper_id: String,
) -> String {
    let paper_id = paper_id.trim();
    if paper_id.is_empty() {
        return error_json("paper_id is required");
    }
    let parsed = match ArxivId::parse(paper_id) {
        Ok(id) => id.bare_id(),
        Err(_) => return error_json("invalid arXiv ID format"),
    };
    let cache_key = CacheKey::abstract_id(&parsed);
    if let Some(cached) = cache.get(&cache_key).await {
        return abstract_success(&parsed, cached.feed);
    }
    match gateway.metadata_by_id(&parsed).await {
        Ok(feed) => {
            if feed.papers.is_empty() {
                return error_json(format!("Paper {parsed} not found on arXiv"));
            }
            let response = abstract_success(&parsed, feed.clone());
            cache
                .set(
                    cache_key,
                    CachedAtom {
                        xml: String::new(),
                        feed,
                    },
                    MemorySearchCache::default_abstract_ttl(),
                )
                .await;
            response
        }
        Err(e) => map_arxiv_error(e),
    }
}

fn abstract_success(paper_id: &str, feed: arxiv_core::AtomFeed) -> String {
    let Some(paper) = feed.papers.first() else {
        return error_json(format!("Paper {paper_id} not found on arXiv"));
    };
    json_pretty(&json!({
        "status": "success",
        "paper_id": paper_id,
        "title": paper.title,
        "authors": paper.authors,
        "content_warning": CONTENT_WARNING,
        "abstract": sanitize_returned_text(&paper.abstract_text, strict_sanitize_from_env()).text,
        "categories": paper.categories,
        "published": paper.published,
        "pdf_url": paper.url,
    }))
}

fn map_arxiv_error(err: ArxivError) -> String {
    match err {
        ArxivError::RateLimited {
            retry_after_seconds,
            http_status,
            message,
        } => rate_limited_json(message, retry_after_seconds, http_status),
        ArxivError::Http { status } => error_json(format!("arXiv API HTTP error (HTTP {status})")),
        other => error_json(other.to_string()),
    }
}

fn build_search_json(feed: &arxiv_core::AtomFeed, params: &SearchParams) -> String {
    let papers: Vec<_> = feed
        .papers
        .iter()
        .map(|p| paper_to_json_map(p, params.abstract_mode))
        .collect();
    let returned = papers.len() as u32;
    let total_results = feed.total_results;
    let mut has_more = None;
    let mut next_start = None;
    if let Some(total) = total_results {
        let more = total > params.start as u64 + returned as u64;
        has_more = Some(more);
        next_start = if more {
            Some(params.start + returned)
        } else {
            None
        };
    }
    let mut body = json!({
        "returned": returned,
        "start": params.start,
        "abstract_mode": params.abstract_mode.as_str(),
        "papers": papers,
    });
    if params.abstract_mode != AbstractMode::None {
        body["content_warning"] = json!(CONTENT_WARNING);
    }
    if let Some(total) = total_results {
        body["total_results"] = json!(total);
    }
    if let Some(has_more) = has_more {
        body["has_more"] = json!(has_more);
    }
    body["next_start"] = json!(next_start);
    json_pretty(&body)
}

#[cfg(test)]
mod tests {
    use super::*;
    use arxiv_core::{parse_atom_feed, AtomFeed};
    use async_trait::async_trait;

    struct FixtureGateway;

    #[async_trait]
    impl ArxivGateway for FixtureGateway {
        async fn search(&self, _params: &SearchParams) -> Result<AtomFeed, ArxivError> {
            parse_atom_feed(include_str!(
                "../../arxiv-core/tests/fixtures/search_atom.xml"
            ))
            .map_err(ArxivError::Parse)
        }

        async fn metadata_by_id(&self, _paper_id: &str) -> Result<AtomFeed, ArxivError> {
            parse_atom_feed(include_str!(
                "../../arxiv-core/tests/fixtures/id_list_atom.xml"
            ))
            .map_err(ArxivError::Parse)
        }

        async fn fetch_html(&self, _paper_id: &str) -> Result<Option<Vec<u8>>, ArxivError> {
            Ok(None)
        }

        async fn fetch_pdf(&self, _paper_id: &str) -> Result<Vec<u8>, ArxivError> {
            Err(ArxivError::Http { status: 404 })
        }
    }

    #[tokio::test]
    async fn search_json_has_papers_and_warning() {
        let gateway: Arc<dyn ArxivGateway> = Arc::new(FixtureGateway);
        let cache: Arc<dyn SearchCache> = Arc::new(MemorySearchCache::new());
        let params = SearchParams {
            query: "attention".into(),
            max_results: 5,
            start: 0,
            sort_by: SortBy::Relevance,
            date_from: None,
            date_to: None,
            categories: vec![],
            abstract_mode: AbstractMode::Snippet,
        };
        let text =
            search_papers_with_gateway(gateway.clone(), cache.clone(), params.clone()).await;
        assert!(text.contains("\"papers\""));
        assert!(text.contains(CONTENT_WARNING));
        let again = search_papers_with_gateway(gateway, cache, params).await;
        assert_eq!(text, again);
    }

    struct NoNetworkGateway;

    #[async_trait]
    impl ArxivGateway for NoNetworkGateway {
        async fn search(&self, _params: &SearchParams) -> Result<AtomFeed, ArxivError> {
            panic!("search must not run for invalid paper_id");
        }

        async fn metadata_by_id(&self, _paper_id: &str) -> Result<AtomFeed, ArxivError> {
            panic!("metadata must not run for invalid paper_id");
        }

        async fn fetch_html(&self, _paper_id: &str) -> Result<Option<Vec<u8>>, ArxivError> {
            panic!("fetch_html must not run for invalid paper_id");
        }

        async fn fetch_pdf(&self, _paper_id: &str) -> Result<Vec<u8>, ArxivError> {
            panic!("fetch_pdf must not run for invalid paper_id");
        }
    }

    #[tokio::test]
    async fn get_abstract_rejects_path_traversal_before_network() {
        let gateway: Arc<dyn ArxivGateway> = Arc::new(NoNetworkGateway);
        let cache: Arc<dyn SearchCache> = Arc::new(MemorySearchCache::new());
        let body =
            get_abstract_with_gateway(gateway, cache, "../../../etc/passwd".into()).await;
        assert!(body.contains("\"status\": \"error\""));
        assert!(body.contains("invalid arXiv ID"));
    }

    #[tokio::test]
    async fn get_abstract_rejects_url_paper_id_before_network() {
        let gateway: Arc<dyn ArxivGateway> = Arc::new(NoNetworkGateway);
        let cache: Arc<dyn SearchCache> = Arc::new(MemorySearchCache::new());
        let body = get_abstract_with_gateway(gateway, cache, "https://evil.com/x".into()).await;
        assert!(body.contains("invalid arXiv ID"));
    }
}
