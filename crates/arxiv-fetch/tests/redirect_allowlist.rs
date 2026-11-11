//! Redirect allowlist security scenarios (SPEC §10.2).

use arxiv_fetch::{resolve_redirect, validate_https_allowlisted, AllowlistedClient, FetchError};
use reqwest::Url;

#[test]
fn all_permitted_hosts_accepted() {
    for host in ["export.arxiv.org", "arxiv.org", "www.arxiv.org"] {
        let url = Url::parse(&format!("https://{host}/api/query")).unwrap();
        validate_https_allowlisted(&url).expect(host);
    }
}

#[test]
fn blocked_redirect_host_per_spec_matrix() {
    let start = Url::parse("https://export.arxiv.org/start").unwrap();
    let err = resolve_redirect(&start, "https://evil.com/capture").unwrap_err();
    assert!(matches!(err, FetchError::BlockedRedirect));
}

#[test]
fn allowlisted_client_matches_spec_hosts() {
    assert!(AllowlistedClient::is_allowed_host("arxiv.org"));
    assert!(!AllowlistedClient::is_allowed_host("example.com"));
}
