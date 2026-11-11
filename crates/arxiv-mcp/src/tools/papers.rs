use std::time::Duration;

use arxiv_convert::{html_to_markdown, pdf_to_markdown, ConvertError, CONVERTER_VERSION};
use arxiv_core::{content_warning_for_chunk, paginate_content, ArxivError, ArxivId, ContentPage};
use arxiv_store::{FetchedVia, ManifestV1, StoreBackend, StoreError, StoreRecord};
use chrono::Utc;
use serde_json::json;

use crate::tools::{error_json, json_pretty};
use crate::AppState;

fn max_download_time() -> Duration {
    std::env::var("ARXIV_MAX_TOTAL_TIME")
        .ok()
        .and_then(|s| s.parse().ok())
        .map(Duration::from_secs)
        .unwrap_or(Duration::from_secs(50))
}

pub struct DownloadPaperInput {
    pub paper_id: String,
    pub start: usize,
    pub max_chars: usize,
    pub return_full_text: bool,
    pub force: bool,
}

pub struct ReadPaperInput {
    pub paper_id: String,
    pub start: usize,
    pub max_chars: usize,
    pub return_full_text: bool,
}

pub async fn download_paper(state: &AppState, input: DownloadPaperInput) -> String {
    let parsed = match ArxivId::parse(input.paper_id.trim()) {
        Ok(id) => id,
        Err(_) => return error_json("invalid arXiv ID format"),
    };
    let bare_key = parsed.bare_id();

    if !input.force {
        if let Ok(true) = state.store.has_paper(&parsed).await {
            if let Ok(stored) = state
                .store
                .read_paper(&ArxivId::parse(&bare_key).unwrap())
                .await
            {
                if stored.manifest.converter_version == CONVERTER_VERSION {
                    return paper_success_json(
                        &stored.manifest,
                        &stored.content,
                        input.start,
                        input.max_chars,
                        input.return_full_text,
                        true,
                    );
                }
            }
        }
    }

    let state_for_job = state.clone();
    let id_for_job = parsed.clone();
    let force = input.force;
    let download = state
        .in_flight
        .coalesce(&bare_key, move || {
            let state = state_for_job.clone();
            let id = id_for_job.clone();
            async move { perform_download(state, id, force).await }
        })
        .await;

    if let Err(message) = download {
        return error_json(message);
    }

    match state
        .store
        .read_paper(&ArxivId::parse(&bare_key).unwrap())
        .await
    {
        Ok(stored) => paper_success_json(
            &stored.manifest,
            &stored.content,
            input.start,
            input.max_chars,
            input.return_full_text,
            true,
        ),
        Err(e) => store_error_json(&bare_key, e),
    }
}

async fn perform_download(state: AppState, id: ArxivId, _force: bool) -> Result<(), String> {
    let bare_key = id.bare_id();
    let deadline = max_download_time();
    tokio::time::timeout(deadline, async {
        let metadata = state
            .gateway
            .metadata_by_id(&bare_key)
            .await
            .map_err(download_error)?;

        let paper = metadata
            .papers
            .first()
            .ok_or_else(|| "Paper not found on arXiv".to_string())?;

        let html_id = id.html_path_id();
        let pdf_id = id.versioned_id();

        let (markdown, fetched_via) = match state.gateway.fetch_html(&html_id).await {
            Ok(Some(html_bytes)) => match html_to_markdown(&html_bytes) {
                Ok(md) => (md, FetchedVia::Html),
                Err(ConvertError::Empty) => fetch_pdf_markdown(&state, &pdf_id).await?,
                Err(e) => return Err(format!("HTML conversion failed: {e}")),
            },
            Ok(None) => fetch_pdf_markdown(&state, &pdf_id).await?,
            Err(e) => return Err(download_error(e)),
        };

        let mut manifest = ManifestV1::new(&bare_key);
        manifest.title = Some(paper.title.clone());
        manifest.authors = paper.authors.clone();
        manifest.published = Some(paper.published.clone());
        manifest.converter_version = CONVERTER_VERSION;
        manifest.fetched_via = fetched_via;
        manifest.fetched_at = Utc::now().to_rfc3339();
        manifest.arxiv_version = id.arxiv_version();
        manifest.versioned_id = Some(id.versioned_id());

        state
            .store
            .write_paper(&StoreRecord {
                manifest,
                content: markdown,
            })
            .await
            .map_err(|e| match e {
                StoreError::QuotaExceeded => {
                    "Store quota exceeded; cannot download new papers".to_string()
                }
                other => other.to_string(),
            })
    })
    .await
    .map_err(|_| format!("download timed out after {}s", deadline.as_secs()))?
}

