//! Upstream-aligned prompt metadata (SPEC §8 / `prompts/prompts.py`).

use rmcp::model::{Prompt, PromptArgument};

pub const PROMPT_NAMES: [&str; 7] = [
    "compare_papers",
    "deep-paper-analysis",
    "literature_review",
    "literature-synthesis",
    "research-discovery",
    "research-question",
    "summarize_paper",
];

pub fn research_discovery_arguments() -> Vec<PromptArgument> {
    vec![
        PromptArgument::new("topic")
            .with_description("Research topic or question")
            .with_required(true),
        PromptArgument::new("expertise_level")
            .with_description("User's familiarity (beginner/intermediate/expert)")
            .with_required(false),
        PromptArgument::new("time_period")
            .with_description("Time period for search (e.g., '2023-present')")
            .with_required(false),
        PromptArgument::new("domain")
            .with_description("Academic domain (e.g., computer_science/physics/biology)")
            .with_required(false),
    ]
}

pub fn deep_paper_analysis_arguments() -> Vec<PromptArgument> {
    vec![PromptArgument::new("paper_id")
        .with_description("arXiv paper ID")
        .with_required(true)]
}

pub fn summarize_paper_arguments() -> Vec<PromptArgument> {
    vec![PromptArgument::new("paper_id")
        .with_description("arXiv paper ID")
        .with_required(true)]
}

pub fn compare_papers_arguments() -> Vec<PromptArgument> {
    vec![PromptArgument::new("paper_ids")
        .with_description("Comma-separated list of arXiv paper IDs")
        .with_required(true)]
}

pub fn literature_review_arguments() -> Vec<PromptArgument> {
    vec![
        PromptArgument::new("topic")
            .with_description("Research topic or question")
            .with_required(true),
        PromptArgument::new("paper_ids")
            .with_description("Comma-separated list of arXiv paper IDs")
            .with_required(false),
    ]
}

pub fn literature_synthesis_arguments() -> Vec<PromptArgument> {
    vec![
        PromptArgument::new("paper_ids")
            .with_description("Comma-separated list of arXiv paper IDs")
            .with_required(true),
        PromptArgument::new("synthesis_type")
            .with_description("Synthesis type (themes/methods/timeline/gaps/comprehensive)")
            .with_required(false),
        PromptArgument::new("domain")
            .with_description("Academic domain (e.g., computer_science/physics/biology)")
            .with_required(false),
    ]
}

pub fn research_question_arguments() -> Vec<PromptArgument> {
    vec![
        PromptArgument::new("paper_ids")
            .with_description("Comma-separated list of arXiv paper IDs")
            .with_required(true),
        PromptArgument::new("topic")
            .with_description("Research topic or question")
            .with_required(true),
        PromptArgument::new("domain")
            .with_description("Academic domain (e.g., computer_science/physics/biology)")
            .with_required(false),
    ]
}

/// Static catalog matching upstream `PROMPTS` for tests and parity review.
pub fn all_prompt_definitions() -> Vec<Prompt> {
    vec![
        Prompt::from_raw(
            "research-discovery",
            Some("Begin research exploration on a specific topic"),
            Some(research_discovery_arguments()),
        ),
        Prompt::from_raw(
            "deep-paper-analysis",
            Some("Analyze a specific paper in detail"),
            Some(deep_paper_analysis_arguments()),
        ),
        Prompt::from_raw(
            "summarize_paper",
            Some("Summarize a paper with key methods, results, and limits"),
            Some(summarize_paper_arguments()),
        ),
        Prompt::from_raw(
            "compare_papers",
            Some("Compare two or more papers on methods and results"),
            Some(compare_papers_arguments()),
        ),
        Prompt::from_raw(
            "literature_review",
            Some("Synthesize a literature review for a topic and paper set"),
            Some(literature_review_arguments()),
        ),
        Prompt::from_raw(
            "literature-synthesis",
            Some("Synthesize findings across multiple papers"),
            Some(literature_synthesis_arguments()),
        ),
        Prompt::from_raw(
            "research-question",
            Some("Formulate research questions based on literature"),
            Some(research_question_arguments()),
        ),
    ]
}
