//! PDF text extraction via `pdf-extract` with page caps enforced through `lopdf`.

use lopdf::Document;
use pdf_extract::extract_text_from_mem;
use thiserror::Error;

pub const DEFAULT_MAX_PDF_PAGES: u32 = 500;

#[derive(Debug, Error)]
pub enum PdfConvertError {
    #[error("invalid PDF")]
    InvalidPdf,
    #[error("PDF exceeds page cap ({0} pages)")]
    TooManyPages(u32),
    #[error("no extractable text in PDF")]
    Empty,
    #[error("pdf extraction failed: {0}")]
    Extract(String),
}

fn max_pdf_pages() -> u32 {
    std::env::var("MAX_PDF_PAGES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_MAX_PDF_PAGES)
}

/// Convert PDF bytes to plain Markdown-ish text (paragraph breaks preserved).
pub fn pdf_to_markdown(pdf: &[u8]) -> Result<String, PdfConvertError> {
    let page_count = pdf_page_count(pdf)?;
    let cap = max_pdf_pages();
    if page_count > cap {
        return Err(PdfConvertError::TooManyPages(page_count));
    }

    let raw = extract_text_from_mem(pdf).map_err(|e| PdfConvertError::Extract(e.to_string()))?;
    let trimmed = normalize_pdf_text(&raw);
    if trimmed.is_empty() {
        return Err(PdfConvertError::Empty);
    }
    Ok(trimmed)
}

fn pdf_page_count(pdf: &[u8]) -> Result<u32, PdfConvertError> {
    let doc = Document::load_mem(pdf).map_err(|_| PdfConvertError::InvalidPdf)?;
    let count = doc.get_pages().len();
    u32::try_from(count).map_err(|_| PdfConvertError::InvalidPdf)
}

fn normalize_pdf_text(input: &str) -> String {
    let mut out = String::new();
    let mut prev_blank = true;
    for line in input.lines() {
        let line = line.trim();
        if line.is_empty() {
            if !prev_blank && !out.is_empty() {
                out.push('\n');
            }
            prev_blank = true;
            continue;
        }
        if !prev_blank && !out.is_empty() {
            out.push('\n');
        }
        out.push_str(line);
        prev_blank = false;
    }
    out.trim().to_string()
}
