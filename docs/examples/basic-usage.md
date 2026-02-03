---
layout: default
title: Basic Usage
parent: Examples
nav_order: 1
---

# Basic Usage Examples

Simple examples to get started.

## CLI Examples

### Hello World

```bash
fortified-llm-client \
  --api-url http://localhost:11434/v1/chat/completions \
  --model llama3 \
  --user-text "Hello, world!"
```

### With System Prompt

```bash
fortified-llm-client \
  --api-url http://localhost:11434/v1/chat/completions \
  --model llama3 \
  --system-text "You are a helpful Rust expert" \
  --user-text "Explain ownership"
```

### Using Config File

`config.toml`:
```toml
api_url = "http://localhost:11434/v1/chat/completions"
model = "llama3"
temperature = 0.7
```

```bash
fortified-llm-client -c config.toml --user-text "Your question"
```

## Library Examples

### Basic Evaluation

```rust
use fortified_llm_client::{evaluate, EvaluationConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = EvaluationConfig {
        api_url: "http://localhost:11434/v1/chat/completions".to_string(),
        model: "llama3".to_string(),
        user_prompt: "Explain Rust ownership".to_string(),
        ..Default::default()
    };

    let result = evaluate(config).await?;
    println!("Response: {}", result.content);
    println!("Tokens: {}", result.metadata.tokens_estimated);

    Ok(())
}
```

### With Error Handling

```rust
use fortified_llm_client::{evaluate, EvaluationConfig, FortifiedError};

match evaluate(config).await {
    Ok(result) => println!("Success: {}", result.content),
    Err(FortifiedError::ApiError { message, .. }) => eprintln!("API error: {}", message),
    Err(e) => eprintln!("Error: {:?}", e),
}
```

### With Custom Parameters

```rust
let config = EvaluationConfig {
    api_url: "http://localhost:11434/v1/chat/completions".to_string(),
    model: "llama3".to_string(),
    system_prompt: Some("You are a creative writer.".to_string()),
    user_prompt: "Write a haiku about Rust".to_string(),
    temperature: Some(1.2),
    max_tokens: Some(100),
    seed: Some(42),
    ..Default::default()
};
```
