//! Egress allowlist (`export.arxiv.org`, `arxiv.org`, `www.arxiv.org`).

#[derive(Debug, Clone)]
pub struct AllowlistedClient {
    pub user_agent: String,
}

impl Default for AllowlistedClient {
    fn default() -> Self {
        Self {
            user_agent: format!(
                "arxiv-mcp-rs/{} (https://github.com/OrenVill/arxiv-mcp-rs; research tool)",
                env!("CARGO_PKG_VERSION")
            ),
        }
    }
}

impl AllowlistedClient {
    pub fn is_allowed_host(host: &str) -> bool {
        matches!(host, "export.arxiv.org" | "arxiv.org" | "www.arxiv.org")
    }
}
