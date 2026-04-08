use crate::{
    error::CliError,
    models::{
        AnthropicMessage, AnthropicOutputConfig, AnthropicOutputFormat, AnthropicRequest,
        AnthropicResponse, ResponseFormat,
    },
    provider::{InvokeParams, LlmProvider},
};
use async_trait::async_trait;
use reqwest::Client;

use super::logging::{handle_error_response, log_request, log_response};

/// Default max_tokens when not specified by the user (Anthropic requires this field)
const DEFAULT_MAX_TOKENS: u32 = 4096;

/// Anthropic API mode: direct API or Vertex AI
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnthropicMode {
    /// Direct Anthropic API (api.anthropic.com) — uses x-api-key header
    Direct,
    /// Google Vertex AI — uses Authorization: Bearer, anthropic_version in body, no model in body
    Vertex,
}

/// Anthropic provider supporting both direct API and Vertex AI
pub struct AnthropicProvider {
    client: Client,
    api_url: String,
    mode: AnthropicMode,
}

impl AnthropicProvider {
    pub fn new(api_url: String) -> Self {
        let mode = if api_url.contains("aiplatform.googleapis.com") {
            AnthropicMode::Vertex
        } else {
            AnthropicMode::Direct
        };

        Self {
            client: Client::new(),
            api_url,
            mode,
        }
    }

    pub fn new_vertex(api_url: String) -> Self {
        Self {
            client: Client::new(),
            api_url,
            mode: AnthropicMode::Vertex,
        }
    }
}

#[async_trait]
impl LlmProvider for AnthropicProvider {
    async fn invoke(&self, params: InvokeParams<'_>) -> Result<String, CliError> {
        let max_tokens = params.max_tokens.unwrap_or(DEFAULT_MAX_TOKENS);

        let output_config = params.response_format.and_then(|fmt| match fmt {
            ResponseFormat::JsonSchema { json_schema } => Some(AnthropicOutputConfig {
                format: AnthropicOutputFormat::JsonSchema {
                    schema: json_schema.schema.clone(),
                },
            }),
            ResponseFormat::JsonObject => {
                log::warn!(
                    "Anthropic provider does not support 'json-object' response format; ignoring"
                );
                None
            }
            ResponseFormat::Text => None,
        });

        let request = AnthropicRequest {
            model: match self.mode {
                AnthropicMode::Direct => Some(params.model.to_string()),
                AnthropicMode::Vertex => None,
            },
            max_tokens,
            messages: vec![AnthropicMessage {
                role: "user".to_string(),
                content: params.user_prompt.to_string(),
            }],
            system: if params.system_prompt.is_empty() {
                None
            } else {
                Some(params.system_prompt.to_string())
            },
            temperature: params.temperature,
            anthropic_version: match self.mode {
                AnthropicMode::Vertex => Some("vertex-2023-10-16".to_string()),
                AnthropicMode::Direct => None,
            },
            output_config,
        };

        log_request(&request);

        let mut req = self
            .client
            .post(&self.api_url)
            .json(&request)
            .timeout(std::time::Duration::from_secs(params.timeout_secs));

        match self.mode {
            AnthropicMode::Direct => {
                if let Some(key) = params.api_key {
                    req = req.header("x-api-key", key);
                    log::debug!("x-api-key header: [REDACTED]");
                }
                req = req.header("anthropic-version", "2023-06-01");
            }
            AnthropicMode::Vertex => {
                if let Some(token) = params.api_key {
                    req = req.header("Authorization", format!("Bearer {token}"));
                    log::debug!("Authorization header: Bearer [REDACTED]");
                }
            }
        }

        let response = req.send().await?;

        if !response.status().is_success() {
            return Err(handle_error_response(response).await);
        }

        let response_text = response.text().await?;
        log_response(&response_text);

        let anthropic_response: AnthropicResponse = serde_json::from_str(&response_text)
            .map_err(|e| CliError::InvalidResponse(format!("Failed to parse response: {e}")))?;

        anthropic_response
            .content
            .iter()
            .find(|block| block.block_type == "text")
            .and_then(|block| block.text.clone())
            .ok_or_else(|| CliError::InvalidResponse("No text content in response".to_string()))
    }

