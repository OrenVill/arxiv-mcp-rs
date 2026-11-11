//! Shared types and arXiv ID validation.

mod atom;
mod cache;
mod config;
mod content;
mod gateway;
mod id;
mod legacy;
mod policy;
mod rate;
mod search_query;

pub use atom::{parse_atom_feed, AtomFeed, AtomParseError, PaperRecord};
pub use cache::{paper_to_json_map, CacheKey, CachedAtom, SearchCache};
pub use config::Config;
pub use content::{
    paginate_content, snippet_abstract, ContentPage, ABSTRACT_SNIPPET_CHARS,
    ABSTRACT_TRUNCATION_MARK, CONTENT_WARNING,
};
pub use gateway::{ArxivError, ArxivGateway};
pub use id::{ArxivId, ParseIdError};
pub use legacy::{legacy_flat_store_detected, legacy_python_storage_path};
pub use policy::{
    content_warning_for_chunk, sanitize_returned_text, strict_sanitize_from_env, SanitizedText,
    MANIFEST_UNTRUSTED,
};
pub use rate::ArxivRateLimiter;
pub use search_query::{
    build_html_url, build_id_list_url, build_pdf_url, build_search_url, validate_categories,
    AbstractMode, SearchParams, SearchQueryError, SortBy,
};
