---
layout: default
title: Code Guidelines
parent: Contributing
nav_order: 3
---

# Code Guidelines

Coding standards and best practices.

## Rust Conventions

- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `rustfmt` (nightly) for formatting
- Pass `clippy` with no warnings
- Prefer explicit over implicit (avoid `unwrap()`)

## Error Handling

**Good**:
```rust
let result = some_function()?;
```

**Bad**:
```rust
let result = some_function().unwrap();  // Don't panic
```

## Testing

- Write tests for new features
- Use descriptive test names: `test_token_validation_fails_when_limit_exceeded()`
- Test error cases, not just happy paths

## Documentation

- Add doc comments for public APIs:
```rust
/// Evaluates a prompt against an LLM.
///
/// # Arguments
/// * `config` - Evaluation configuration
///
/// # Returns
/// Result with LLM response and metadata
pub async fn evaluate(config: EvaluationConfig) -> Result<...>
```

## Commit Messages

- Use imperative mood: "Add feature" not "Added feature"
- Reference issues: "Fix #123: Handle timeout errors"
- Keep first line < 72 characters

## Pull Request Guidelines

- Clear description of changes
- Link to related issue
- Include tests
- Pass CI checklist locally first
