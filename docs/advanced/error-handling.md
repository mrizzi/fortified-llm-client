---
layout: default
title: Error Handling
parent: Advanced
nav_order: 2
---

# Error Handling

Error types and recovery strategies.

## FortifiedError Variants

```rust
pub enum FortifiedError {
    ApiError { message: String, status_code: Option<u16> },
    ValidationError { message: String, validation_type: Option<String> },
    ConfigError { message: String },
    PdfError { message: String },
    InternalError { message: String },
}
```

## Error Handling Patterns

### Match on Specific Errors

```rust
match evaluate(config).await {
    Ok(result) => println!("{}", result.content),
    Err(FortifiedError::ApiError { message, status_code }) => {
        eprintln!("API error ({}): {}", status_code.unwrap_or(0), message);
        // Retry logic
    }
    Err(FortifiedError::ValidationError { message, .. }) => {
        eprintln!("Validation failed: {}", message);
        // Log and skip
    }
    Err(e) => eprintln!("Unexpected: {:?}", e),
}
```

### Graceful Degradation

```rust
// Try with guardrails, fallback to no guardrails
match evaluate_with_guardrails(config.clone(), "config.toml").await {
    Ok(result) => result,
    Err(FortifiedError::ConfigError { .. }) => {
        eprintln!("Guardrails config invalid, proceeding without");
        evaluate(config).await?
    }
    Err(e) => return Err(e),
}
```

## Common Errors

| Error | Cause | Recovery |
|-------|-------|----------|
| `ApiError` 401 | Invalid API key | Check environment variable |
| `ApiError` 429 | Rate limit | Retry with backoff |
| `ValidationError` | Guardrail failure | Adjust prompt or disable guardrails |
| `PdfError` | Docling missing | Install docling or skip PDF |
| `ConfigError` | Invalid TOML | Validate syntax |
