use crate::{
    error::CliError,
    guardrails::provider::{GuardrailProvider, GuardrailResult, Severity, Violation},
    schema_validator,
};
use async_trait::async_trait;
use std::path::PathBuf;

/// Maximum number of schema validation errors to report individually.
/// When exceeded, remaining errors are summarized in one additional entry.
const MAX_VIOLATIONS: usize = 25;

/// Rule name for JSON parse failures (content is not valid JSON).
pub const RULE_JSON_PARSE_ERROR: &str = "JSON_PARSE_ERROR";
/// Rule name for JSON Schema validation failures (content doesn't match schema).
pub const RULE_JSON_SCHEMA_VIOLATION: &str = "JSON_SCHEMA_VIOLATION";

/// JSON Schema guardrail that validates content against a compiled JSON Schema.
///
/// The schema is compiled once at construction time and reused for all validations.
/// Supports JSON Schema Draft 7 via the `jsonschema` crate.
#[derive(Debug)]
pub struct JsonSchemaGuardrail {
    validator: jsonschema::Validator,
    /// Retained for diagnostic logging only; not used for re-validation.
    schema_path: PathBuf,
}

impl JsonSchemaGuardrail {
    /// Create a new JSON Schema guardrail from a schema file path.
    ///
    /// The schema file is read and compiled at construction time.
    /// Returns an error if the file doesn't exist, isn't valid JSON,
    /// or isn't a valid JSON Schema.
    pub fn new(schema_file: PathBuf) -> Result<Self, CliError> {
        let schema_content = std::fs::read_to_string(&schema_file).map_err(|e| {
            CliError::FileNotFound(format!(
                "JSON Schema guardrail: Failed to read schema file '{}': {}",
                schema_file.display(),
                e
            ))
        })?;

        let schema_value: serde_json::Value =
            serde_json::from_str(&schema_content).map_err(|e| {
                CliError::InvalidArguments(format!(
                    "JSON Schema guardrail: Schema file '{}' is not valid JSON: {}",
                    schema_file.display(),
                    e
                ))
            })?;

        let validator = schema_validator::compile_json_schema(&schema_value).map_err(|e| {
            CliError::InvalidArguments(format!(
                "JSON Schema guardrail: Schema file '{}' is not a valid JSON Schema: {e}",
                schema_file.display(),
            ))
        })?;

        log::info!(
            "JSON Schema guardrail initialized with schema from '{}'",
            schema_file.display()
        );

        Ok(Self {
            validator,
            schema_path: schema_file,
        })
    }
}

#[async_trait]
impl GuardrailProvider for JsonSchemaGuardrail {
    async fn validate(&self, content: &str) -> Result<GuardrailResult, CliError> {
        if content.is_empty() {
            return Ok(GuardrailResult::without_quality_score(
                false,
                vec![Violation {
                    rule: RULE_JSON_PARSE_ERROR.to_string(),
                    severity: Severity::High,
                    message: "Content is empty (expected valid JSON)".to_string(),
                    location: None,
                }],
                vec![],
            ));
        }

        let json_value: serde_json::Value = match serde_json::from_str(content) {
            Ok(v) => v,
            Err(e) => {
                return Ok(GuardrailResult::without_quality_score(
                    false,
                    vec![Violation {
                        rule: RULE_JSON_PARSE_ERROR.to_string(),
                        severity: Severity::High,
                        message: format!("Content is not valid JSON: {e}"),
                        location: None,
                    }],
                    vec![],
                ));
            }
        };

        let errors: Vec<_> = self.validator.iter_errors(&json_value).collect();

        if errors.is_empty() {
            log::debug!(
                "JSON Schema validation passed (schema: '{}')",
                self.schema_path.display()
            );
            return Ok(GuardrailResult::without_quality_score(true, vec![], vec![]));
        }

        let total_errors = errors.len();
        let mut violations: Vec<Violation> = errors
            .into_iter()
            .take(MAX_VIOLATIONS)
            .map(|error| {
                let instance_path = error.instance_path().to_string();
                Violation {
                    rule: RULE_JSON_SCHEMA_VIOLATION.to_string(),
                    severity: Severity::High,
                    message: error.to_string(),
                    location: if instance_path.is_empty() {
                        Some("/".to_string())
                    } else {
                        Some(instance_path)
                    },
                }
            })
            .collect();

        if total_errors > MAX_VIOLATIONS {
            violations.push(Violation {
                rule: RULE_JSON_SCHEMA_VIOLATION.to_string(),
                severity: Severity::Medium,
                message: format!(
                    "... and {} more validation errors (showing first {} of {})",
                    total_errors - MAX_VIOLATIONS,
                    MAX_VIOLATIONS,
                    total_errors
                ),
                location: None,
            });
        }

        log::warn!(
            "JSON Schema validation failed with {} errors (schema: '{}')",
            total_errors,
            self.schema_path.display()
        );

        Ok(GuardrailResult::without_quality_score(
            false,
            violations,
            vec![],
        ))
    }

