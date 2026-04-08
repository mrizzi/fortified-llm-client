use crate::{
    error::CliError,
    models::{AnthropicMessage, AnthropicRequest, AnthropicResponse},
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
}

#[async_trait]
impl LlmProvider for AnthropicProvider {
    async fn invoke(&self, params: InvokeParams<'_>) -> Result<String, CliError> {
        let max_tokens = params.max_tokens.unwrap_or(DEFAULT_MAX_TOKENS);

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
