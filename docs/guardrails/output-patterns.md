---
layout: default
title: Output Patterns
parent: Guardrails
nav_order: 2
---

# Output Pattern Validation

Validate LLM responses for safety and quality.

## Configuration

```toml
[guardrails.output]
type = "patterns"

[guardrails.output.patterns]
detect_toxic = true
min_quality_score = 0.7
max_length_bytes = 10485760  # 10MB
```

## Checks

- **Toxic content detection** - Offensive language, hate speech
- **Quality scoring** - Coherence, completeness
- **Length validation** - Prevent oversized responses

## Usage

```bash
fortified-llm-client -c config.toml --user-text "prompt"
```

Response rejected if:
- Toxic content detected
- Quality score < threshold
- Length exceeds limit
