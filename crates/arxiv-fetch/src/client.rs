use std::time::Duration;

use anyhow::Context;
use reqwest::{redirect, Client, StatusCode, Url};
use thiserror::Error;

use crate::allowlist::AllowlistedClient;

const MAX_REDIRECTS: usize = 5;
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const READ_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Debug, Error)]
pub enum FetchError {
    #[error("blocked redirect to non-allowlisted host")]
    BlockedRedirect,
    #[error("http {0}")]
    Http(StatusCode),
    #[error("network: {0}")]
    Network(String),
}

pub struct AllowlistedHttp {
    client: Client,
    user_agent: String,
}

/// Reject non-HTTPS URLs or hosts outside the arXiv egress allowlist.
pub fn validate_https_allowlisted(url: &Url) -> Result<(), FetchError> {
    let host = url.host_str().unwrap_or_default();
    if url.scheme() != "https" || !AllowlistedClient::is_allowed_host(host) {
        return Err(FetchError::BlockedRedirect);
    }
    Ok(())
}

/// Apply one redirect hop and re-check the allowlist (used by [`AllowlistedHttp::get_text`]).
pub fn resolve_redirect(current: &Url, location: &str) -> Result<Url, FetchError> {
    let url = current
        .join(location)
        .map_err(|_| FetchError::BlockedRedirect)?;
    validate_https_allowlisted(&url)?;
    Ok(url)
}

impl AllowlistedHttp {
    pub fn new(allowlist: &AllowlistedClient) -> anyhow::Result<Self> {
        let client = Client::builder()
            .user_agent(allowlist.user_agent.clone())
            .redirect(redirect::Policy::none())
            .connect_timeout(CONNECT_TIMEOUT)
            .timeout(READ_TIMEOUT)
            .use_rustls_tls()
            .build()
            .context("build reqwest client")?;
        Ok(Self {
            client,
            user_agent: allowlist.user_agent.clone(),
        })
    }

    pub async fn get_bytes(
        &self,
        start_url: &str,
        max_bytes: usize,
    ) -> Result<(StatusCode, Vec<u8>), FetchError> {
        let mut url = Url::parse(start_url).map_err(|e| FetchError::Network(e.to_string()))?;
        validate_https_allowlisted(&url)?;

        for _ in 0..=MAX_REDIRECTS {
            let response = self
                .client
                .get(url.clone())
                .header(reqwest::header::USER_AGENT, &self.user_agent)
                .send()
                .await
                .map_err(|e| FetchError::Network(e.to_string()))?;
            if response.status().is_redirection() {
                let Some(location) = response.headers().get(reqwest::header::LOCATION) else {
                    return Err(FetchError::Http(response.status()));
                };
                let location = location
                    .to_str()
                    .map_err(|_| FetchError::BlockedRedirect)?;
                url = resolve_redirect(&url, location)?;
                continue;
            }
            let status = response.status();
            if status.is_success() {
                let bytes = response
                    .bytes()
                    .await
                    .map_err(|e| FetchError::Network(e.to_string()))?;
                if bytes.len() > max_bytes {
                    return Err(FetchError::Network(format!(
                        "response exceeds max {max_bytes} bytes"
                    )));
                }
                return Ok((status, bytes.to_vec()));
            }
            return Err(FetchError::Http(status));
        }
        Err(FetchError::BlockedRedirect)
    }

    pub async fn get_text(&self, start_url: &str) -> Result<(StatusCode, String), FetchError> {
        let mut url = Url::parse(start_url).map_err(|e| FetchError::Network(e.to_string()))?;
        validate_https_allowlisted(&url)?;

        for _ in 0..=MAX_REDIRECTS {
            let response = self
                .client
                .get(url.clone())
                .header(reqwest::header::USER_AGENT, &self.user_agent)
                .send()
                .await
                .map_err(|e| FetchError::Network(e.to_string()))?;
            if response.status().is_redirection() {
                let Some(location) = response.headers().get(reqwest::header::LOCATION) else {
                    return Err(FetchError::Http(response.status()));
                };
                let location = location
                    .to_str()
                    .map_err(|_| FetchError::BlockedRedirect)?;
                url = resolve_redirect(&url, location)?;
                continue;
            }
            let status = response.status();
            if status.is_success() {
                let text = response
                    .text()
                    .await
                    .map_err(|e| FetchError::Network(e.to_string()))?;
                return Ok((status, text));
            }
            return Err(FetchError::Http(status));
        }
        Err(FetchError::BlockedRedirect)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::Url;

    #[test]
    fn allows_export_arxiv_https() {
        let url = Url::parse("https://export.arxiv.org/api/query?id_list=1234.5678").unwrap();
        assert!(validate_https_allowlisted(&url).is_ok());
    }

    #[test]
    fn blocks_http_and_unknown_hosts() {
        let http = Url::parse("http://arxiv.org/abs/1234.5678").unwrap();
        assert!(matches!(
            validate_https_allowlisted(&http),
            Err(FetchError::BlockedRedirect)
        ));
        let evil = Url::parse("https://evil.com/x").unwrap();
        assert!(matches!(
            validate_https_allowlisted(&evil),
            Err(FetchError::BlockedRedirect)
        ));
    }

    #[test]
    fn redirect_to_evil_host_is_blocked() {
        let start = Url::parse("https://arxiv.org/redirect").unwrap();
        let err = resolve_redirect(&start, "https://evil.com/steal").unwrap_err();
        assert!(matches!(err, FetchError::BlockedRedirect));
    }

    #[test]
    fn redirect_within_allowlist_ok() {
        let start = Url::parse("https://arxiv.org/redirect").unwrap();
        let next = resolve_redirect(&start, "https://export.arxiv.org/api/query").unwrap();
        assert_eq!(next.host_str(), Some("export.arxiv.org"));
    }
}