async fn fetch_pdf_markdown(
    state: &AppState,
    pdf_id: &str,
) -> Result<(String, FetchedVia), String> {
    let pdf_bytes = state
        .gateway
        .fetch_pdf(pdf_id)
        .await
        .map_err(download_error)?;
    let markdown =
        pdf_to_markdown(&pdf_bytes).map_err(|e| format!("PDF conversion failed: {e}"))?;
    Ok((markdown, FetchedVia::Pdf))
}

fn download_error(err: ArxivError) -> String {
    match err {
        ArxivError::RateLimited {
            retry_after_seconds,
            http_status,
            message,
        } => format!(
            "{} (retry_after_seconds={retry_after_seconds}, http_status={http_status})",
            message
        ),
        ArxivError::Http { status } => format!("arXiv HTTP error (HTTP {status})"),
        ArxivError::BlockedRedirect => "Blocked redirect to non-allowlisted host".into(),
        ArxivError::Network(e) => format!("Network error: {e}"),
        other => other.to_string(),
    }
}

pub async fn list_papers(state: &AppState, compact: bool) -> String {
    match state.store.list_papers(compact).await {
        Ok(response) => {
            if compact {
                let ids: Vec<_> = response.papers.into_iter().map(|p| p.id).collect();
                json_pretty(&json!({
                    "status": "success",
                    "papers": ids,
                }))
            } else {
                let papers: Vec<_> = response
                    .papers
                    .into_iter()
                    .map(|p| {
                        json!({
                            "id": p.id,
                            "title": p.title,
                            "authors": p.authors,
                            "published": p.published,
                            "arxiv_version": p.arxiv_version,
                            "versioned_id": p.versioned_id,
                        })
                    })
                    .collect();
                json_pretty(&json!({
                    "status": "success",
                    "papers": papers,
                }))
            }
        }
        Err(e) => store_error_json("list", e),
    }
}

pub async fn read_paper(state: &AppState, input: ReadPaperInput) -> String {
    let parsed = match ArxivId::parse(input.paper_id.trim()) {
        Ok(id) => id,
        Err(_) => return error_json("invalid arXiv ID format"),
    };
    let bare_key = parsed.bare_id();
    match state
        .store
        .read_paper(&ArxivId::parse(&bare_key).unwrap())
        .await
    {
        Ok(stored) => paper_success_json(
            &stored.manifest,
            &stored.content,
            input.start,
            input.max_chars,
            input.return_full_text,
            input.start == 0,
        ),
        Err(StoreError::NotFound(_)) => error_json(format!(
            "Paper {bare_key} is not in the local store. Call download_paper first to fetch and convert it."
        )),
        Err(e) => store_error_json(&bare_key, e),
    }
}

fn store_error_json(context: &str, err: StoreError) -> String {
    error_json(format!("{context}: {err}"))
}

fn paper_success_json(
    manifest: &ManifestV1,
    content: &str,
    start: usize,
    max_chars: usize,
    return_full_text: bool,
    include_warning: bool,
) -> String {
    let page = paginate_content(content, start, max_chars, return_full_text, include_warning);
    json_pretty(&build_paper_body(manifest, &page))
}

