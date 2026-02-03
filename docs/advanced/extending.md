---
layout: default
title: Extending
parent: Advanced
nav_order: 4
---

# Extending the Codebase

Add new providers and guardrails.

## Adding a New Provider

### 1. Implement `LlmProvider` Trait

`src/providers/my_provider.rs`:
```rust
use async_trait::async_trait;
use crate::providers::{LlmProvider, LlmRequest, LlmResponse};

pub struct MyProvider {
    api_url: String,
}

#[async_trait]
impl LlmProvider for MyProvider {
    async fn invoke(&self, request: LlmRequest) -> Result<LlmResponse, FortifiedError> {
        // Implement API call
    }
}
```

### 2. Add to Provider Enum

`src/lib.rs`:
```rust
pub enum Provider {
    OpenAI,
    Ollama,
    MyProvider,
}
```

### 3. Update Detection

`src/providers/detection.rs`:
```rust
if api_url.contains("myprovider.com") {
    Provider::MyProvider
}
```

### 4. Update Factory

`src/client.rs`:
```rust
Provider::MyProvider => Box::new(MyProvider::new(api_url)),
```

## Adding a Custom Guardrail

### 1. Implement `GuardrailProvider` Trait

`src/guardrails/my_guardrail.rs`:
```rust
use async_trait::async_trait;
use crate::guardrails::GuardrailProvider;

pub struct MyGuardrail {
    // Config fields
}

#[async_trait]
impl GuardrailProvider for MyGuardrail {
    async fn validate(&self, input: &str) -> Result<(), FortifiedError> {
        // Validation logic
    }
}
```

### 2. Add Config Struct

```rust
#[derive(Serialize, Deserialize)]
pub struct MyGuardrailConfig {
    pub threshold: f32,
}
```

### 3. Update Config Parser

`src/guardrails/config.rs`:
```rust
match config_type {
    "my_guardrail" => {
        let config: MyGuardrailConfig = /* parse */;
        Box::new(MyGuardrail::new(config))
    }
    // ...
}
```

### 4. Use in Config File

```toml
[guardrails.input]
type = "my_guardrail"

[guardrails.input.my_guardrail]
threshold = 0.8
```
