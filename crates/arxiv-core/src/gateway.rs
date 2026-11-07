use async_trait::async_trait;
use thiserror::Error;

use crate::atom::AtomFeed;
use crate::search_query::SearchParams;

#[derive(Debug, Error)]
pub enum ArxivError {
    #[error("rate limited")]
    RateLimited {
        retry_after_seconds: u64,
        http_status: u16,
        message: String,
    },
    #[error("http error (status {status})")]
    Http { status: u16 },
    #[error("blocked redirect")]
    BlockedRedirect,
    #[error("network: {0}")]
    Network(String),
    #[error("parse: {0}")]
    Parse(#[from] crate::atom::AtomParseError),
    #[error("query: {0}")]
    Query(String),
}

#[async_trait]
pub trait ArxivGateway: Send + Sync {
    async fn search(&self, params: &SearchParams) -> Result<AtomFeed, ArxivError>;
    async fn metadata_by_id(&self, paper_id: &str) -> Result<AtomFeed, ArxivError>;
    async fn fetch_html(&self, paper_id: &str) -> Result<Option<Vec<u8>>, ArxivError>;

    async fn fetch_pdf(&self, paper_id: &str) -> Result<Vec<u8>, ArxivError>;
}
