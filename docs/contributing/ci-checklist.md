---
layout: default
title: CI Checklist
parent: Contributing
nav_order: 2
---

# Pre-Push CI Checklist

Run these commands locally before pushing to ensure CI passes.

## The Checklist

```bash
# 1. Format check (CI fails if not formatted)
cargo +nightly fmt --check

# 2. Compilation check (warnings = errors)
RUSTFLAGS="-D warnings" cargo check

# 3. Linting (warnings = errors)
cargo clippy -- -D warnings

# 4. Tests (warnings = errors, requires docling)
RUSTFLAGS="-D warnings" cargo test
```

## One-Liner

```bash
cargo +nightly fmt --check && \
RUSTFLAGS="-D warnings" cargo check && \
cargo clippy -- -D warnings && \
RUSTFLAGS="-D warnings" cargo test
```

## Why Each Step

1. **Format check** - Ensures consistent code style (uses nightly rustfmt for `imports_granularity`)
2. **Compilation** - Catches compile errors and warnings
3. **Clippy** - Catches common mistakes and anti-patterns
4. **Tests** - Validates functionality (requires `pip install docling`)

## If Checks Fail

### Format

```bash
cargo +nightly fmt
```

### Warnings

Fix warnings shown by `cargo check` or `cargo clippy`.

### Tests

```bash
cargo test -- --nocapture  # See test output
cargo test test_name       # Run specific test
```

## Nightly Requirement

**Why `+nightly`?** Project uses `imports_granularity` in `rustfmt.toml` (nightly-only feature).

**Install**: `rustup toolchain install nightly`
