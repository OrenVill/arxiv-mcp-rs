use std::fmt;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ParseIdError {
    #[error("empty arXiv id")]
    Empty,
    #[error("arXiv id too long")]
    TooLong,
    #[error("invalid arXiv id: {0}")]
    Invalid(String),
}

/// Normalized arXiv identifier (bare or versioned).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ArxivId(pub String);

impl ArxivId {
    pub fn parse(input: &str) -> Result<Self, ParseIdError> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Err(ParseIdError::Empty);
        }
        if trimmed.len() > 120 {
            return Err(ParseIdError::TooLong);
        }
        if trimmed.contains("..") || trimmed.contains('\\') || trimmed.contains("://") {
            return Err(ParseIdError::Invalid(trimmed.to_string()));
        }
        if trimmed
            .chars()
            .any(|c| c.is_control() && !matches!(c, '\t' | '\n' | '\r'))
        {
            return Err(ParseIdError::Invalid(trimmed.to_string()));
        }

        let bare = strip_version_suffix(trimmed);
        if is_new_style(bare) || is_legacy_style(bare) {
            Ok(ArxivId(trimmed.to_string()))
        } else {
            Err(ParseIdError::Invalid(trimmed.to_string()))
        }
    }

    pub fn bare_id(&self) -> String {
        strip_version_suffix(&self.0).to_string()
    }

    pub fn filesystem_stem(&self) -> String {
        self.bare_id().replace('/', "__")
    }

    pub fn arxiv_version(&self) -> Option<String> {
        let bare = strip_version_suffix(&self.0);
        if bare.len() < self.0.len() {
            Some(self.0[bare.len()..].to_string())
        } else {
            None
        }
    }

    pub fn versioned_id(&self) -> String {
        match self.arxiv_version() {
            Some(ver) => format!("{}{ver}", self.bare_id()),
            None => self.bare_id(),
        }
    }

    pub fn html_path_id(&self) -> String {
        self.versioned_id()
    }
}

impl fmt::Display for ArxivId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

fn strip_version_suffix(id: &str) -> &str {
    if let Some((base, ver)) = id.rsplit_once('v') {
        if !ver.is_empty() && ver.chars().all(|c| c.is_ascii_digit()) {
            return base;
        }
    }
    id
}

fn is_new_style(id: &str) -> bool {
    let Some((yyyymm, rest)) = id.split_once('.') else {
        return false;
    };
    yyyymm.len() == 4
        && yyyymm.chars().all(|c| c.is_ascii_digit())
        && (4..=5).contains(&rest.len())
        && rest.chars().all(|c| c.is_ascii_digit())
}

fn is_legacy_style(id: &str) -> bool {
    let Some((archive, rest)) = id.split_once('/') else {
        return false;
    };
    !archive.is_empty()
        && archive
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '.')
        && (7..=11).contains(&rest.len())
        && rest.chars().all(|c| c.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_style_ids() {
        assert!(ArxivId::parse("2404.19756").is_ok());
        assert!(ArxivId::parse("1706.03762v7").is_ok());
    }

    #[test]
    fn legacy_ids() {
        assert!(ArxivId::parse("hep-th/9901001").is_ok());
    }

    #[test]
    fn rejects_url() {
        assert!(ArxivId::parse("https://arxiv.org/abs/2404.19756").is_err());
    }
}
