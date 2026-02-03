---
layout: default
title: Development Setup
parent: Contributing
nav_order: 1
---

# Development Setup

Set up your local development environment.

## Prerequisites

```bash
# Rust stable + nightly
rustup toolchain install stable nightly

# Clippy and rustfmt
rustup component add clippy
rustup component add rustfmt --toolchain nightly

# Docling (for PDF tests)
pip install docling
```

## Clone and Build

```bash
git clone https://github.com/mrizzi/fortified-llm-client
cd fortified-llm-client

# Build
cargo build

# Run tests
cargo test
```

## Development Workflow

```bash
# Make changes
vim src/...

# Format code
cargo +nightly fmt

# Check compilation
cargo check

# Run linter
cargo clippy

# Run tests
cargo test

# Run specific test
cargo test test_name -- --nocapture
```

## IDE Setup

### VS Code

Install extensions:
- rust-analyzer
- CodeLLDB (debugging)

`.vscode/settings.json`:
```json
{
  "rust-analyzer.checkOnSave.command": "clippy"
}
```

### IntelliJ IDEA

Install Rust plugin, enable Clippy in settings.
