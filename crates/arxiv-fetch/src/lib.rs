//! Allowlisted arXiv HTTP access.

pub mod allowlist;
mod client;
mod gateway;

pub use allowlist::AllowlistedClient;
pub use client::{resolve_redirect, validate_https_allowlisted, AllowlistedHttp, FetchError};
pub use gateway::ProcessArxivGateway;
