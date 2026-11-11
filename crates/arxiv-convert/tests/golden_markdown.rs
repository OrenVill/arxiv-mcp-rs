//! Optional golden Markdown fixtures under `tests/golden/`.

use std::path::PathBuf;

use arxiv_convert::pdf_to_markdown;

fn golden_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/golden")
}

fn fixture_pdf() -> PathBuf {
    golden_root().join("fixtures/minimal.pdf")
}

fn expected_md() -> PathBuf {
    golden_root().join("fixtures/minimal.expected.md")
}

#[test]
fn golden_markdown_minimal_pdf() {
    let pdf = std::fs::read(fixture_pdf()).expect("read minimal.pdf fixture");
    let got = pdf_to_markdown(&pdf).expect("convert golden PDF");
    let update = std::env::var("UPDATE_GOLDEN")
        .ok()
        .is_some_and(|v| v == "1");

    if update {
        std::fs::write(expected_md(), format!("{got}\n")).expect("write golden");
        return;
    }

    let want = std::fs::read_to_string(expected_md()).expect("read golden md");
    let normalize = |s: &str| s.trim().to_lowercase();
    assert_eq!(
        normalize(&got),
        normalize(&want),
        "Markdown output drifted; see tests/golden/KNOWN_DIFFERENCES.md or run UPDATE_GOLDEN=1"
    );
}
