//! MCP workflow prompts (SPEC §8).

pub mod args;
pub mod definitions;
pub mod render;

use std::collections::HashMap;

use rmcp::model::{PromptMessage, Role};

pub use args::*;
pub use definitions::*;

/// SHA-256 of the primary user message text for stable snapshot testing (SPEC §13.2).
pub fn prompt_text_sha256(text: &str) -> String {
    use sha2::{Digest, Sha256};
    let digest = Sha256::digest(text.as_bytes());
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

pub fn user_message(text: impl Into<String>) -> Vec<PromptMessage> {
    vec![PromptMessage::new_text(Role::User, text.into())]
}

/// Build prompt text from string arguments (mirrors MCP `prompts/get` maps).
pub fn prompt_text_from_map(
    name: &str,
    arguments: &HashMap<String, String>,
) -> Result<String, String> {
    let json = serde_json::to_value(arguments).map_err(|e| e.to_string())?;
    match name {
        "research-discovery" => {
            let args: ResearchDiscoveryArgs =
                serde_json::from_value(json).map_err(|e| e.to_string())?;
            Ok(render::research_discovery(&args))
        }
        "deep-paper-analysis" => {
            let args: PaperIdArgs = serde_json::from_value(json).map_err(|e| e.to_string())?;
            Ok(render::deep_paper_analysis(&args))
        }
        "summarize_paper" => {
            let args: PaperIdArgs = serde_json::from_value(json).map_err(|e| e.to_string())?;
            Ok(render::summarize_paper(&args))
        }
        "compare_papers" => {
            let args: PaperIdsArgs = serde_json::from_value(json).map_err(|e| e.to_string())?;
            Ok(render::compare_papers(&args))
        }
        "literature_review" => {
            let args: LiteratureReviewArgs =
                serde_json::from_value(json).map_err(|e| e.to_string())?;
            Ok(render::literature_review(&args))
        }
        "literature-synthesis" => {
            let args: LiteratureSynthesisArgs =
                serde_json::from_value(json).map_err(|e| e.to_string())?;
            Ok(render::literature_synthesis(&args))
        }
        "research-question" => {
            let args: ResearchQuestionArgs =
                serde_json::from_value(json).map_err(|e| e.to_string())?;
            Ok(render::research_question(&args))
        }
        other => Err(format!("Prompt not found: {other}")),
    }
}
