---
layout: default
title: Providers
parent: Architecture
nav_order: 3
---

# Provider System

Multi-provider support with automatic detection and unified interface.

## Supported Providers

1. **OpenAI** - Official OpenAI API
2. **Ollama** - Local models with OpenAI-compatible API
3. **OpenAI-compatible** - Any service using `/v1/chat/completions` endpoint
4. **Anthropic** - Claude models via direct API or Google Vertex AI
5. **Gemini** - Google Gemini models via Vertex AI

## Provider Detection

**Location**: `src/providers/detection.rs`

### Auto-Detection Logic

Analyzes API URL to infer provider:

```rust
pub fn detect_provider_type(url: &str) -> ProviderType {
    // Path-based detection (highest priority)
    if url.contains("/v1/messages") { return ProviderType::Anthropic; }
    if url.contains("/api/generate") { return ProviderType::Ollama; }
    if url.contains("/v1/chat/completions") { return ProviderType::OpenAI; }

    // Publisher-based detection for Vertex AI (multi-provider on same host)
    if url.contains("aiplatform.googleapis.com") {
        if url.contains("/publishers/google/") { return ProviderType::Gemini; }
        return ProviderType::Anthropic;  // Fallback for backward compat
    }

    // Host-based detection
    if url.contains("anthropic.com") { return ProviderType::Anthropic; }

    // Port-based fallback
    if url.contains("localhost:11434") || url.contains("127.0.0.1:11434") {
        return ProviderType::Ollama;
    }

    ProviderType::OpenAI  // Default fallback
}
```

**Patterns matched**:
- `/v1/messages` → Anthropic (path takes highest priority)
- `/api/generate` → Ollama
- `/v1/chat/completions` → OpenAI
- `aiplatform.googleapis.com` + `/publishers/google/` → Gemini
- `aiplatform.googleapis.com` + other → Anthropic (backward compat)
- `anthropic.com` → Anthropic
- `localhost:11434` → Ollama
- Everything else → OpenAI (fallback)

### Explicit Override

Force provider via CLI or config:

**CLI**:
```bash
--provider openai
--provider ollama
--provider anthropic
--provider anthropic-vertex
--provider gemini
```

**Config**:
```toml
provider = "openai"     # or "ollama", "anthropic", "anthropic-vertex", "gemini"
```

**Library**:
```rust
use fortified_llm_client::Provider;

let config = EvaluationConfig {
    provider: Some(Provider::OpenAI),    // or Ollama, Anthropic, AnthropicVertex, Gemini
    // ...
};
```

## LlmProvider Trait

**Location**: `src/providers/provider.rs`

Unified interface for all providers:

```rust
#[async_trait]
pub trait LlmProvider: Send + Sync {
    async fn invoke(&self, request: LlmRequest) -> Result<LlmResponse, FortifiedError>;
}
```

### LlmRequest

```rust
pub struct LlmRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub seed: Option<u64>,
    pub response_format: Option<ResponseFormat>,
    // ...
}
```

### LlmResponse

```rust
pub struct LlmResponse {
    pub content: String,
    pub model: String,
    pub finish_reason: Option<String>,
}
```

## OpenAI Provider

**Location**: `src/providers/openai.rs`

### Implementation

```rust
pub struct OpenAIProvider {
    api_url: String,
    api_key: Option<String>,
    timeout: Duration,
}

#[async_trait]
impl LlmProvider for OpenAIProvider {
    async fn invoke(&self, request: LlmRequest) -> Result<LlmResponse, FortifiedError> {
        // Build request body
        // Make HTTP POST to api_url
        // Parse JSON response
        // Extract content from choices[0].message.content
    }
}
```

### Request Format

```json
{
  "model": "gpt-4",
  "messages": [
    {"role": "system", "content": "You are a helpful assistant."},
    {"role": "user", "content": "Explain Rust ownership"}
  ],
  "temperature": 0.7,
  "max_tokens": 1000,
  "seed": 42
}
```

### Response Format

```json
{
  "choices": [
    {
      "message": {
        "role": "assistant",
        "content": "Rust ownership ensures..."
      },
      "finish_reason": "stop"
    }
  ],
  "model": "gpt-4",
  "usage": {
    "prompt_tokens": 20,
    "completion_tokens": 100,
    "total_tokens": 120
  }
}
```

## Ollama Provider

**Location**: `src/providers/ollama.rs`

### Implementation

