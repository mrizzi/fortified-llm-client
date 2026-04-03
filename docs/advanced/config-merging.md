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

Config files are **partial overlays** — all fields are optional. Required-field validation happens after merging with CLI arguments, in `ConfigBuilder::build()`.

The resolution order for each field:
1. CLI argument (highest priority)
2. Config file value
3. Built-in default (if any)

If a required field is missing from both CLI and config file, `ConfigBuilder::build()` produces a clear error message indicating both sources.

```rust
pub struct ConfigFileRequest {
    pub api_url: Option<String>,     // Required overall, optional in config
    pub model: Option<String>,       // Required overall, optional in config
    pub system_prompt: Option<String>,
    pub temperature: Option<f32>,    // Default: 0.0 (applied by ConfigBuilder)
    pub timeout_secs: Option<u64>,   // Default: 300 (applied by ConfigBuilder)
    pub guardrails: Option<Guardrails>,
    // ... other optional fields
}
```

Parsed separately to support nested structures (guardrails) not in `Args`.

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
