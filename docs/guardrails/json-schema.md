---
layout: default
title: JSON Schema Guardrails
parent: Guardrails
nav_order: 6
---

# JSON Schema Guardrails

Client-side validation of LLM responses against a JSON Schema.

## Overview

JSON Schema guardrails validate that LLM output conforms to a specific JSON Schema, providing **hard enforcement** of structured output contracts. Unlike `--response-format json-schema` (which is a hint to the LLM provider), this guardrail rejects non-conforming responses at the client level.

**Speed**: <10ms (no LLM calls)
**Cost**: Free (local validation)
**Works for**: Both input and output validation (primarily output)
**Schema support**: JSON Schema Draft 7 (full keyword support)

## When to Use

- Structured output enforcement in security-sensitive pipelines
- Validating LLM-extracted data matches expected schema
- Defense-in-depth alongside `--response-format json-schema`
- Automated pipelines where non-conforming output must be rejected

## Configuration

### Basic Configuration

```toml
[guardrails.output]
type = "json_schema"
schema_file = "schemas/requirements.json"
```

### Schema File Example

`schemas/requirements.json`:

```json
{
  "type": "object",
  "required": ["requirements"],
  "properties": {
    "requirements": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["id", "summary", "priority"],
        "properties": {
          "id": { "type": "string", "pattern": "^REQ_[0-9]+$" },
          "summary": { "type": "string", "maxLength": 200 },
          "priority": { "enum": ["high", "medium", "low"] }
        },
        "additionalProperties": false
      }
    }
  },
  "additionalProperties": false
}
```

### All Options

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `schema_file` | `PathBuf` | Yes | Path to a JSON Schema file (.json) |

### Supported JSON Schema Keywords

All JSON Schema Draft 7 keywords are supported, including:

- `type`, `properties`, `required`, `additionalProperties`
- `items` (for arrays), `minItems`, `maxItems`
- `enum`, `const`
- `pattern`, `minLength`, `maxLength`
- `minimum`, `maximum`, `exclusiveMinimum`, `exclusiveMaximum`
- `$ref`, `definitions`
- `oneOf`, `anyOf`, `allOf`, `not`
- `format` (e.g., `"email"`, `"uri"`)

## Relationship to `--response-format json-schema`

These two features are **complementary**, not redundant:

| Feature | Stage | Enforcement | Purpose |
|---------|-------|-------------|---------|
| `--response-format json-schema` | LLM invocation | Provider-side (best-effort) | Guides LLM to generate structured output |
| `json_schema` guardrail | Post-processing | Client-side (hard enforcement) | Rejects non-conforming responses |

For maximum reliability, use both:

```toml
# Guide the LLM to produce structured output
response_format = "json-schema"
response_format_schema = "schemas/requirements.json"

# Hard-enforce the schema on the actual response
[guardrails.output]
type = "json_schema"
schema_file = "schemas/requirements.json"
```

## Composite Guardrails

Combine JSON Schema validation with regex pattern checks:

```toml
[guardrails.output]
type = "composite"
execution = "sequential"
aggregation = "all_must_pass"

# Fast regex check first (reject HTML, markdown, instruction patterns)
[[guardrails.output.providers]]
type = "regex"
max_length_bytes = 50000
patterns_file = "patterns/output.txt"

# Then validate against schema
[[guardrails.output.providers]]
type = "json_schema"
schema_file = "schemas/requirements.json"
```

With `sequential` execution and `all_must_pass` aggregation, the regex check runs first. If it fails, the JSON Schema check is skipped (short-circuit).

## Validation Behavior

### Edge Cases

| Input | Result | Rule Code | Message |
|-------|--------|-----------|---------|
| Empty string | Fail | `JSON_PARSE_ERROR` | "Content is empty (expected valid JSON)" |
| Not JSON | Fail | `JSON_PARSE_ERROR` | "Content is not valid JSON: ..." |
| Valid JSON, wrong type | Fail | `JSON_SCHEMA_VIOLATION` | Type mismatch details |
| Missing required field | Fail | `JSON_SCHEMA_VIOLATION` | Missing field details |
| Extra field (with `additionalProperties: false`) | Fail | `JSON_SCHEMA_VIOLATION` | Additional property details |
| Valid JSON matching schema | Pass | - | - |

### Violation Details

Each schema violation includes:
- **Rule**: `JSON_SCHEMA_VIOLATION` (or `JSON_PARSE_ERROR` for unparseable content)
- **Severity**: `High`
- **Location**: JSON path to the failing field (e.g., `/requirements/0/id`)
- **Message**: Human-readable error from the JSON Schema validator

Violations are capped at 25 per validation to prevent unbounded output.

### Error Output

When the guardrail rejects a response:

```json
{
  "status": "error",
  "response": null,
  "metadata": { ... },
  "error": {
    "code": "OUTPUT_VALIDATION_FAILED",
    "message": "JSON_SCHEMA_VIOLATION: \"name\" is a required property"
  }
}
```

### Exit Codes

| Scenario | Exit Code |
|----------|-----------|
| Success (guardrails passed) | 0 |
| Guardrail rejection (input or output) | 9 |
| HTTP/network error | 3 |
| Invalid response from LLM | 4 |
| File not found | 5 |
| Invalid arguments | 6 |
| Auth failure | 7 |

### Constructor Errors

Schema validation happens at startup. If the schema file is invalid:

| Condition | Error |
|-----------|-------|
| File doesn't exist | `FileNotFound` |
| File is not valid JSON | `InvalidArguments` |
| File is valid JSON but not a valid JSON Schema | `InvalidArguments` |

## Library Usage

```rust
use fortified_llm_client::{
    evaluate, config_builder::ConfigBuilder,
    GuardrailProviderConfig,
};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ConfigBuilder::new()
        .api_url("http://localhost:11434/v1/chat/completions")
        .model("llama3")
        .system_prompt("Extract requirements as JSON")
        .user_prompt("Build a login page with OAuth support")
        .output_guardrails(GuardrailProviderConfig::JsonSchema {
            schema_file: PathBuf::from("schemas/requirements.json"),
        })
        .build()?;

    let result = evaluate(config).await?;

    if result.status == "success" {
        println!("Validated response: {:?}", result.response);
    } else {
        eprintln!("Validation failed: {:?}", result.error);
    }

    Ok(())
}
```

## See Also

- [Composite Guardrails]({{ site.baseurl }}{% link guardrails/hybrid.md %}) - Combine JSON Schema with other guardrails
- [Regex Guardrails]({{ site.baseurl }}{% link guardrails/regex.md %}) - Pattern-based pre-screening
- [Configuration]({{ site.baseurl }}{% link user-guide/configuration.md %}) - Full configuration reference
