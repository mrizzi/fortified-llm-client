use crate::models::ResponseFormat;
use serde::Serialize;

#[derive(Serialize)]
pub struct CliOutput {
    pub status: String, // "success" or "error"
    pub response: Option<serde_json::Value>,
    pub metadata: Metadata,
    pub error: Option<ErrorInfo>,
}

#[derive(Serialize)]
pub struct Metadata {
    // Execution results
    pub model: String,
    pub tokens_estimated: usize,
    pub latency_ms: u64,
    pub timestamp: String, // ISO 8601

    // Input configuration (for reproducibility)
    pub api_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    pub temperature: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<u64>,
    pub timeout_secs: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_limit: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<String>,
    pub validate_tokens: bool,

    // Input sources (prompts: mutually exclusive text/file for each type)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_prompt_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_prompt_file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_prompt_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_prompt_file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pdf_input: Option<String>,

    // Guardrails
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_guardrails_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_guardrails_enabled: Option<bool>,
}

#[derive(Serialize)]
pub struct ErrorInfo {
    pub code: String,
    pub message: String,
}

impl CliOutput {
    /// Create a success output
    pub fn success(
        response: String,
        metadata: Metadata,
        response_format: Option<&ResponseFormat>,
    ) -> Self {
        // Parse as JSON if response_format indicates JSON
        let parsed_response = if let Some(format) = response_format {
            match format {
                ResponseFormat::JsonObject | ResponseFormat::JsonSchema { .. } => {
                    // Attempt to parse as JSON
                    match serde_json::from_str::<serde_json::Value>(&response) {
                        Ok(json_value) => json_value,
                        Err(e) => {
                            log::warn!(
                                "Failed to parse response as JSON despite response_format={format:?}: {e}. \
                                Falling back to string representation."
                            );
                            serde_json::Value::String(response)
                        }
                    }
                }
                ResponseFormat::Text => serde_json::Value::String(response),
            }
        } else {
            // No response_format specified, treat as text
            serde_json::Value::String(response)
        };

        Self {
            status: "success".to_string(),
            response: Some(parsed_response),
            metadata,
            error: None,
        }
    }

    /// Create an error output
    pub fn error(code: String, message: String, metadata: Metadata) -> Self {
        Self {
            status: "error".to_string(),
            response: None,
            metadata,
            error: Some(ErrorInfo { code, message }),
        }
    }
}