```rust
pub struct OllamaProvider {
    api_url: String,
    timeout: Duration,
}

#[async_trait]
impl LlmProvider for OllamaProvider {
    async fn invoke(&self, request: LlmRequest) -> Result<LlmResponse, FortifiedError> {
        // Same format as OpenAI (Ollama is compatible)
        // No API key required
    }
}
```

### Differences from OpenAI

1. **No API key required** - Ollama runs locally
2. **Same request/response format** - OpenAI-compatible
3. **Local models** - Models must be pulled first (`ollama pull llama3`)

## Anthropic Provider

**Location**: `src/providers/anthropic.rs`

### Implementation

Supports both direct Anthropic API and Google Vertex AI through a single provider with an internal mode:

```rust
pub enum AnthropicMode {
    Direct,  // api.anthropic.com — uses x-api-key header
    Vertex,  // Vertex AI — uses Authorization: Bearer, anthropic_version in body
}

pub struct AnthropicProvider {
    client: Client,
    api_url: String,
    mode: AnthropicMode,  // Auto-detected from URL
}
```

Mode is auto-detected: URLs containing `aiplatform.googleapis.com` use Vertex mode, everything else uses Direct mode. Use `--provider anthropic-vertex` to force Vertex mode when the URL doesn't match (e.g., behind a proxy).

### Differences from OpenAI

1. **Different auth header** - Direct: `x-api-key`, Vertex: `Authorization: Bearer`
2. **System prompt is a top-level field** - Not part of the messages array
3. **`max_tokens` is required** - Defaults to 4096 when not specified
4. **Different response format** - Content blocks array instead of choices
5. **Vertex AI specifics** - Model omitted from body (in URL), `anthropic_version` in body
6. **Structured output** - Uses `output_config` instead of `response_format` (see below)

### Request Format (Direct API)

```json
{
  "model": "claude-sonnet-4-6",
  "max_tokens": 4096,
  "system": "You are a helpful assistant.",
  "messages": [
    {"role": "user", "content": "Explain Rust ownership"}
  ],
  "temperature": 0.7
}
```

### Request Format (Vertex AI)

```json
{
  "max_tokens": 4096,
  "anthropic_version": "vertex-2023-10-16",
  "system": "You are a helpful assistant.",
  "messages": [
    {"role": "user", "content": "Explain Rust ownership"}
  ],
  "temperature": 0.7
}
```

### Structured Output (JSON Schema)

When `response_format = "json-schema"` is configured, the Anthropic provider maps it to Anthropic's `output_config` parameter:

```json
{
  "model": "claude-sonnet-4-6",
  "max_tokens": 4096,
  "messages": [
    {"role": "user", "content": "Extract name and age from: John is 30"}
  ],
  "output_config": {
    "format": {
      "type": "json_schema",
      "schema": {
        "type": "object",
        "properties": {
          "name": {"type": "string"},
          "age": {"type": "number"}
        },
        "required": ["name", "age"]
      }
    }
  },
  "temperature": 0.7
}
```

