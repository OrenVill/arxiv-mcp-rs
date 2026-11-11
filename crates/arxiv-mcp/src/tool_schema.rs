//! JSON Schema for MCP tools (agent-facing descriptions and enums).

use rmcp::schemars;
use serde::{Deserialize, Deserializer};

/// Parse the agent-facing `categories` string into arXiv category codes.
pub fn categories_string_to_vec(categories: &str) -> Vec<String> {
    categories
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect()
}

fn deserialize_categories<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Raw {
        Text(String),
        List(Vec<String>),
    }
    match Raw::deserialize(deserializer)? {
        Raw::Text(text) => Ok(text),
        Raw::List(list) => Ok(list.join(",")),
    }
}

/// How much abstract text to include in each search hit.
#[derive(Debug, Clone, Copy, Default, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AbstractModeArg {
    /// Metadata only; omit abstract body from each hit.
    #[schemars(description = "Omit abstract text from search results.")]
    None,
    /// Short excerpt per paper (default upstream behavior).
    #[default]
    #[schemars(description = "Include a short abstract snippet (~280 characters) per hit.")]
    Snippet,
    /// Full abstract text when arXiv provides it.
    #[schemars(description = "Include the full abstract text for each hit.")]
    Full,
}

/// Search result ordering.
#[derive(Debug, Clone, Copy, Default, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SortByArg {
    /// Best match first (default).
    #[default]
    #[schemars(description = "Sort by relevance to the query.")]
    Relevance,
    /// Newest submission date first.
    #[schemars(description = "Sort by submission date (newest first).")]
    Date,
}

#[derive(Debug, Default, Deserialize, schemars::JsonSchema)]
#[schemars(
    title = "search_papers",
    description = "Search the arXiv catalog. Returns JSON with paper ids, titles, authors, and optional abstract snippets. \
Example: {\"query\":\"attention\",\"abstract_mode\":\"snippet\",\"sort_by\":\"relevance\",\"categories\":\"cs.LG\"}."
)]
pub struct SearchPapersToolInput {
    /// Search terms or field prefixes (`ti:`, `au:`, `abs:`, `cat:`, …). Example: `transformer attention`.
    #[schemars(example = "au:Hinton cat:cs.LG")]
    pub query: String,
    /// Number of results to return (1–50). Default 10.
    #[serde(default)]
    pub max_results: Option<u32>,
    /// Zero-based offset for pagination when the result set is larger than `max_results`.
    #[serde(default)]
    pub start: Option<u32>,
    /// How abstract text appears in each hit. Default `snippet`.
    #[serde(default)]
    pub abstract_mode: AbstractModeArg,
    /// Filter submissions on or after this date (`YYYYMMDD`).
    #[serde(default)]
    #[schemars(example = "20240101")]
    pub date_from: Option<String>,
    /// Filter submissions on or before this date (`YYYYMMDD`).
    #[serde(default)]
    #[schemars(example = "20241231")]
    pub date_to: Option<String>,
    /// arXiv category codes to AND. Use `cs.LG`, comma-separated `cs.LG,stat.ML`, or empty string for all subjects.
    #[serde(default, deserialize_with = "deserialize_categories")]
    #[schemars(
        example = "cs.LG",
        description = "Category filter as one string (not a JSON array). Examples: cs.LG or cs.LG,stat.ML."
    )]
    pub categories: String,
    /// Sort order for hits. Default `relevance`.
    #[serde(default)]
    pub sort_by: SortByArg,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[schemars(
    title = "get_abstract",
    description = "Load title, authors, dates, and abstract for one paper id without downloading full text to the store."
)]
pub struct GetAbstractInput {
    /// Bare or versioned arXiv id (not a URL). Examples: `1706.03762`, `1706.03762v7`.
    #[schemars(example = "1706.03762")]
    pub paper_id: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[schemars(
    title = "download_paper",
    description = "Fetch HTML or PDF from arXiv, convert to Markdown, save under the local store, and return a bounded JSON page of text."
)]
pub struct DownloadPaperToolInput {
    /// Bare or versioned arXiv id (not a URL). Examples: `1706.03762`, `1706.03762v7`.
    #[schemars(example = "1706.03762")]
    pub paper_id: String,
    /// Character offset into the converted Markdown (for pagination). Default 0.
    #[serde(default)]
    pub start: Option<u32>,
    /// Maximum characters in this response chunk. Default from server config (often 12000).
    #[serde(default)]
    #[schemars(example = 8000)]
    pub max_chars: Option<u32>,
    /// When true, return the entire stored body in one response (still subject to server caps).
    #[serde(default)]
    pub return_full_text: Option<bool>,
    /// When true, bypass cache and re-download from arXiv even if a fresh manifest exists.
    #[serde(default)]
    pub force: Option<bool>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[schemars(
    title = "list_papers",
    description = "List papers already downloaded into the local arxiv-mcp-rs store (not live arXiv search)."
)]
pub struct ListPapersInput {
    /// When true, return only ids and titles; when false, include manifest fields (authors, dates, versions).
    #[serde(default)]
    pub compact: Option<bool>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[schemars(
    title = "read_paper",
    description = "Read Markdown for a paper that was previously downloaded with `download_paper`."
)]
pub struct ReadPaperToolInput {
    /// Bare or versioned arXiv id matching a stored paper (not a URL).
    #[schemars(example = "1706.03762")]
    pub paper_id: String,
    /// Character offset into stored Markdown. Default 0.
    #[serde(default)]
    pub start: Option<u32>,
    /// Maximum characters in this response chunk. Default from server config.
    #[serde(default)]
    #[schemars(example = 8000)]
    pub max_chars: Option<u32>,
    /// When true, return the full stored body in one response.
    #[serde(default)]
    pub return_full_text: Option<bool>,
}

impl AbstractModeArg {
    pub fn to_parse_string(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Snippet => "snippet",
            Self::Full => "full",
        }
    }
}

impl SortByArg {
    pub fn to_parse_string(self) -> &'static str {
        match self {
            Self::Relevance => "relevance",
            Self::Date => "date",
        }
    }
}
