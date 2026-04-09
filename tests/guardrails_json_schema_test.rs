//! Integration tests for the JSON Schema guardrail provider.
//!
//! Tests cover: edge cases, complex schemas, full pipeline integration
//! with mockito, composite guardrail scenarios, and violation cap behavior.

use fortified_llm_client::guardrails::{
    json_schema::JsonSchemaGuardrail, GuardrailProvider, Severity,
};
use std::{io::Write, path::PathBuf};

fn create_temp_schema(schema_json: &str) -> tempfile::NamedTempFile {
    let mut temp = tempfile::Builder::new().suffix(".json").tempfile().unwrap();
    temp.write_all(schema_json.as_bytes()).unwrap();
    temp.flush().unwrap();
    temp
}

const SIMPLE_SCHEMA: &str = r#"{
    "type": "object",
    "properties": {
        "name": { "type": "string" },
        "age": { "type": "integer" }
    },
    "required": ["name"]
}"#;

// --- Trait compliance ---

#[tokio::test]
async fn test_trait_implementation_valid_content() {
    let schema_file = create_temp_schema(SIMPLE_SCHEMA);
    let guardrail = JsonSchemaGuardrail::new(schema_file.path().to_path_buf()).unwrap();

    let result = guardrail
        .validate(r#"{"name": "Alice", "age": 30}"#)
        .await
        .unwrap();
    assert!(result.passed);
    assert_eq!(guardrail.name(), "JsonSchemaGuardrail");
    assert!(result.quality_score.is_none());
    assert!(result.violations.is_empty());
}

#[tokio::test]
async fn test_trait_implementation_invalid_content() {
    let schema_file = create_temp_schema(SIMPLE_SCHEMA);
    let guardrail = JsonSchemaGuardrail::new(schema_file.path().to_path_buf()).unwrap();

    let result = guardrail.validate(r#"{"age": 30}"#).await.unwrap();
    assert!(!result.passed);
    assert!(!result.violations.is_empty());
    assert!(result
        .violations
        .iter()
        .any(|v| v.rule == "JSON_SCHEMA_VIOLATION"));
    assert!(result
        .violations
        .iter()
        .all(|v| v.severity == Severity::High));
}

// --- Edge cases ---

#[tokio::test]
async fn test_not_json() {
    let schema_file = create_temp_schema(SIMPLE_SCHEMA);
    let guardrail = JsonSchemaGuardrail::new(schema_file.path().to_path_buf()).unwrap();

    let result = guardrail.validate("plain text response").await.unwrap();
    assert!(!result.passed);
    assert_eq!(result.violations.len(), 1);
    assert_eq!(result.violations[0].rule, "JSON_PARSE_ERROR");
}

#[tokio::test]
async fn test_empty_string() {
    let schema_file = create_temp_schema(SIMPLE_SCHEMA);
    let guardrail = JsonSchemaGuardrail::new(schema_file.path().to_path_buf()).unwrap();

    let result = guardrail.validate("").await.unwrap();
    assert!(!result.passed);
    assert_eq!(result.violations[0].rule, "JSON_PARSE_ERROR");
    assert!(result.violations[0].message.contains("empty"));
}

#[tokio::test]
async fn test_wrong_json_type() {
    let schema_file = create_temp_schema(SIMPLE_SCHEMA);
    let guardrail = JsonSchemaGuardrail::new(schema_file.path().to_path_buf()).unwrap();

    // Schema expects object, got array
    let result = guardrail.validate(r#"[1, 2, 3]"#).await.unwrap();
    assert!(!result.passed);
    assert!(result
        .violations
        .iter()
        .any(|v| v.rule == "JSON_SCHEMA_VIOLATION"));
}

#[tokio::test]
async fn test_violations_have_location() {
    let schema_file = create_temp_schema(SIMPLE_SCHEMA);
    let guardrail = JsonSchemaGuardrail::new(schema_file.path().to_path_buf()).unwrap();

    // name should be string, not number
    let result = guardrail.validate(r#"{"name": 123}"#).await.unwrap();
    assert!(!result.passed);
    let violation = result
        .violations
        .iter()
        .find(|v| v.rule == "JSON_SCHEMA_VIOLATION")
        .unwrap();
    assert!(violation.location.is_some());
}

// --- Constructor errors ---

#[test]
fn test_schema_file_not_found() {
    let result = JsonSchemaGuardrail::new(PathBuf::from("/nonexistent/schema.json"));
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Failed to read schema file"));
}

#[test]
fn test_schema_file_invalid_json() {
    let mut temp = tempfile::Builder::new().suffix(".json").tempfile().unwrap();
    temp.write_all(b"not valid json {{{").unwrap();
    temp.flush().unwrap();

    let result = JsonSchemaGuardrail::new(temp.path().to_path_buf());
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not valid JSON"));
}

#[test]
fn test_schema_file_invalid_schema() {
    let mut temp = tempfile::Builder::new().suffix(".json").tempfile().unwrap();
    temp.write_all(br#"{"type": "invalid_type"}"#).unwrap();
    temp.flush().unwrap();

    let result = JsonSchemaGuardrail::new(temp.path().to_path_buf());
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("not a valid JSON Schema"));
}

// --- Complex schemas ---

#[tokio::test]
async fn test_nested_object_with_refs() {
    let schema = r##"{
        "definitions": {
            "address": {
                "type": "object",
                "properties": {
                    "street": { "type": "string" },
                    "city": { "type": "string" }
                },
                "required": ["street", "city"]
            }
        },
        "type": "object",
        "properties": {
            "billing": { "$ref": "#/definitions/address" },
            "shipping": { "$ref": "#/definitions/address" }
        },
        "required": ["billing"]
    }"##;
    let schema_file = create_temp_schema(schema);
    let guardrail = JsonSchemaGuardrail::new(schema_file.path().to_path_buf()).unwrap();

    let valid = r#"{"billing": {"street": "123 Main St", "city": "Springfield"}}"#;
    let result = guardrail.validate(valid).await.unwrap();
    assert!(result.passed);

    // Missing required nested field
    let invalid = r#"{"billing": {"street": "123 Main St"}}"#;
    let result = guardrail.validate(invalid).await.unwrap();
    assert!(!result.passed);
}

#[tokio::test]
async fn test_agent_sentinel_requirements_schema() {
    let schema = r#"{
        "type": "object",
        "required": ["requirements"],
        "properties": {
            "requirements": {
                "type": "array",
                "items": {
                    "type": "object",
                    "required": ["id", "summary", "priority"],
                    "properties": {
                        "id": { "type": "string", "pattern": "^REQ_[0-9]+$" },
                        "summary": { "type": "string", "maxLength": 200 },
                        "priority": { "enum": ["high", "medium", "low"] }
                    },
                    "additionalProperties": false
                }
            }
        },
        "additionalProperties": false
    }"#;
    let schema_file = create_temp_schema(schema);
    let guardrail = JsonSchemaGuardrail::new(schema_file.path().to_path_buf()).unwrap();

    // Valid
    let valid =
        r#"{"requirements": [{"id": "REQ_001", "summary": "Implement auth", "priority": "high"}]}"#;
    let result = guardrail.validate(valid).await.unwrap();
    assert!(result.passed);

    // Invalid: bad id pattern
    let invalid = r#"{"requirements": [{"id": "BAD", "summary": "Test", "priority": "high"}]}"#;
    let result = guardrail.validate(invalid).await.unwrap();
    assert!(!result.passed);

    // Invalid: wrong priority enum
    let invalid2 =
        r#"{"requirements": [{"id": "REQ_001", "summary": "Test", "priority": "urgent"}]}"#;
    let result = guardrail.validate(invalid2).await.unwrap();
    assert!(!result.passed);

    // Invalid: additional property
    let invalid3 = r#"{"requirements": [{"id": "REQ_001", "summary": "Test", "priority": "high", "extra": true}]}"#;
    let result = guardrail.validate(invalid3).await.unwrap();
    assert!(!result.passed);
}

// --- Full pipeline integration ---

#[tokio::test]
async fn test_output_guardrail_rejects_non_conforming_llm_response() {
    use fortified_llm_client::{config_builder::ConfigBuilder, evaluate, GuardrailProviderConfig};
    use mockito::Server;

    let schema_file = create_temp_schema(SIMPLE_SCHEMA);

    let mut server = Server::new_async().await;
    let _mock = server
        .mock("POST", "/v1/chat/completions")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"choices": [{"message": {"role": "assistant", "content": "{\"wrong_field\": true}"}}]}"#)
        .create_async()
        .await;

    let config = ConfigBuilder::new()
        .api_url(server.url() + "/v1/chat/completions")
        .model("test-model")
        .system_prompt("Extract data")
        .user_prompt("test input")
        .output_guardrails(GuardrailProviderConfig::JsonSchema {
            schema_file: schema_file.path().to_path_buf(),
        })
        .build()
        .unwrap();

    let result = evaluate(config).await.unwrap();
    assert_eq!(result.status, "error");
    assert!(result.error.is_some());
    let error = result.error.unwrap();
    assert_eq!(error.code, "OUTPUT_VALIDATION_FAILED");
    assert!(error.message.contains("JSON_SCHEMA_VIOLATION"));
}

#[tokio::test]
async fn test_output_guardrail_passes_conforming_llm_response() {
    use fortified_llm_client::{config_builder::ConfigBuilder, evaluate, GuardrailProviderConfig};
    use mockito::Server;

    let schema_file = create_temp_schema(SIMPLE_SCHEMA);

    let mut server = Server::new_async().await;
    let _mock = server
        .mock("POST", "/v1/chat/completions")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"choices": [{"message": {"role": "assistant", "content": "{\"name\": \"Alice\", \"age\": 30}"}}]}"#)
        .create_async()
        .await;

    let config = ConfigBuilder::new()
        .api_url(server.url() + "/v1/chat/completions")
        .model("test-model")
        .system_prompt("Extract data")
        .user_prompt("test input")
        .output_guardrails(GuardrailProviderConfig::JsonSchema {
            schema_file: schema_file.path().to_path_buf(),
        })
        .build()
        .unwrap();

    let result = evaluate(config).await.unwrap();
    assert_eq!(result.status, "success");
    assert!(result.response.is_some());
}

