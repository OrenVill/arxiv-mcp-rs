//! Shared types and arXiv ID validation.

mod config;
mod id;
mod legacy;

pub use config::Config;
pub use id::{ArxivId, ParseIdError};
pub use legacy::{legacy_flat_store_detected, legacy_python_storage_path};
