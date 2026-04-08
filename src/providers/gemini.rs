use crate::{
    error::CliError,
    models::{
        GeminiContent, GeminiGenerationConfig, GeminiPart, GeminiRequest, GeminiResponse,
        ResponseFormat,
    },
    provider::{InvokeParams, LlmProvider},
};
use async_trait::async_trait;
use reqwest::Client;

use super::logging::{handle_error_response, log_request, log_response};

/// Google Gemini provider via Vertex AI
pub struct GeminiProvider {
    client: Client,
    api_url: String,
}

impl GeminiProvider {
    pub fn new(api_url: String) -> Self {
        Self {
            client: Client::new(),
            api_url,
        }
    }
}

#[async_trait]
impl LlmProvider for GeminiProvider {
    async fn invoke(&self, params: InvokeParams<'_>) -> Result<String, CliError> {
        // Map ResponseFormat to Gemini's generationConfig fields
        let (response_mime_type, response_schema) = match params.response_format {
            Some(ResponseFormat::JsonObject) => (Some("application/json".to_string()), None),
            Some(ResponseFormat::JsonSchema { json_schema }) => (
                Some("application/json".to_string()),
                Some(json_schema.schema.clone()),
            ),
            Some(ResponseFormat::Text) | None => (None, None),
        };

        let system_instruction = if params.system_prompt.is_empty() {
            None
        } else {
            Some(GeminiContent {
                role: None,
                parts: vec![GeminiPart {
                    text: params.system_prompt.to_string(),
                }],
            })
        };

        let request = GeminiRequest {
            system_instruction,
            contents: vec![GeminiContent {
                role: Some("user".to_string()),
                parts: vec![GeminiPart {
                    text: params.user_prompt.to_string(),
                }],
            }],
            generation_config: Some(GeminiGenerationConfig {
                temperature: Some(params.temperature),
                max_output_tokens: params.max_tokens,
                seed: params.seed,
                response_mime_type,
                response_schema,
            }),
        };

        log_request(&request);

        let mut req = self
            .client
            .post(&self.api_url)
            .json(&request)
            .timeout(std::time::Duration::from_secs(params.timeout_secs));

        // Vertex AI uses OAuth2 Bearer token authentication
        if let Some(token) = params.api_key {
            req = req.header("Authorization", format!("Bearer {token}"));
            log::debug!("Authorization header: Bearer [REDACTED]");
        }

        let response = req.send().await?;

        if !response.status().is_success() {
            return Err(handle_error_response(response).await);
        }

        let response_text = response.text().await?;
        log_response(&response_text);

        let gemini_response: GeminiResponse = serde_json::from_str(&response_text)
            .map_err(|e| CliError::InvalidResponse(format!("Failed to parse response: {e}")))?;

        // Extract text from all parts of the first candidate
        let text = gemini_response
            .candidates
            .first()
            .map(|candidate| {
                candidate
                    .content
                    .parts
                    .iter()
                    .map(|part| part.text.as_str())
                    .collect::<Vec<_>>()
                    .join("")
            })
            .filter(|s| !s.is_empty())
            .ok_or_else(|| {
                CliError::InvalidResponse(
                    "No text content in Gemini response candidates".to_string(),
                )
            })?;

        Ok(text)
    }

    fn name(&self) -> &str {
        "Gemini"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::GeminiRequest;

    #[test]
    fn test_gemini_provider_new() {
        let provider = GeminiProvider::new("https://us-central1-aiplatform.googleapis.com/v1/projects/my-project/locations/us-central1/publishers/google/models/gemini-pro:generateContent".to_string());
        assert_eq!(provider.name(), "Gemini");
    }

    #[test]
    fn test_request_system_instruction_separate_from_contents() {
        let request = GeminiRequest {
            system_instruction: Some(GeminiContent {
                role: None,
                parts: vec![GeminiPart {
                    text: "You are helpful".to_string(),
                }],
            }),
            contents: vec![GeminiContent {
                role: Some("user".to_string()),
                parts: vec![GeminiPart {
                    text: "Hello".to_string(),
                }],
            }],
            generation_config: None,
        };

        let json = serde_json::to_value(&request).unwrap();

        // systemInstruction must be a top-level field, not in contents
        let contents = json["contents"].as_array().unwrap();
        assert_eq!(contents.len(), 1);
        assert_eq!(contents[0]["role"].as_str().unwrap(), "user");

        let sys = &json["systemInstruction"];
        assert!(sys["role"].is_null());
        assert_eq!(sys["parts"][0]["text"].as_str().unwrap(), "You are helpful");
    }

    #[test]
    fn test_generation_config_json_schema() {
        let schema = serde_json::json!({
            "type": "object",
            "properties": {
                "name": { "type": "string" }
            },
            "required": ["name"]
        });

        let request = GeminiRequest {
            system_instruction: None,
            contents: vec![GeminiContent {
                role: Some("user".to_string()),
                parts: vec![GeminiPart {
                    text: "Hello".to_string(),
                }],
            }],
            generation_config: Some(GeminiGenerationConfig {
                temperature: Some(0.7),
                max_output_tokens: Some(1000),
                seed: None,
                response_mime_type: Some("application/json".to_string()),
                response_schema: Some(schema.clone()),
            }),
        };

        let json = serde_json::to_value(&request).unwrap();
        let gen_config = &json["generationConfig"];
        assert_eq!(
            gen_config["responseMimeType"].as_str().unwrap(),
            "application/json"
        );
        assert_eq!(gen_config["responseSchema"], schema);
        assert_eq!(gen_config["maxOutputTokens"].as_u64().unwrap(), 1000);
    }

    #[test]
    fn test_no_system_instruction_when_empty() {
        let request = GeminiRequest {
            system_instruction: None,
            contents: vec![GeminiContent {
                role: Some("user".to_string()),
                parts: vec![GeminiPart {
                    text: "Hello".to_string(),
                }],
            }],
            generation_config: None,
        };

        let json = serde_json::to_value(&request).unwrap();
        assert!(json.get("systemInstruction").is_none());
        assert!(json.get("generationConfig").is_none());
    }

    #[test]
    fn test_gemini_provider_supports_streaming() {
        let provider = GeminiProvider::new("https://example.com".to_string());
        assert!(!provider.supports_streaming());
    }
}