// --- Composite guardrail ---

#[tokio::test]
async fn test_composite_regex_plus_json_schema() {
    use fortified_llm_client::{
        config_builder::ConfigBuilder, evaluate, guardrails::config::RegexGuardrailConfig,
        AggregationMode, ExecutionMode, GuardrailProviderConfig, Severity as GuardrailSeverity,
    };
    use mockito::Server;

    let schema_file = create_temp_schema(SIMPLE_SCHEMA);

    let mut server = Server::new_async().await;
    let _mock = server
        .mock("POST", "/v1/chat/completions")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"choices": [{"message": {"role": "assistant", "content": "{\"name\": \"Alice\"}"}}]}"#)
        .create_async()
        .await;

    let config = ConfigBuilder::new()
        .api_url(server.url() + "/v1/chat/completions")
        .model("test-model")
        .system_prompt("Extract data")
        .user_prompt("test input")
        .output_guardrails(GuardrailProviderConfig::Composite {
            providers: vec![
                GuardrailProviderConfig::Regex(RegexGuardrailConfig {
                    max_length_bytes: 100000,
                    patterns_file: None,
                    severity_threshold: GuardrailSeverity::Medium,
                }),
                GuardrailProviderConfig::JsonSchema {
                    schema_file: schema_file.path().to_path_buf(),
                },
            ],
            execution: ExecutionMode::Sequential,
            aggregation: AggregationMode::AllMustPass,
        })
        .build()
        .unwrap();

    let result = evaluate(config).await.unwrap();
    assert_eq!(result.status, "success");
}

