---
layout: default
title: Advanced
nav_order: 7
has_children: true
permalink: /advanced/
---

# Advanced Topics

Deep dives into internal mechanisms and advanced usage.

## Contents

- **[Config Merging]({% link advanced/config-merging.md %})** - Figment + ConfigBuilder internals
- **[Error Handling]({% link advanced/error-handling.md %})** - Error types and recovery strategies
- **[Security]({% link advanced/security.md %})** - Security features checklist
- **[Extending]({% link advanced/extending.md %})** - Adding providers and guardrails

## Topics Covered

### Configuration System
- Dual loading (Figment + ConfigFileRequest)
- Merge priority rules
- CLI-only vs config-file fields

### Error Management
- `FortifiedError` variants
- Error propagation patterns
- Graceful degradation

### Security Features
- Input limits
- Timeout protection
- API key handling
- Atomic file writes

### Extensibility
- Implementing `LlmProvider` trait
- Creating custom guardrails
- Adding new response formats
