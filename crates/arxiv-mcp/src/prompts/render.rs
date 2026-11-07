//! Stateless prompt body generation (upstream `handlers.get_prompt` parity).

use super::args::{
    LiteratureReviewArgs, LiteratureSynthesisArgs, PaperIdArgs, PaperIdsArgs, ResearchDiscoveryArgs,
    ResearchQuestionArgs,
};

const DEEP_PAPER_ANALYSIS_BODY: &str = include_str!("../../prompts/deep_paper_analysis.md");
const SUMMARIZE_PAPER_BODY: &str = include_str!("../../prompts/summarize_paper.md");
const COMPARE_PAPERS_BODY: &str = include_str!("../../prompts/compare_papers.md");
const LITERATURE_REVIEW_BODY: &str = include_str!("../../prompts/literature_review.md");

const OUTPUT_STRUCTURE: &str = r"
Present your analysis with the following structure:
1. Executive Summary: 3-5 sentence overview of key contributions
2. Detailed Analysis: Following the specific focus requested
3. Visual Breakdown: Describe key figures/tables and their significance
4. Related Work Map: Position this paper within the research landscape
5. Implementation Notes: Practical considerations for applying these findings
";

fn optional_or_unspecified(value: Option<&str>) -> &str {
    value.filter(|s| !s.is_empty()).unwrap_or("unspecified")
}

pub fn research_discovery(args: &ResearchDiscoveryArgs) -> String {
    let expertise_level = optional_or_unspecified(args.expertise_level.as_deref());
    let time_period = optional_or_unspecified(args.time_period.as_deref());
    let domain = optional_or_unspecified(args.domain.as_deref());
    format!(
        "Create a research discovery plan for: {}.\n\
         Expertise level: {expertise_level}.\n\
         Time period: {time_period}.\n\
         Domain: {domain}.\n\n\
         Use search_papers and get_abstract to map the field before recommending papers. \
         Return key terminology, several focused search queries, foundational and recent \
         papers, major research clusters, disagreements, and a prioritized reading path. \
         Calibrate explanations and assumed background to the expertise level.",
        args.topic
    )
}

pub fn deep_paper_analysis(args: &PaperIdArgs) -> String {
    format!(
        "Analyze paper {}.\n\n{OUTPUT_STRUCTURE}\n\n{DEEP_PAPER_ANALYSIS_BODY}",
        args.paper_id
    )
}

pub fn summarize_paper(args: &PaperIdArgs) -> String {
    format!(
        "Summarize paper {}.\n\n\
         Use list_papers/download_paper/read_paper as needed before summarizing.\n\n\
         {SUMMARIZE_PAPER_BODY}",
        args.paper_id
    )
}

pub fn compare_papers(args: &PaperIdsArgs) -> String {
    format!(
        "Compare papers: {}.\n\n\
         Use list_papers/download_paper/read_paper to gather full text for each paper.\n\n\
         {COMPARE_PAPERS_BODY}",
        args.paper_ids
    )
}

pub fn literature_review(args: &LiteratureReviewArgs) -> String {
    let optional_ids = match args
        .paper_ids
        .as_deref()
        .filter(|s| !s.is_empty())
    {
        Some(ids) => format!("\nFocus papers: {ids}."),
        None => String::new(),
    };
    format!(
        "Create a literature review on topic: {topic}.{optional_ids}\n\n\
         Use search_papers to discover missing papers and read_paper to synthesize evidence.\n\n\
         {LITERATURE_REVIEW_BODY}",
        topic = args.topic
    )
}

pub fn literature_synthesis(args: &LiteratureSynthesisArgs) -> String {
    let synthesis_type = args
        .synthesis_type
        .as_deref()
        .filter(|s| !s.is_empty())
        .unwrap_or("comprehensive");
    let domain = optional_or_unspecified(args.domain.as_deref());
    format!(
        "Synthesize these papers: {}.\n\
         Synthesis type: {synthesis_type}.\n\
         Domain: {domain}.\n\n\
         Use list_papers/download_paper/read_paper to inspect each paper. Compare evidence \
         across papers rather than summarizing them independently. Identify agreements, \
         contradictions, methodological differences, limitations, and open gaps. Cite the \
         relevant paper IDs for every substantive comparison.",
        args.paper_ids
    )
}

pub fn research_question(args: &ResearchQuestionArgs) -> String {
    let domain = optional_or_unspecified(args.domain.as_deref());
    format!(
        "Formulate research questions about: {}.\n\
         Source papers: {}.\n\
         Domain: {domain}.\n\n\
         Use list_papers/download_paper/read_paper to ground the questions in the supplied \
         literature. Produce specific, falsifiable research questions. For each question, \
         state the motivating gap, a plausible hypothesis, an evaluation method, required \
         data or experiments, and the result that would falsify the hypothesis.",
        args.topic, args.paper_ids
    )
}
