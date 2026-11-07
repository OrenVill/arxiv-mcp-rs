//! Parse arXiv Atom API XML feeds into paper records.

use quick_xml::events::Event;
use quick_xml::name::QName;
use quick_xml::reader::Reader;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaperRecord {
    pub id: String,
    pub versioned_id: String,
    pub title: String,
    pub authors: Vec<String>,
    pub abstract_text: String,
    pub categories: Vec<String>,
    pub published: String,
    pub url: String,
    pub resource_uri: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AtomFeed {
    pub papers: Vec<PaperRecord>,
    pub total_results: Option<u64>,
}

#[derive(Debug, Error)]
pub enum AtomParseError {
    #[error("invalid XML: {0}")]
    Xml(String),
}

pub fn parse_atom_feed(xml: &str) -> Result<AtomFeed, AtomParseError> {
    Ok(AtomFeed {
        total_results: parse_opensearch_total(xml),
        papers: parse_entries(xml)?,
    })
}

fn parse_opensearch_total(xml: &str) -> Option<u64> {
    let needle = "totalResults>";
    let start = xml.find(needle)? + needle.len();
    let rest = &xml[start..];
    let end = rest.find('<')?;
    rest[..end].trim().parse().ok()
}

fn parse_entries(xml: &str) -> Result<Vec<PaperRecord>, AtomParseError> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    let mut papers = Vec::new();
    let mut in_entry = false;
    let mut capture: Option<CaptureTarget> = None;
    let mut text_buf = String::new();
    let mut id_url = String::new();
    let mut title = String::new();
    let mut summary = String::new();
    let mut published = String::new();
    let mut authors: Vec<String> = Vec::new();
    let mut categories: Vec<String> = Vec::new();
    let mut pdf_url: Option<String> = None;
    let mut in_author = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let local = local_name(e.name());
                if local == "entry" {
                    in_entry = true;
                    id_url.clear();
                    title.clear();
                    summary.clear();
                    published.clear();
                    authors.clear();
                    categories.clear();
                    pdf_url = None;
                    continue;
                }
                if !in_entry {
                    continue;
                }
                match local.as_str() {
                    "author" => in_author = true,
                    "link" => {
                        if attr_local(&e, "title").as_deref() == Some("pdf") {
                            pdf_url = attr_local(&e, "href");
                        }
                    }
                    "primary_category" | "category" => {
                        if let Some(term) = attr_local(&e, "term") {
                            push_unique(&mut categories, term);
                        }
                    }
                    "id" => {
                        capture = Some(CaptureTarget::Id);
                        text_buf.clear();
                    }
                    "title" if !in_author => {
                        capture = Some(CaptureTarget::Title);
                        text_buf.clear();
                    }
                    "summary" => {
                        capture = Some(CaptureTarget::Summary);
                        text_buf.clear();
                    }
                    "published" => {
                        capture = Some(CaptureTarget::Published);
                        text_buf.clear();
                    }
                    "name" if in_author => {
                        capture = Some(CaptureTarget::AuthorName);
                        text_buf.clear();
                    }
                    _ => {}
                }
            }
            Ok(Event::Text(t)) => {
                if capture.is_some() {
                    if let Ok(s) = t.unescape() {
                        for part in s.split('\n') {
                            let piece = part.trim();
                            if !piece.is_empty() {
                                if !text_buf.is_empty() {
                                    text_buf.push(' ');
                                }
                                text_buf.push_str(piece);
                            }
                        }
                    }
                }
            }
            Ok(Event::End(e)) => {
                let local = local_name(e.name());
                if local == "entry" {
                    if let Some(paper) = build_paper(
                        &id_url,
                        &title,
                        &summary,
                        &published,
                        &authors,
                        &categories,
                        pdf_url.as_deref(),
                    ) {
                        papers.push(paper);
                    }
                    in_entry = false;
                    capture = None;
                    continue;
                }
                if !in_entry {
                    continue;
                }
                if let Some(target) = capture.take() {
                    match target {
                        CaptureTarget::Id => id_url = text_buf.clone(),
                        CaptureTarget::Title => title = text_buf.clone(),
                        CaptureTarget::Summary => summary = text_buf.clone(),
                        CaptureTarget::Published => published = text_buf.clone(),
                        CaptureTarget::AuthorName => {
                            if !text_buf.is_empty() {
                                authors.push(text_buf.clone());
                            }
                        }
                    }
                    text_buf.clear();
                }
                if local == "author" {
                    in_author = false;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(AtomParseError::Xml(e.to_string())),
            _ => {}
        }
        buf.clear();
    }
    Ok(papers)
}

fn build_paper(
    id_url: &str,
    title: &str,
    summary: &str,
    published: &str,
    authors: &[String],
    categories: &[String],
    pdf_url: Option<&str>,
) -> Option<PaperRecord> {
    if id_url.is_empty() {
        return None;
    }
    let paper_id = id_url.split("/abs/").nth(1)?.to_string();
    let short_id = strip_version_suffix(&paper_id);
    let url = pdf_url
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("https://arxiv.org/pdf/{paper_id}"));
    Some(PaperRecord {
        id: short_id.clone(),
        versioned_id: paper_id,
        title: title.to_string(),
        authors: authors.to_vec(),
        abstract_text: summary.to_string(),
        categories: categories.to_vec(),
        published: published.to_string(),
        url,
        resource_uri: format!("arxiv://{short_id}"),
    })
}

fn strip_version_suffix(id: &str) -> String {
    if let Some((base, ver)) = id.rsplit_once('v') {
        if !ver.is_empty() && ver.chars().all(|c| c.is_ascii_digit()) {
            return base.to_string();
        }
    }
    id.to_string()
}

fn push_unique(vec: &mut Vec<String>, item: String) {
    if !vec.contains(&item) {
        vec.push(item);
    }
}

fn local_name(qname: QName<'_>) -> String {
    let raw = qname.as_ref();
    if let Some(brace) = raw.iter().position(|&b| b == b'}') {
        String::from_utf8_lossy(&raw[brace + 1..]).into_owned()
    } else {
        String::from_utf8_lossy(raw).into_owned()
    }
}

fn attr_local(e: &quick_xml::events::BytesStart<'_>, local: &str) -> Option<String> {
    e.attributes()
        .filter_map(|a| a.ok())
        .find(|a| local_name(a.key) == local)
        .and_then(|a| String::from_utf8(a.value.into_owned()).ok())
}

#[derive(Clone, Copy)]
enum CaptureTarget {
    Id,
    Title,
    Summary,
    Published,
    AuthorName,
}

#[cfg(test)]
mod tests {
    use super::*;

    const SEARCH_FIXTURE: &str = include_str!("../tests/fixtures/search_atom.xml");

    #[test]
    fn parses_search_fixture() {
        let feed = parse_atom_feed(SEARCH_FIXTURE).expect("parse");
        assert_eq!(feed.total_results, Some(42));
        assert_eq!(feed.papers.len(), 1);
        assert_eq!(feed.papers[0].id, "1706.03762");
    }

    #[test]
    fn parses_id_list_fixture() {
        let xml = include_str!("../tests/fixtures/id_list_atom.xml");
        let feed = parse_atom_feed(xml).expect("parse");
        assert_eq!(feed.papers.len(), 1);
    }
}
