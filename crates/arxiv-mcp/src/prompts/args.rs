//! Deserialize types for MCP prompt `arguments` (string values).

use rmcp::schemars;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResearchDiscoveryArgs {
    pub topic: String,
    #[serde(default)]
    pub expertise_level: Option<String>,
    #[serde(default)]
    pub time_period: Option<String>,
    #[serde(default)]
    pub domain: Option<String>,
}

#[derive(Debug, Clone, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PaperIdArgs {
    pub paper_id: String,
}

#[derive(Debug, Clone, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PaperIdsArgs {
    pub paper_ids: String,
}

#[derive(Debug, Clone, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LiteratureReviewArgs {
    pub topic: String,
    #[serde(default)]
    pub paper_ids: Option<String>,
}

#[derive(Debug, Clone, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LiteratureSynthesisArgs {
    pub paper_ids: String,
    #[serde(default)]
    pub synthesis_type: Option<String>,
    #[serde(default)]
    pub domain: Option<String>,
}

#[derive(Debug, Clone, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResearchQuestionArgs {
    pub paper_ids: String,
    pub topic: String,
    #[serde(default)]
    pub domain: Option<String>,
}
