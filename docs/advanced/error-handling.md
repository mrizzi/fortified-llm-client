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

## CLI Exit Codes

The CLI uses specific exit codes for different failure types:

| Exit Code | Error Code | Cause |
|-----------|------------|-------|
| 0 | - | Success |
| 1 | - | I/O error writing output |
| 2 | `CONTEXT_LIMIT_EXCEEDED` | Token count exceeds context window |
| 3 | `HTTP_ERROR` | Network/HTTP request failure |
| 4 | `INVALID_RESPONSE` | API response parsing failed |
| 5 | `FILE_NOT_FOUND` | File read failure (prompts, PDF, config, schema) |
| 6 | `INVALID_ARGUMENTS` | CLI argument or config validation failure |
| 7 | `AUTH_FAILED` | API authentication failure |
| 8 | `PDF_PROCESSING_FAILED` / `FILE_TOO_LARGE` | PDF extraction failure or file size limit |
| 9 | `INPUT_VALIDATION_FAILED` / `OUTPUT_VALIDATION_FAILED` | Guardrail rejection |

{: .note }
> Exit codes 2-8 come from infrastructure errors (`Err` path). Exit code 9 comes from guardrail rejections that return structured JSON output with full metadata.

## Common Errors

| Error | Cause | Recovery |
|-------|-------|----------|
| `ApiError` 401 | Invalid API key | Check environment variable |
| `ApiError` 429 | Rate limit | Retry with backoff |
| `ValidationError` | Guardrail failure | Adjust prompt or disable guardrails |
| `PdfError` | Docling missing | Install docling or skip PDF |
| `ConfigError` | Invalid TOML | Validate syntax |
