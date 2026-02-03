---
layout: default
title: Security
parent: Advanced
nav_order: 3
---

# Security Features

Security protections built into Fortified LLM Client.

## Input Validation

1. **Max input length** - Prevents resource exhaustion (default: 1MB)
2. **Max input tokens** - Prevents context overflow (default: 200K)
3. **PDF size validation** - Max 50MB per PDF
4. **Guardrails** - Pattern + LLM-based validation

## API Key Handling

**Best practices**:
- Use `--api-key-name` with environment variables (not `--api-key`)
- Never commit API keys in config files
- Keys stored in memory only, not logged

## Timeout Protection

- **Default**: 300 seconds (5 minutes)
- **Configurable**: `--timeout` flag
- Prevents hanging on slow/unresponsive APIs

## Atomic File Writes

Output files use temp + rename:
```rust
// Write to temp file
fs::write(&temp_path, content)?;
// Atomic rename
fs::rename(&temp_path, &final_path)?;
```

Prevents partial writes on failure.

## Guardrails-Only Validate User Input

**System prompts are trusted** - only user prompts validated by guardrails.

## No Code Injection

All LLM inputs are data (JSON), not code. No `eval()` or dynamic execution.
