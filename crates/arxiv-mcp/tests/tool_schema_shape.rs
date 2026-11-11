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
        assert!(
            field.get("enum").is_some() || field.get("$ref").is_some(),
            "{key} enum"
        );
    }
    let categories = &props["categories"];
    assert_eq!(categories.get("type"), Some(&serde_json::json!("string")));
}

#[test]
fn categories_accepts_string_or_legacy_array() {
    use arxiv_mcp::tool_schema::{categories_string_to_vec, SearchPapersToolInput};

    let from_str: SearchPapersToolInput = serde_json::from_value(serde_json::json!({
        "query": "test",
        "categories": "cs.LG, stat.ML"
    }))
    .unwrap();
    assert_eq!(
        categories_string_to_vec(&from_str.categories),
        vec!["cs.LG", "stat.ML"]
    );

    let from_arr: SearchPapersToolInput = serde_json::from_value(serde_json::json!({
        "query": "test",
        "categories": ["cs.AI"]
    }))
    .unwrap();
    assert_eq!(
        categories_string_to_vec(&from_arr.categories),
        vec!["cs.AI"]
    );
}
