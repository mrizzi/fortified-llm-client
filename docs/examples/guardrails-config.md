---
layout: default
title: Guardrails Examples
parent: Examples
nav_order: 2
---

# Guardrails Configuration Examples

Complete examples for all guardrail types.

## Pattern-Based Input Validation

```toml
api_url = "http://localhost:11434/v1/chat/completions"
model = "llama3"

[guardrails.input]
type = "patterns"
max_length_bytes = 1048576

[guardrails.input.patterns]
detect_pii = true
detect_prompt_injection = true
```

## Llama Guard

```toml
[guardrails.input]
type = "llama_guard"

[guardrails.input.llama_guard]
api_url = "http://localhost:11434/v1/chat/completions"
model = "llama-guard-3"
enabled_categories = ["S1", "S2", "S3", "S10"]
max_tokens = 512
timeout_secs = 60
```

## Hybrid (Defense-in-Depth)

```toml
[guardrails.input]
type = "hybrid"

[guardrails.input.hybrid]
execution_mode = "sequential"
aggregation_mode = "all"

# Layer 1: Fast patterns
[[guardrails.input.hybrid.providers]]
type = "patterns"
[guardrails.input.hybrid.providers.patterns]
detect_pii = true
detect_prompt_injection = true

# Layer 2: LLM validation
[[guardrails.input.hybrid.providers]]
type = "llama_guard"
[guardrails.input.hybrid.providers.llama_guard]
api_url = "http://localhost:11434/v1/chat/completions"
model = "llama-guard-3"
```

## Input + Output Guardrails

```toml
# Validate inputs
[guardrails.input]
type = "patterns"
[guardrails.input.patterns]
detect_prompt_injection = true

# Validate outputs
[guardrails.output]
type = "patterns"
[guardrails.output.patterns]
detect_toxic = true
min_quality_score = 0.7
```
