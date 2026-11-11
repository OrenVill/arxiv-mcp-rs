//! SHA-256 snapshots of `prompts/get` user message text (SPEC §13.2).

use std::collections::HashMap;

use arxiv_mcp::{all_prompt_definitions, prompt_text_from_map, prompt_text_sha256, PROMPT_NAMES};

fn map(pairs: &[(&str, &str)]) -> HashMap<String, String> {
    pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

#[test]
fn lists_seven_upstream_prompts() {
    let defs = all_prompt_definitions();
    assert_eq!(defs.len(), 7);
    let mut names: Vec<_> = defs.iter().map(|p| p.name.as_str()).collect();
    names.sort_unstable();
    let mut expected = PROMPT_NAMES.to_vec();
    expected.sort_unstable();
    assert_eq!(names, expected);
}

/// Fixture args aligned with upstream arxiv-mcp-server 0.8.2 golden output.
#[test]
fn prompt_sha256_snapshots() {
    let cases: Vec<(&str, HashMap<String, String>, &str)> = vec![
        (
            "research-discovery",
            map(&[
                ("topic", "transformers"),
                ("expertise_level", "intermediate"),
                ("time_period", "2023-present"),
                ("domain", "computer_science"),
            ]),
            "b800fe7742fa8dc8f8ca1a37f2f524b44b1c25e453df5821dea0132b338322a2",
        ),
        (
            "deep-paper-analysis",
            map(&[("paper_id", "1706.03762")]),
            "a62fa8266e6b2f6cfd8528ae95d90927aa766e020ca6cff05833a1e45d46f97a",
        ),
        (
            "summarize_paper",
            map(&[("paper_id", "1706.03762")]),
            "dab3bc828607cccf141b0ccd42c280b3b5db9c2c5a2672943ccd7ec84f788289",
        ),
        (
            "compare_papers",
            map(&[("paper_ids", "1706.03762,1810.04805")]),
            "c8828cea86d6f0ec729331849ac7d38af57c472159bb31e8927e4663b167b9cd",
        ),
        (
            "literature_review",
            map(&[
                ("topic", "attention mechanisms"),
                ("paper_ids", "1706.03762"),
            ]),
            "29996fe1a19bafaf3eabecbdd77cda97d5ff7daad9225b8d94f9aa884e692a41",
        ),
        (
            "literature-synthesis",
            map(&[
                ("paper_ids", "1706.03762,1810.04805"),
                ("synthesis_type", "methods"),
                ("domain", "computer_science"),
            ]),
            "732c3a5f926c4ec5f88818837192c418e953c7feeec5668768c9b9ca88f6b543",
        ),
        (
            "research-question",
            map(&[
                ("paper_ids", "1706.03762"),
                ("topic", "efficient transformers"),
                ("domain", "computer_science"),
            ]),
            "86b49649c85999ea9831ae6a2a1e719e6201deb244a9f5e7eee2f65c1522f761",
        ),
    ];

    for (name, args, expected_sha) in cases {
        let text = prompt_text_from_map(name, &args).expect("render prompt");
        let sha = prompt_text_sha256(&text);
        assert_eq!(sha, expected_sha, "SHA-256 mismatch for prompt {name}");
    }
}

#[test]
fn optional_args_use_upstream_defaults() {
    let cases: Vec<(&str, HashMap<String, String>, &str)> = vec![
        (
            "research-discovery",
            map(&[("topic", "transformers")]),
            "38816b2a9c5a94ab633a7de900a2b057d0d13b2776f01f83a020254bbb80dcf3",
        ),
        (
            "literature_review",
            map(&[("topic", "attention mechanisms")]),
            "3f00b638619379cc5c91f06bbda6d7fae8ec018e5da8ee7e15264ab1ca7654ff",
        ),
        (
            "literature-synthesis",
            map(&[("paper_ids", "1706.03762")]),
            "a6e312cc2b088257de76320c170a6698f1d162cc4ff9121ef338a3f4a3369441",
        ),
        (
            "research-question",
            map(&[
                ("paper_ids", "1706.03762"),
                ("topic", "efficient transformers"),
            ]),
            "890d3270d3917c2c9c949d64225ae5fdaa16ee1186a969d43abdd8afb2de9db6",
        ),
    ];

    for (name, args, expected_sha) in cases {
        let text = prompt_text_from_map(name, &args).expect("render prompt");
        assert_eq!(prompt_text_sha256(&text), expected_sha);
    }
}
