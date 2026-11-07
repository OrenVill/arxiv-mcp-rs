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

    pub async fn get_text(&self, start_url: &str) -> Result<(StatusCode, String), FetchError> {
        let mut url = Url::parse(start_url).map_err(|e| FetchError::Network(e.to_string()))?;
        self.ensure_allowed(&url)?;

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
                url = url
                    .join(location)
                    .map_err(|_| FetchError::BlockedRedirect)?;
                self.ensure_allowed(&url)?;
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

    fn ensure_allowed(&self, url: &Url) -> Result<(), FetchError> {
        let host = url.host_str().unwrap_or_default();
        if url.scheme() != "https" || !AllowlistedClient::is_allowed_host(host) {
            return Err(FetchError::BlockedRedirect);
        }
        Ok(())
    }
}