    fn name(&self) -> &str {
        "Anthropic"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::AnthropicRequest;

    #[test]
    fn test_request_has_no_system_role_in_messages() {
        let request = AnthropicRequest {
            model: Some("claude-sonnet-4-6".to_string()),
            max_tokens: 4096,
            messages: vec![AnthropicMessage {
                role: "user".to_string(),
                content: "Hello".to_string(),
            }],
            system: Some("You are helpful".to_string()),
            temperature: 0.7,
            anthropic_version: None,
            output_config: None,
        };

        let json = serde_json::to_value(&request).unwrap();

        // System prompt must be a top-level field, never a message role
        let messages = json["messages"].as_array().unwrap();
        for msg in messages {
            assert_ne!(
                msg["role"].as_str().unwrap(),
                "system",
                "system prompt must not appear as a message role in Anthropic API"
            );
        }
        assert_eq!(json["system"].as_str().unwrap(), "You are helpful");
    }

    #[test]
    fn test_vertex_request_omits_model_and_includes_version() {
        let request = AnthropicRequest {
            model: None,
            max_tokens: 4096,
            messages: vec![AnthropicMessage {
                role: "user".to_string(),
                content: "Hello".to_string(),
            }],
            system: Some("You are helpful".to_string()),
            temperature: 0.7,
            anthropic_version: Some("vertex-2023-10-16".to_string()),
            output_config: None,
        };

        let json = serde_json::to_value(&request).unwrap();
        assert!(
            json.get("model").is_none(),
            "Vertex requests must omit model"
        );
        assert_eq!(
            json["anthropic_version"].as_str().unwrap(),
            "vertex-2023-10-16"
        );
    }

    #[test]
    fn test_output_config_json_schema() {
        let schema = serde_json::json!({
            "type": "object",
            "properties": {
                "name": { "type": "string" },
                "age": { "type": "number" }
            },
            "required": ["name", "age"]
        });

        let request = AnthropicRequest {
            model: Some("claude-sonnet-4-6".to_string()),
            max_tokens: 4096,
            messages: vec![AnthropicMessage {
                role: "user".to_string(),
                content: "Hello".to_string(),
            }],
            system: None,
            temperature: 0.7,
            anthropic_version: None,
            output_config: Some(AnthropicOutputConfig {
                format: AnthropicOutputFormat::JsonSchema {
                    schema: schema.clone(),
                },
            }),
        };

        let json = serde_json::to_value(&request).unwrap();
        let output_config = &json["output_config"];
        assert_eq!(output_config["format"]["type"], "json_schema");
        assert_eq!(output_config["format"]["schema"], schema);
    }

    #[test]
    fn test_anthropic_provider_direct_mode() {
        let provider = AnthropicProvider::new("https://api.anthropic.com/v1/messages".to_string());
        assert_eq!(provider.name(), "Anthropic");
        assert_eq!(provider.mode, AnthropicMode::Direct);
    }

    #[test]
    fn test_anthropic_provider_vertex_mode() {
        let provider = AnthropicProvider::new(
            "https://global-aiplatform.googleapis.com/v1/projects/my-project/locations/global/publishers/anthropic/models/claude-sonnet-4-6:streamRawPredict".to_string(),
        );
        assert_eq!(provider.name(), "Anthropic");
        assert_eq!(provider.mode, AnthropicMode::Vertex);
    }

    #[test]
    fn test_anthropic_provider_supports_streaming() {
        let provider = AnthropicProvider::new("https://api.anthropic.com/v1/messages".to_string());
        assert!(!provider.supports_streaming());
    }
}