fn build_paper_body(manifest: &ManifestV1, page: &ContentPage) -> serde_json::Value {
    let mut body = json!({
        "status": "success",
        "paper_id": manifest.paper_id,
        "title": manifest.title,
        "authors": manifest.authors,
        "published": manifest.published,
        "arxiv_version": manifest.arxiv_version,
        "versioned_id": manifest.versioned_id,
        "fetched_via": serde_json::to_value(manifest.fetched_via).unwrap_or(json!("html")),
        "untrusted": manifest.untrusted,
        "content_length": page.content_length,
        "returned_chars": page.returned_chars,
        "start": page.start,
        "next_start": page.next_start,
        "is_truncated": page.is_truncated,
        "content": page.content,
    });
    if let Some(warning) = content_warning_for_chunk(page.start, page.include_warning) {
        body["content_warning"] = json!(warning);
    }
    if page.is_truncated {
        body["next_retrieval"] = json!({
            "paper_id": manifest.paper_id,
            "start": page.next_start,
            "max_chars": page.returned_chars,
        });
    }
    body
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    use arxiv_core::{parse_atom_feed, ArxivGateway, AtomFeed};
    use arxiv_store::{LocalFsStore, StoreBackend};
    use async_trait::async_trait;
    use tempfile::tempdir;

    use crate::in_flight::InFlightDownloads;

    struct HtmlGateway {
        html: Option<Vec<u8>>,
        pdf: Vec<u8>,
    }

    #[async_trait]
    impl ArxivGateway for HtmlGateway {
        async fn search(&self, _params: &arxiv_core::SearchParams) -> Result<AtomFeed, ArxivError> {
            Err(ArxivError::Query("unused".into()))
        }

        async fn metadata_by_id(&self, _paper_id: &str) -> Result<AtomFeed, ArxivError> {
            parse_atom_feed(include_str!(
                "../../../arxiv-core/tests/fixtures/id_list_atom.xml"
            ))
            .map_err(ArxivError::Parse)
        }

        async fn fetch_html(&self, _paper_id: &str) -> Result<Option<Vec<u8>>, ArxivError> {
            Ok(self.html.clone())
        }

        async fn fetch_pdf(&self, _paper_id: &str) -> Result<Vec<u8>, ArxivError> {
            Ok(self.pdf.clone())
        }
    }

    fn sample_html() -> Vec<u8> {
        br#"<html><body><article><h1>Title</h1><p>Body text.</p></article></body></html>"#.to_vec()
    }

    fn test_state(gateway: Arc<dyn ArxivGateway>, store: LocalFsStore) -> AppState {
        AppState {
            config: Arc::new(arxiv_core::Config::default()),
            store: Arc::new(store),
            gateway,
            cache: Arc::new(crate::cache::MemorySearchCache::new()),
            in_flight: InFlightDownloads::new(),
        }
    }

    #[tokio::test]
    async fn download_persists_and_read_returns_content() {
        let dir = tempdir().unwrap();
        let config = arxiv_core::Config {
            storage_path: dir.path().to_path_buf(),
            ..Default::default()
        };
        let store = LocalFsStore::new(config.storage_path.clone());
        store.ensure_root().await.unwrap();
        let gateway: Arc<dyn ArxivGateway> = Arc::new(HtmlGateway {
            html: Some(sample_html()),
            pdf: include_bytes!("../../../../tests/golden/fixtures/minimal.pdf").to_vec(),
        });
        let state = test_state(gateway, store);

        let dl = download_paper(
            &state,
            DownloadPaperInput {
                paper_id: "2404.19756".into(),
                start: 0,
                max_chars: 500,
                return_full_text: false,
                force: false,
            },
        )
        .await;
        assert!(dl.contains("\"status\": \"success\""));
        assert!(dl.contains("Body text"));

        let read = read_paper(
            &state,
            ReadPaperInput {
                paper_id: "2404.19756".into(),
                start: 0,
                max_chars: 500,
                return_full_text: false,
            },
        )
        .await;
        assert!(read.contains("Body text"));
    }

    #[tokio::test]
    async fn download_falls_back_to_pdf_when_html_missing() {
        let dir = tempdir().unwrap();
        let config = arxiv_core::Config {
            storage_path: dir.path().to_path_buf(),
            ..Default::default()
        };
        let store = LocalFsStore::new(config.storage_path.clone());
        store.ensure_root().await.unwrap();
        let gateway: Arc<dyn ArxivGateway> = Arc::new(HtmlGateway {
            html: None,
            pdf: include_bytes!("../../../../tests/golden/fixtures/minimal.pdf").to_vec(),
        });
        let state = test_state(gateway, store);

        let dl = download_paper(
            &state,
            DownloadPaperInput {
                paper_id: "2404.19756".into(),
                start: 0,
                max_chars: 500,
                return_full_text: false,
                force: false,
            },
        )
        .await;
        assert!(dl.contains("\"status\": \"success\""));
        assert!(dl.contains("\"fetched_via\": \"pdf\""));
        assert!(dl.to_lowercase().contains("hello"));

        let stored = state
            .store
            .read_paper(&ArxivId::parse("2404.19756").unwrap())
            .await
            .unwrap();
        assert_eq!(stored.manifest.fetched_via, FetchedVia::Pdf);
    }
}