#[tokio::test]
async fn test_composite_json_schema_fails_within_composite() {
    use fortified_llm_client::{
        config_builder::ConfigBuilder, evaluate, guardrails::config::RegexGuardrailConfig,
        AggregationMode, ExecutionMode, GuardrailProviderConfig, Severity as GuardrailSeverity,
    };
    use mockito::Server;

    let schema_file = create_temp_schema(SIMPLE_SCHEMA);

    let mut server = Server::new_async().await;
    // LLM returns JSON that passes regex but fails schema (missing required "name")
    let _mock = server
        .mock("POST", "/v1/chat/completions")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{"choices": [{"message": {"role": "assistant", "content": "{\"age\": 30}"}}]}"#,
        )
        .create_async()
        .await;

    let config = ConfigBuilder::new()
        .api_url(server.url() + "/v1/chat/completions")
        .model("test-model")
        .system_prompt("Extract data")
        .user_prompt("test input")
        .output_guardrails(GuardrailProviderConfig::Composite {
            providers: vec![
                GuardrailProviderConfig::Regex(RegexGuardrailConfig {
                    max_length_bytes: 100000,
                    patterns_file: None,
                    severity_threshold: GuardrailSeverity::Medium,
                }),
                GuardrailProviderConfig::JsonSchema {
                    schema_file: schema_file.path().to_path_buf(),
                },
            ],
            execution: ExecutionMode::Sequential,
            aggregation: AggregationMode::AllMustPass,
        })
        .build()
        .unwrap();

    let result = evaluate(config).await.unwrap();
    assert_eq!(result.status, "error");
    let error = result.error.unwrap();
    assert_eq!(error.code, "OUTPUT_VALIDATION_FAILED");
    assert!(error.message.contains("JSON_SCHEMA_VIOLATION"));
}

// --- MAX_VIOLATIONS cap ---

#[tokio::test]
async fn test_max_violations_capped() {
    // Schema requiring 30+ fields — an empty object will produce 30+ violations
    let mut properties = serde_json::Map::new();
    let mut required = Vec::new();
    for i in 0..30 {
        let field = format!("field_{i}");
        properties.insert(field.clone(), serde_json::json!({"type": "string"}));
        required.push(serde_json::Value::String(field));
    }
    let schema = serde_json::json!({
        "type": "object",
        "properties": properties,
        "required": required
    });

    let schema_file = create_temp_schema(&schema.to_string());
    let guardrail = JsonSchemaGuardrail::new(schema_file.path().to_path_buf()).unwrap();

    let result = guardrail.validate("{}").await.unwrap();
    assert!(!result.passed);
    // 25 individual violations + 1 summary = 26 max
    assert!(
        result.violations.len() <= 26,
        "Expected at most 26 violations (25 + summary), got {}",
        result.violations.len()
    );
    // Should have the summary violation
    assert!(result
        .violations
        .iter()
        .any(|v| v.message.contains("more validation errors")));
}
