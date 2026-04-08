---
layout: default
title: Multi-Provider Examples
parent: Examples
nav_order: 3
---

# Multi-Provider Examples

Switch between OpenAI, Ollama, and Anthropic seamlessly.

## Ollama (Local)

```bash
# Ensure ollama is running
ollama serve

# Pull model
ollama pull llama3

# Use CLI
fortified-llm-client \
  --api-url http://localhost:11434/v1/chat/completions \
  --model llama3 \
  --user-text "Hello"
```

## OpenAI (Cloud)

```bash
export OPENAI_API_KEY=sk-...

fortified-llm-client \
  --api-url https://api.openai.com/v1/chat/completions \
  --model gpt-4 \
  --api-key-name OPENAI_API_KEY \
  --user-text "Hello"
```

## Anthropic (Direct API)

```bash
export ANTHROPIC_API_KEY=sk-ant-...

fortified-llm-client \
  --api-url https://api.anthropic.com/v1/messages \
  --model claude-sonnet-4-6 \
  --api-key-name ANTHROPIC_API_KEY \
  --system-text "You are a helpful assistant." \
  --user-text "Hello" \
  --max-tokens 1000
```

## Anthropic (Vertex AI)

```bash
fortified-llm-client \
  --api-url "https://global-aiplatform.googleapis.com/v1/projects/MY_PROJECT/locations/global/publishers/anthropic/models/claude-sonnet-4-6:streamRawPredict" \
  --model claude-sonnet-4-6 \
  --api-key "$(gcloud auth print-access-token)" \
  --system-text "You are a helpful assistant." \
  --user-text "Hello" \
  --max-tokens 1000
```

## Provider-Specific Config Files

`ollama.toml`:
```toml
api_url = "http://localhost:11434/v1/chat/completions"
model = "llama3"
temperature = 0.7
```

`openai.toml`:
```toml
api_url = "https://api.openai.com/v1/chat/completions"
model = "gpt-4"
api_key_name = "OPENAI_API_KEY"
temperature = 0.7
```

`anthropic.toml`:
```toml
api_url = "https://api.anthropic.com/v1/messages"
model = "claude-sonnet-4-6"
api_key_name = "ANTHROPIC_API_KEY"
temperature = 0.7
max_tokens = 1000
```

`vertex.toml`:
```toml
api_url = "https://global-aiplatform.googleapis.com/v1/projects/MY_PROJECT/locations/global/publishers/anthropic/models/claude-sonnet-4-6:streamRawPredict"
provider = "anthropic"
temperature = 0.7
max_tokens = 1000
```

`vertex-proxy.toml` (when Vertex AI is behind a proxy/gateway):
```toml
api_url = "https://my-proxy.example.com/claude"
provider = "anthropic-vertex"
temperature = 0.7
max_tokens = 1000
```

Usage:
```bash
# Use Ollama
fortified-llm-client -c ollama.toml --user-text "prompt"

# Use OpenAI
fortified-llm-client -c openai.toml --user-text "prompt"

# Use Anthropic
fortified-llm-client -c anthropic.toml --user-text "prompt"

# Use Vertex AI (pass OAuth token via CLI)
fortified-llm-client -c vertex.toml --api-key "$(gcloud auth print-access-token)" --user-text "prompt"
```

## Library Multi-Provider

```rust
async fn call_llm(provider: &str, prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
    let (api_url, model, api_key_name) = match provider {
        "ollama" => ("http://localhost:11434/v1/chat/completions", "llama3", None),
        "openai" => ("https://api.openai.com/v1/chat/completions", "gpt-4", Some("OPENAI_API_KEY")),
        "anthropic" => ("https://api.anthropic.com/v1/messages", "claude-sonnet-4-6", Some("ANTHROPIC_API_KEY")),
        _ => return Err("Unknown provider".into()),
    };

    let config = EvaluationConfig {
        api_url: api_url.to_string(),
        model: model.to_string(),
        user_prompt: prompt.to_string(),
        api_key_name: api_key_name.map(String::from),
        ..Default::default()
    };

    let result = evaluate(config).await?;
    Ok(result.content)
}
```
