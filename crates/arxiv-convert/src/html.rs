//! arXiv HTML → Markdown cleanup pipeline.

use scraper::node::Node;
use scraper::{ElementRef, Html, Selector};
use thiserror::Error;

pub const CONVERTER_VERSION: u32 = 1;

#[derive(Debug, Error)]
pub enum ConvertError {
    #[error("invalid UTF-8 in HTML body")]
    Utf8,
    #[error("no extractable content in HTML")]
    Empty,
}

pub fn html_to_markdown(html: &[u8]) -> Result<String, ConvertError> {
    let text = std::str::from_utf8(html).map_err(|_| ConvertError::Utf8)?;
    let document = Html::parse_document(text);
    let root = select_content_root(&document).ok_or(ConvertError::Empty)?;
    let md = render_element(root);
    let trimmed = md.trim().to_string();
    if trimmed.is_empty() {
        return Err(ConvertError::Empty);
    }
    Ok(trimmed)
}

fn select_content_root<'a>(document: &'a Html) -> Option<ElementRef<'a>> {
    const CANDIDATES: &[&str] = &[
        "article.ltx_document",
        "article",
        "main",
        "div.ltx_page_content",
        "#content",
        "body",
    ];
    for sel in CANDIDATES {
        if let Ok(selector) = Selector::parse(sel) {
            if let Some(el) = document.select(&selector).next() {
                return Some(el);
            }
        }
    }
    None
}

fn render_element(el: ElementRef<'_>) -> String {
    let mut out = String::new();
    for child in el.children() {
        match child.value() {
            Node::Text(t) => out.push_str(t),
            Node::Element(e) => {
                let tag = e.name();
                if matches!(tag, "script" | "style" | "noscript") {
                    continue;
                }
                let child_el = ElementRef::wrap(child).expect("element node");
                match tag {
                    "h1" => push_block(&mut out, &render_element(child_el), "# "),
                    "h2" => push_block(&mut out, &render_element(child_el), "## "),
                    "h3" => push_block(&mut out, &render_element(child_el), "### "),
                    "h4" => push_block(&mut out, &render_element(child_el), "#### "),
                    "h5" => push_block(&mut out, &render_element(child_el), "##### "),
                    "h6" => push_block(&mut out, &render_element(child_el), "###### "),
                    "p" | "div" | "section" | "li" | "blockquote" | "ul" | "ol" => {
                        push_block(&mut out, &render_element(child_el), "");
                    }
                    "br" => out.push('\n'),
                    "a" => {
                        let inner = render_element(child_el);
                        let href = e.attr("href").unwrap_or("");
                        if href.is_empty() {
                            out.push_str(&inner);
                        } else {
                            out.push('[');
                            out.push_str(inner.trim());
                            out.push_str("](");
                            out.push_str(href);
                            out.push(')');
                        }
                    }
                    "em" | "i" => {
                        out.push('*');
                        out.push_str(&render_element(child_el));
                        out.push('*');
                    }
                    "strong" | "b" => {
                        out.push_str("**");
                        out.push_str(&render_element(child_el));
                        out.push_str("**");
                    }
                    "code" => {
                        out.push('`');
                        out.push_str(&render_element(child_el));
                        out.push('`');
                    }
                    _ => out.push_str(&render_element(child_el)),
                }
            }
            _ => {}
        }
    }
    out
}

fn push_block(out: &mut String, body: &str, prefix: &str) {
    let trimmed = body.trim();
    if trimmed.is_empty() {
        return;
    }
    if !out.is_empty() && !out.ends_with('\n') {
        out.push('\n');
    }
    if prefix.is_empty() {
        out.push_str(trimmed);
        out.push_str("\n\n");
    } else {
        out.push_str(prefix);
        out.push_str(trimmed);
        out.push_str("\n\n");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_article_headings() {
        let html = r#"<!DOCTYPE html><html><body>
<article class="ltx_document">
<h1 class="ltx_title">Test Title</h1>
<p>Hello world.</p>
</article></body></html>"#;
        let md = html_to_markdown(html.as_bytes()).expect("convert");
        assert!(md.contains("# Test Title"));
        assert!(md.contains("Hello world."));
    }
}
