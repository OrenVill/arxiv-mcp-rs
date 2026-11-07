//! SPEC §10.6 security scenarios for arXiv ID validation (no I/O).

use arxiv_core::{ArxivId, ParseIdError};

#[test]
fn spec_matrix_path_traversal_rejected() {
    assert!(ArxivId::parse("../../../etc/passwd").is_err());
}

#[test]
fn spec_matrix_url_rejected() {
    assert!(ArxivId::parse("https://evil.com/x").is_err());
}

#[test]
fn accepts_representative_new_and_legacy_ids() {
    for id in [
        "2404.19756",
        "1706.03762v7",
        "hep-th/9901001",
        "cs.AI/0001001",
    ] {
        assert!(ArxivId::parse(id).is_ok(), "expected ok for {id}");
    }
}

#[test]
fn rejects_backslash_paths_and_oversize() {
    assert!(ArxivId::parse(r"arxiv\abs\2404.19756").is_err());
    let long = "2".repeat(121);
    assert!(matches!(
        ArxivId::parse(&long),
        Err(ParseIdError::TooLong)
    ));
}

#[test]
fn rejects_non_printable_control_chars() {
    assert!(ArxivId::parse("2404.19756\u{0001}").is_err());
}

#[test]
fn filesystem_stem_never_escapes_paper_root() {
    let id = ArxivId::parse("hep-th/9901001").unwrap();
    assert_eq!(id.filesystem_stem(), "hep-th__9901001");
    assert!(!id.filesystem_stem().contains('/'));
    assert!(!id.filesystem_stem().contains(".."));
}