{: .warning }
> **Vertex AI limitation**: Structured output (`output_config`) is not supported on Vertex AI. Only the direct Anthropic API supports it. When using `anthropic-vertex`, the `response_format` setting is ignored with a warning. See [Anthropic feature availability](https://platform.claude.com/docs/en/build-with-claude/overview) for the full platform support matrix.

{: .note }
> Anthropic does not support `json-object` mode (OpenAI-only). Only `json-schema` is supported for structured output on the direct API. Using `json-object` logs a warning and is ignored.

### Response Format

```json
{
  "content": [
    {
      "type": "text",
      "text": "Rust ownership ensures..."
    }
  ],
  "stop_reason": "end_turn"
}
```

## Gemini Provider

**Location**: `src/providers/gemini.rs`

### Implementation

Google Gemini via Vertex AI, using OAuth2 Bearer token authentication:

```rust
pub struct GeminiProvider {
    client: Client,
    api_url: String,
}
```

Authentication uses `Authorization: Bearer` with an OAuth2 token (obtained via `gcloud auth print-access-token`).

### Differences from OpenAI

1. **OAuth2 auth** - `Authorization: Bearer {token}` (not API key)
2. **System prompt is `systemInstruction`** - Top-level field with `parts`, not in messages
3. **`contents` instead of `messages`** - Each with `role` and `parts` array
4. **`generationConfig` for parameters** - `maxOutputTokens`, `responseMimeType`, `responseSchema`
5. **Model in URL** - Vertex AI embeds model in endpoint path

### Request Format

```json
{
  "systemInstruction": {
    "parts": [{"text": "You are a helpful assistant."}]
  },
  "contents": [
    {"role": "user", "parts": [{"text": "Explain Rust ownership"}]}
  ],
  "generationConfig": {
    "temperature": 0.7,
    "maxOutputTokens": 1000,
    "seed": 42
  }
}
```

### Structured Output (JSON Schema)

When `response_format = "json-schema"` is configured, the Gemini provider maps it to `responseMimeType` + `responseSchema`:

```json
{
  "contents": [
    {"role": "user", "parts": [{"text": "Extract name and age"}]}
  ],
  "generationConfig": {
    "temperature": 0.7,
    "maxOutputTokens": 1000,
    "responseMimeType": "application/json",
    "responseSchema": {
      "type": "object",
      "properties": {
        "name": {"type": "string"},
        "age": {"type": "number"}
      },
      "required": ["name", "age"]
    }
  }
}
```

`json-object` mode is also supported (sets `responseMimeType: "application/json"` without a schema).

### Response Format

```json
{
  "candidates": [
    {
      "content": {
        "parts": [{"text": "Rust ownership ensures..."}]
      }
    }
  ]
}
```

## Error Handling

### Common Errors

| Error | Provider | Cause |
|-------|----------|-------|
| 401 Unauthorized | OpenAI, Anthropic | Invalid/missing API key |
| 404 Not Found | Ollama | Model not pulled |
| 429 Rate Limit | OpenAI, Anthropic | Too many requests |
| Connection Refused | Ollama | Ollama not running |
| Timeout | All | Request took too long |

### Error Mapping

```rust
match status {
    401 => FortifiedError::ApiError {
        message: "Authentication failed".to_string(),
        status_code: Some(401),
    },
    404 => FortifiedError::ApiError {
        message: "Model not found".to_string(),
        status_code: Some(404),
    },
    // ...
}
```

## Adding New Providers

### Step 1: Implement LlmProvider Trait

Create `src/providers/my_provider.rs`:

```rust
use async_trait::async_trait;
use crate::providers::{LlmProvider, LlmRequest, LlmResponse};
use crate::FortifiedError;

pub struct MyProvider {
    api_url: String,
    api_key: Option<String>,
}

#[async_trait]
impl LlmProvider for MyProvider {
    async fn invoke(&self, request: LlmRequest) -> Result<LlmResponse, FortifiedError> {
        // Your implementation
    }
}
```

### Step 2: Update Provider Enum

In `src/lib.rs`:

```rust
pub enum ProviderType {
    OpenAI,
    Ollama,
    Anthropic,
    MyProvider,  // Add new variant
}
```

### Step 3: Update Detection Logic

In `src/providers/detection.rs`:

```rust
pub fn detect_provider(api_url: &str) -> Provider {
    if api_url.contains("myprovider.com") {
        Provider::MyProvider
    } else if api_url.contains("openai.com") {
        Provider::OpenAI
    }
    // ...
}
```

### Step 4: Update Client Factory

In `src/client.rs`:

```rust
match provider {
    Provider::OpenAI => Box::new(OpenAIProvider::new(/* ... */)),
    Provider::Ollama => Box::new(OllamaProvider::new(/* ... */)),
    Provider::MyProvider => Box::new(MyProvider::new(/* ... */)),
}
```

## Testing

### Mock Providers for Testing

Use `mockito` for HTTP mocking:

```rust
use mockito::{mock, server_url};

#[tokio::test]
async fn test_openai_provider() {
    let _m = mock("POST", "/v1/chat/completions")
        .with_status(200)
        .with_body(r#"{"choices":[{"message":{"content":"Test"}}]}"#)
        .create();

    let provider = OpenAIProvider::new(server_url(), None, Duration::from_secs(30));
    let request = LlmRequest { /* ... */ };
    let response = provider.invoke(request).await.unwrap();

    assert_eq!(response.content, "Test");
}
```

## See Also

- [Layers]({{ site.baseurl }}{% link architecture/layers.md %}) - Architecture overview
- [Evaluation Pipeline]({{ site.baseurl }}{% link architecture/evaluation-pipeline.md %}) - Where providers fit
- [User Guide]({{ site.baseurl }}{% link user-guide/cli-usage.md %}) - Using providers in practice
