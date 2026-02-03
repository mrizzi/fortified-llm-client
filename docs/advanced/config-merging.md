---
layout: default
title: Config Merging
parent: Advanced
nav_order: 1
---

# Configuration Merging

How Figment and ConfigBuilder work together.

## Dual System

**Two loading mechanisms**:
1. **Figment** (`main.rs::merge_config()`) - Scalar fields (api_url, model, temperature, etc.)
2. **ConfigFileRequest** (`config.rs`) - Complex nested structures (guardrails)

## Figment Merging

**Priority**: CLI args > Config file

```rust
fn merge_config(args: &Args) -> Result<Args, CliError> {
    let file_provider = match config_path.extension() {
        Some("json") => Figment::from(Json::file(config_path)),
        Some("toml") => Figment::from(Toml::file(config_path)),
        // ...
    };

    file_provider
        .merge(Serialized::defaults(args))  // CLI wins
        .extract()
}
```

**Example**:
- Config: `temperature = 0.5`
- CLI: `--temperature 0.9`
- Result: `0.9` (CLI overrides)

## ConfigFileRequest

Loads guardrails from TOML/JSON:

```rust
pub struct ConfigFileRequest {
    pub guardrails: Option<Guardrails>,
}

pub struct Guardrails {
    pub input: Option<GuardrailConfig>,
    pub output: Option<GuardrailConfig>,
}
```

Parsed separately to support nested structures not in `Args`.

## CLI-Only Fields

These cannot be set in config files:

- `config_file` - Path to config itself
- `verbose` - Logging flag
- `quiet` - Logging flag
- `output` - Output file path
- `enable_input_validation` - CLI validation flag
- `max_input_length` - CLI validation limit
- `max_input_tokens` - CLI validation limit

**Restoration** in `merge_config()`:
```rust
Ok(Args {
    config_file: args.config_file.clone(),
    verbose: args.verbose,
    // ... restore all CLI-only fields
    ..merged
})
```