    fn name(&self) -> &str {
        "JsonSchemaGuardrail"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn create_guardrail(schema_json: &str) -> (tempfile::NamedTempFile, JsonSchemaGuardrail) {
        let mut temp = tempfile::Builder::new().suffix(".json").tempfile().unwrap();
        temp.write_all(schema_json.as_bytes()).unwrap();
        temp.flush().unwrap();
        let guardrail = JsonSchemaGuardrail::new(temp.path().to_path_buf()).unwrap();
        (temp, guardrail)
    }

    const SIMPLE_SCHEMA: &str = r#"{
        "type": "object",
        "properties": {
            "name": { "type": "string" },
            "age": { "type": "integer" }
        },
        "required": ["name"]
    }"#;

    #[tokio::test]
    async fn test_valid_json_matching_schema() {
        let (_tmp, guardrail) = create_guardrail(SIMPLE_SCHEMA);
        let result = guardrail
            .validate(r#"{"name": "Alice", "age": 30}"#)
            .await
            .unwrap();
        assert!(result.passed);
        assert!(result.violations.is_empty());
    }

    #[tokio::test]
    async fn test_valid_json_not_matching_schema() {
        let (_tmp, guardrail) = create_guardrail(SIMPLE_SCHEMA);
        let result = guardrail.validate(r#"{"name": 123}"#).await.unwrap();
        assert!(!result.passed);
        assert!(result
            .violations
            .iter()
            .any(|v| v.rule == "JSON_SCHEMA_VIOLATION"));
    }

    #[tokio::test]
    async fn test_invalid_json() {
        let (_tmp, guardrail) = create_guardrail(SIMPLE_SCHEMA);
        let result = guardrail.validate("not json at all").await.unwrap();
        assert!(!result.passed);
        assert!(result
            .violations
            .iter()
            .any(|v| v.rule == "JSON_PARSE_ERROR"));
    }

    #[tokio::test]
    async fn test_empty_content() {
        let (_tmp, guardrail) = create_guardrail(SIMPLE_SCHEMA);
        let result = guardrail.validate("").await.unwrap();
        assert!(!result.passed);
        assert!(result
            .violations
            .iter()
            .any(|v| v.rule == "JSON_PARSE_ERROR"));
        assert!(result.violations[0].message.contains("Content is empty"));
    }

    #[tokio::test]
    async fn test_wrong_type_array_vs_object() {
        let (_tmp, guardrail) = create_guardrail(SIMPLE_SCHEMA);
        let result = guardrail.validate(r#"[1, 2, 3]"#).await.unwrap();
        assert!(!result.passed);
        assert!(result
            .violations
            .iter()
            .any(|v| v.rule == "JSON_SCHEMA_VIOLATION"));
    }

    #[tokio::test]
    async fn test_missing_required_field() {
        let (_tmp, guardrail) = create_guardrail(SIMPLE_SCHEMA);
        let result = guardrail.validate(r#"{"age": 30}"#).await.unwrap();
        assert!(!result.passed);
        assert!(result
            .violations
            .iter()
            .any(|v| v.rule == "JSON_SCHEMA_VIOLATION"));
    }

    #[tokio::test]
    async fn test_violation_has_location() {
        let (_tmp, guardrail) = create_guardrail(SIMPLE_SCHEMA);
        let result = guardrail.validate(r#"{"name": 123}"#).await.unwrap();
        assert!(!result.passed);
        let violation = result
            .violations
            .iter()
            .find(|v| v.rule == "JSON_SCHEMA_VIOLATION")
            .unwrap();
        assert!(violation.location.is_some());
    }

    #[test]
    fn test_schema_file_not_found() {
        let result = JsonSchemaGuardrail::new(PathBuf::from("/nonexistent/schema.json"));
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Failed to read schema file"));
    }

    #[test]
    fn test_schema_file_invalid_json() {
        let mut temp = tempfile::Builder::new().suffix(".json").tempfile().unwrap();
        temp.write_all(b"not valid json {{{").unwrap();
        temp.flush().unwrap();
        let result = JsonSchemaGuardrail::new(temp.path().to_path_buf());
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("not valid JSON"));
    }

    #[test]
    fn test_schema_file_invalid_schema() {
        let mut temp = tempfile::Builder::new().suffix(".json").tempfile().unwrap();
        temp.write_all(br#"{"type": "invalid_type"}"#).unwrap();
        temp.flush().unwrap();
        let result = JsonSchemaGuardrail::new(temp.path().to_path_buf());
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("not a valid JSON Schema"));
    }

    #[test]
    fn test_name() {
        let (_tmp, guardrail) = create_guardrail(SIMPLE_SCHEMA);
        assert_eq!(guardrail.name(), "JsonSchemaGuardrail");
    }

    #[tokio::test]
    async fn test_nested_object_and_array_schema() {
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
                        }
                    }
                }
            }
        }"#;
        let (_tmp, guardrail) = create_guardrail(schema);

        // Valid
        let valid = r#"{"requirements": [{"id": "REQ_001", "summary": "Do something", "priority": "high"}]}"#;
        let result = guardrail.validate(valid).await.unwrap();
        assert!(result.passed);

        // Invalid: bad pattern for id
        let invalid = r#"{"requirements": [{"id": "BAD_ID", "summary": "Do something", "priority": "high"}]}"#;
        let result = guardrail.validate(invalid).await.unwrap();
        assert!(!result.passed);

        // Invalid: wrong enum value for priority
        let invalid2 = r#"{"requirements": [{"id": "REQ_001", "summary": "Do something", "priority": "critical"}]}"#;
        let result = guardrail.validate(invalid2).await.unwrap();
        assert!(!result.passed);
    }
}
