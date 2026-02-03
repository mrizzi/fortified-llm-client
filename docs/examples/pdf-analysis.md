---
layout: default
title: PDF Analysis Examples
parent: Examples
nav_order: 4
---

# PDF Analysis Examples

Extract and analyze PDF documents.

## Basic PDF Summarization

```bash
fortified-llm-client \
  --api-url http://localhost:11434/v1/chat/completions \
  --model llama3 \
  --pdf-file document.pdf \
  --system-text "Summarize the key points in bullet points"
```

## PDF Data Extraction with JSON Schema

`invoice-schema.json`:
```json
{
  "type": "object",
  "properties": {
    "invoice_number": {"type": "string"},
    "total": {"type": "number"},
    "vendor": {"type": "string"}
  },
  "required": ["invoice_number", "total", "vendor"]
}
```

```bash
fortified-llm-client \
  --api-url https://api.openai.com/v1/chat/completions \
  --model gpt-4 \
  --api-key-name OPENAI_API_KEY \
  --pdf-file invoice.pdf \
  --system-text "Extract invoice data" \
  --response-format json-schema \
  --response-format-schema invoice-schema.json
```

## Batch PDF Processing (Library)

```rust
use fortified_llm_client::{evaluate, EvaluationConfig};

async fn process_pdfs(pdf_files: Vec<&str>) -> Result<(), Box<dyn std::error::Error>> {
    for pdf in pdf_files {
        let config = EvaluationConfig {
            api_url: "http://localhost:11434/v1/chat/completions".to_string(),
            model: "llama3".to_string(),
            system_prompt: Some("Summarize in 2 sentences.".to_string()),
            pdf_input: Some(pdf.to_string()),
            ..Default::default()
        };

        match evaluate(config).await {
            Ok(result) => println!("{}: {}", pdf, result.content),
            Err(e) => eprintln!("Failed {}: {}", pdf, e),
        }
    }
    Ok(())
}
```
