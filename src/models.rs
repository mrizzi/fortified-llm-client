use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;

// Response format for OpenAI-compatible APIs
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ResponseFormat {
    #[serde(rename = "text")]
    Text,

    #[serde(rename = "json_object")]
    JsonObject,

    #[serde(rename = "json_schema")]
    JsonSchema { json_schema: JsonSchemaDefinition },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonSchemaDefinition {
    pub name: String,
    pub schema: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strict: Option<bool>,
}

impl ResponseFormat {
    pub fn text() -> Self {
        Self::Text
    }

    pub fn json() -> Self {
        Self::JsonObject
    }

    pub fn json_schema(name: String, schema: Value, strict: bool) -> Self {
        Self::JsonSchema {
            json_schema: JsonSchemaDefinition {
                name,
                schema,
                strict: Some(strict),
            },
        }
    }
}

impl fmt::Display for ResponseFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Text => write!(f, "text"),
            Self::JsonObject => write!(f, "json-object"),
            Self::JsonSchema { json_schema } => {
                write!(
                    f,
                    "json-schema (name: {}, strict: {})",
                    json_schema.name,
                    json_schema.strict.unwrap_or(false)
                )
            }
        }
    }
}

// OpenAI format (standard)
#[derive(Serialize)]
pub struct OpenAIRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub temperature: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<ResponseFormat>,
}

#[derive(Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Deserialize)]
pub struct OpenAIResponse {
    pub choices: Vec<Choice>,
}

#[derive(Deserialize)]
pub struct Choice {
    pub message: Message,
}

// /api/generate format (used by local servers)
#[derive(Serialize)]
pub struct OllamaRequest {
    pub model: String,
    pub system: String,
    pub prompt: String,
    pub stream: bool,
    pub options: OllamaOptions,
}

#[derive(Serialize)]
pub struct OllamaOptions {
    pub temperature: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<u64>,
}

#[derive(Deserialize)]
pub struct OllamaResponse {
    pub response: String,
}
