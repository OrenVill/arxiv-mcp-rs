//! Guardrails for agent-facing tool JSON Schema shape.

use arxiv_mcp::tool_schema::SearchPapersToolInput;
use rmcp::schemars::schema_for;

#[test]
fn search_papers_schema_avoids_optional_enum_any_of() {
    let schema = serde_json::to_value(schema_for!(SearchPapersToolInput)).expect("schema");
    let props = schema
        .get("properties")
        .and_then(|p| p.as_object())
        .expect("properties object");
    for key in ["abstract_mode", "sort_by"] {
        let field = &props[key];
        assert!(
            field.get("anyOf").is_none(),
            "{key} should be a plain enum, not anyOf: {field}"
        );
        assert!(field.get("enum").is_some() || field.get("$ref").is_some(), "{key} enum");
    }
    let categories = &props["categories"];
    assert_eq!(categories.get("type"), Some(&serde_json::json!("array")));
}
