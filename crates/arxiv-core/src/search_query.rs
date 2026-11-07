use std::collections::HashSet;

const ARXIV_API_URL: &str = "https://export.arxiv.org/api/query";
const MAX_RESULTS_CAP: u32 = 50;

const USER_QUERY_FIELD_PREFIXES: &[&str] = &[
    "ti:", "au:", "abs:", "cat:", "co:", "jr:", "rn:", "all:",
];

const VALID_CATEGORY_PREFIXES: &[&str] = &[
    "cs", "econ", "eess", "math", "physics", "q-bio", "q-fin", "stat", "astro-ph", "cond-mat",
    "gr-qc", "hep-ex", "hep-lat", "hep-ph", "hep-th", "math-ph", "nlin", "nucl-ex", "nucl-th",
    "quant-ph",
];

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SearchQueryError {
    #[error("no search criteria provided")]
    NoCriteria,
    #[error("invalid date: {0}")]
    InvalidDate(String),
    #[error("invalid category")]
    InvalidCategory,
    #[error("invalid sort_by")]
    InvalidSortBy(String),
    #[error("invalid abstract_mode")]
    InvalidAbstractMode(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct SearchParams {
    pub query: String,
    pub max_results: u32,
    pub start: u32,
    pub sort_by: SortBy,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub categories: Vec<String>,
    pub abstract_mode: AbstractMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortBy {
    Relevance,
    Date,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AbstractMode {
    None,
    Snippet,
    Full,
}

impl AbstractMode {
    pub fn parse(value: Option<&str>) -> Result<Self, SearchQueryError> {
        match value {
            None => Ok(Self::Snippet),
            Some(v) => match v.trim().to_ascii_lowercase().as_str() {
                "none" => Ok(Self::None),
                "snippet" => Ok(Self::Snippet),
                "full" => Ok(Self::Full),
                _ => Err(SearchQueryError::InvalidAbstractMode(v.to_string())),
            },
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Snippet => "snippet",
            Self::Full => "full",
        }
    }
}

impl SortBy {
    pub fn parse(value: Option<&str>) -> Result<Self, SearchQueryError> {
        match value {
            None => Ok(Self::Relevance),
            Some(v) => match v.trim().to_ascii_lowercase().as_str() {
                "relevance" => Ok(Self::Relevance),
                "date" => Ok(Self::Date),
                _ => Err(SearchQueryError::InvalidSortBy(v.to_string())),
            },
        }
    }
}

impl SearchParams {
    pub fn max_results_clamped(max_results: Option<u32>) -> u32 {
        max_results.unwrap_or(5).min(MAX_RESULTS_CAP)
    }
}

pub fn validate_categories(categories: &[String]) -> Result<(), SearchQueryError> {
    for category in categories {
        let prefix = category.split('.').next().unwrap_or(category.as_str());
        if !VALID_CATEGORY_PREFIXES.contains(&prefix) {
            return Err(SearchQueryError::InvalidCategory);
        }
    }
    Ok(())
}

pub fn build_search_url(params: &SearchParams) -> Result<String, SearchQueryError> {
    let final_query = build_arxiv_search_query(
        &params.query,
        params.date_from.as_deref(),
        params.date_to.as_deref(),
        &params.categories,
    )?;
    let sort_by = match params.sort_by {
        SortBy::Relevance => "relevance",
        SortBy::Date => "submittedDate",
    };
    let encoded = urlencoding::encode(&final_query);
    Ok(format!(
        "{ARXIV_API_URL}?search_query={encoded}&start={}&max_results={}&sortBy={sort_by}&sortOrder=descending",
        params.start, params.max_results
    ))
}

pub fn build_html_url(paper_id: &str) -> String {
    format!("https://arxiv.org/html/{paper_id}")
}

pub fn build_pdf_url(paper_id: &str) -> String {
    format!("https://arxiv.org/pdf/{paper_id}.pdf")
}

pub fn build_id_list_url(paper_id: &str) -> String {
    format!("{ARXIV_API_URL}?id_list={paper_id}&max_results=1")
}

fn build_arxiv_search_query(
    query: &str,
    date_from: Option<&str>,
    date_to: Option<&str>,
    categories: &[String],
) -> Result<String, SearchQueryError> {
    let mut query_parts: Vec<String> = Vec::new();
    if !query.trim().is_empty() {
        query_parts.push(scope_user_query(query));
    }
    if !categories.is_empty() {
        let category_filter: Vec<String> = categories
            .iter()
            .map(|cat| format!("cat:{cat}"))
            .collect();
        query_parts.push(format!("({})", category_filter.join(" OR ")));
    }
    if date_from.is_some() || date_to.is_some() {
        query_parts.push(format_submitted_date_filter(date_from, date_to)?);
    }
    if query_parts.is_empty() {
        return Err(SearchQueryError::NoCriteria);
    }
    Ok(query_parts.join(" AND "))
}

fn scope_user_query(query: &str) -> String {
    let query = query.trim();
    if user_query_has_field_prefix(query) {
        format!("({query})")
    } else {
        format!("(ti:({query}) OR abs:({query}))")
    }
}

fn user_query_has_field_prefix(query: &str) -> bool {
    let lowered = query.to_ascii_lowercase();
    USER_QUERY_FIELD_PREFIXES
        .iter()
        .any(|prefix| lowered.contains(prefix))
}

fn format_submitted_date_filter(
    date_from: Option<&str>,
    date_to: Option<&str>,
) -> Result<String, SearchQueryError> {
    let start_date = date_from
        .map(parse_yyyy_mm_dd_start)
        .transpose()?
        .unwrap_or_else(|| "199107010000".to_string());
    let end_date = date_to
        .map(parse_yyyy_mm_dd_end)
        .transpose()?
        .unwrap_or_else(|| chrono::Utc::now().format("%Y%m%d2359").to_string());
    Ok(format!("submittedDate:[{start_date} TO {end_date}]"))
}

fn parse_yyyy_mm_dd_start(s: &str) -> Result<String, SearchQueryError> {
    let (y, m, d) = parse_date_parts(s)?;
    Ok(format!("{y:04}{m:02}{d:02}0000"))
}

fn parse_yyyy_mm_dd_end(s: &str) -> Result<String, SearchQueryError> {
    let (y, m, d) = parse_date_parts(s)?;
    Ok(format!("{y:04}{m:02}{d:02}2359"))
}

fn parse_date_parts(s: &str) -> Result<(u32, u32, u32), SearchQueryError> {
    let parts: Vec<&str> = s.trim().split('-').collect();
    if parts.len() != 3 {
        return Err(SearchQueryError::InvalidDate(s.to_string()));
    }
    let y: u32 = parts[0]
        .parse()
        .map_err(|_| SearchQueryError::InvalidDate(s.to_string()))?;
    let m: u32 = parts[1]
        .parse()
        .map_err(|_| SearchQueryError::InvalidDate(s.to_string()))?;
    let d: u32 = parts[2]
        .parse()
        .map_err(|_| SearchQueryError::InvalidDate(s.to_string()))?;
    Ok((y, m, d))
}

pub fn cache_key_for_search(params: &SearchParams) -> String {
    let mut set = HashSet::new();
    for c in &params.categories {
        set.insert(c.clone());
    }
    let mut cats: Vec<_> = set.into_iter().collect();
    cats.sort();
    format!(
        "search:{}:{}:{}:{}:{:?}:{:?}:{}",
        params.query,
        params.max_results,
        params.start,
        sort_key(params.sort_by),
        params.date_from,
        params.date_to,
        cats.join(",")
    )
}

pub fn cache_key_for_abstract(paper_id: &str) -> String {
    format!("abstract:{paper_id}")
}

fn sort_key(sort_by: SortBy) -> &'static str {
    match sort_by {
        SortBy::Relevance => "relevance",
        SortBy::Date => "date",
    }
}
