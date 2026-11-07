//! Shared types and arXiv ID validation.

mod config;
mod id;

pub use config::Config;
pub use id::{ArxivId, ParseIdError};
