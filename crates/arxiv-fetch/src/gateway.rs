use arxiv_core::{
    parse_atom_feed, ArxivError, ArxivGateway, ArxivRateLimiter, SearchParams, SearchQueryError,
    build_id_list_url, build_search_url,
};
use async_trait::async_trait;
use reqwest::StatusCode;

use crate::allowlist::AllowlistedClient;
use crate::client::{AllowlistedHttp, FetchError};

const DEFAULT_RETRY_AFTER: u64 = 60;

pub struct ProcessArxivGateway {
    http: AllowlistedHttp,
    gate: ArxivRateLimiter,
}

impl ProcessArxivGateway {
    pub fn new(allowlist: AllowlistedClient) -> anyhow::Result<Self> {
        Ok(Self {
            http: AllowlistedHttp::new(&allowlist)?,
            gate: ArxivRateLimiter::default_gate(),
        })
    }

    async fn fetch_atom(&self, url: String) -> Result<String, ArxivError> {
        self.gate
            .run(async {
                match self.http.get_text(&url).await {
                    Ok((_status, text)) => Ok(text),
                    Err(FetchError::Http(status)) if status == StatusCode::TOO_MANY_REQUESTS => {
                        Err(ArxivError::RateLimited {
                            retry_after_seconds: DEFAULT_RETRY_AFTER,
                            http_status: status.as_u16(),
                            message: "arXiv is rate limiting this IP (HTTP 429). Please wait before retrying.".into(),
                        })
                    }
                    Err(FetchError::Http(status))
                        if status == StatusCode::SERVICE_UNAVAILABLE =>
                    {
                        Err(ArxivError::RateLimited {
                            retry_after_seconds: DEFAULT_RETRY_AFTER,
                            http_status: status.as_u16(),
                            message: "arXiv service unavailable (HTTP 503). Please wait before retrying.".into(),
                        })
                    }
                    Err(FetchError::Http(status)) => Err(ArxivError::Http {
                        status: status.as_u16(),
                    }),
                    Err(FetchError::BlockedRedirect) => Err(ArxivError::BlockedRedirect),
                    Err(FetchError::Network(e)) => Err(ArxivError::Network(e)),
                }
            })
            .await
    }
}

#[async_trait]
impl ArxivGateway for ProcessArxivGateway {
    async fn search(&self, params: &SearchParams) -> Result<arxiv_core::AtomFeed, ArxivError> {
        let url = build_search_url(params).map_err(|e| match e {
            SearchQueryError::NoCriteria => ArxivError::Query("No search criteria provided".into()),
            other => ArxivError::Query(other.to_string()),
        })?;
        let xml = self.fetch_atom(url).await?;
        parse_atom_feed(&xml).map_err(ArxivError::from)
    }

    async fn metadata_by_id(&self, paper_id: &str) -> Result<arxiv_core::AtomFeed, ArxivError> {
        let url = build_id_list_url(paper_id);
        let xml = self.fetch_atom(url).await?;
        parse_atom_feed(&xml).map_err(ArxivError::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arxiv_core::{parse_atom_feed, AbstractMode, SearchParams, SortBy};

    #[test]
    fn parses_fixture() {
        let xml = include_str!("../../arxiv-core/tests/fixtures/search_atom.xml");
        let feed = parse_atom_feed(xml).unwrap();
        assert_eq!(feed.papers.len(), 1);
    }

    #[test]
    fn builds_search_url() {
        let params = SearchParams {
            query: "test".into(),
            max_results: 5,
            start: 0,
            sort_by: SortBy::Relevance,
            date_from: None,
            date_to: None,
            categories: vec![],
            abstract_mode: AbstractMode::Snippet,
        };
        let url = build_search_url(&params).unwrap();
        assert!(url.contains("export.arxiv.org"));
    }
}
