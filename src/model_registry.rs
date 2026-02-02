//! Model registry with tokenizer characteristics and context limits
//!
//! Provides model-specific token estimation and context window information.

use once_cell::sync::Lazy;
use std::collections::HashMap;

/// Tokenizer family with specific encoding characteristics
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenizerFamily {
    /// OpenAI GPT tokenizers (cl100k_base, p50k_base)
    Gpt,
    /// Meta Llama tokenizers
    Llama,
    /// Qwen tokenizers
    Qwen,
    /// Mistral tokenizers
    Mistral,
    /// Generic/unknown tokenizer
    Generic,
}

impl TokenizerFamily {
    /// Get approximate chars per token for this tokenizer family
    ///
    /// These are empirical averages for English text.
    /// Actual values vary by content type (code vs prose vs special chars).
    pub fn chars_per_token(&self) -> f64 {
        match self {
            Self::Gpt => 4.0,     // GPT-3.5/4 tokenizer
            Self::Llama => 3.8,   // Llama 2/3 tokenizer (slightly more efficient than GPT)
            Self::Qwen => 3.7,    // Qwen tokenizer (similar to Llama)
            Self::Mistral => 3.8, // Mistral tokenizer (based on Llama)
            Self::Generic => 4.0, // Conservative default
        }
    }
}

/// Model information for token estimation and validation
#[derive(Debug, Clone)]
pub struct ModelInfo {
    /// Model name/identifier
    pub name: &'static str,

    /// Tokenizer family
    pub tokenizer: TokenizerFamily,

    /// Context window size (total tokens including input + output)
    pub context_window: usize,

    /// Recommended max input tokens (leave room for output)
    pub max_input_tokens: Option<usize>,

    /// Provider/organization
    pub provider: &'static str,
}

impl ModelInfo {
    /// Estimate tokens for text using model-specific tokenizer
    pub fn estimate_tokens(&self, text: &str) -> usize {
        (text.len() as f64 / self.tokenizer.chars_per_token()).ceil() as usize
    }
}

/// Global model registry
static MODEL_REGISTRY: Lazy<HashMap<&'static str, ModelInfo>> = Lazy::new(|| {
    let mut registry = HashMap::new();

    // OpenAI GPT models
    registry.insert(
        "gpt-4",
        ModelInfo {
            name: "gpt-4",
            tokenizer: TokenizerFamily::Gpt,
            context_window: 8192,
            max_input_tokens: Some(6144), // Leave 2048 for output
            provider: "OpenAI",
        },
    );

    registry.insert(
        "gpt-4-turbo",
        ModelInfo {
            name: "gpt-4-turbo",
            tokenizer: TokenizerFamily::Gpt,
            context_window: 128000,
            max_input_tokens: Some(100000),
            provider: "OpenAI",
        },
    );

    registry.insert(
        "gpt-4o",
        ModelInfo {
            name: "gpt-4o",
            tokenizer: TokenizerFamily::Gpt,
            context_window: 128000,
            max_input_tokens: Some(100000),
            provider: "OpenAI",
        },
    );

    registry.insert(
        "gpt-3.5-turbo",
        ModelInfo {
            name: "gpt-3.5-turbo",
            tokenizer: TokenizerFamily::Gpt,
            context_window: 16385,
            max_input_tokens: Some(12288),
            provider: "OpenAI",
        },
    );

    // Meta Llama models
    registry.insert(
        "llama-3.1-8b",
        ModelInfo {
            name: "llama-3.1-8b",
            tokenizer: TokenizerFamily::Llama,
            context_window: 128000,
            max_input_tokens: Some(100000),
            provider: "Meta",
        },
    );

    registry.insert(
        "llama-3.1-70b",
        ModelInfo {
            name: "llama-3.1-70b",
            tokenizer: TokenizerFamily::Llama,
            context_window: 128000,
            max_input_tokens: Some(100000),
            provider: "Meta",
        },
    );

    registry.insert(
        "llama-3.1-405b",
        ModelInfo {
            name: "llama-3.1-405b",
            tokenizer: TokenizerFamily::Llama,
            context_window: 128000,
            max_input_tokens: Some(100000),
            provider: "Meta",
        },
    );

    registry.insert(
        "llama-3.2-1b",
        ModelInfo {
            name: "llama-3.2-1b",
            tokenizer: TokenizerFamily::Llama,
            context_window: 131072,
            max_input_tokens: Some(100000),
            provider: "Meta",
        },
    );

    registry.insert(
        "llama-3.2-3b",
        ModelInfo {
            name: "llama-3.2-3b",
            tokenizer: TokenizerFamily::Llama,
            context_window: 131072,
            max_input_tokens: Some(100000),
            provider: "Meta",
        },
    );

    // Qwen models
    registry.insert(
        "qwen-2.5-72b",
        ModelInfo {
            name: "qwen-2.5-72b",
            tokenizer: TokenizerFamily::Qwen,
            context_window: 131072,
            max_input_tokens: Some(100000),
            provider: "Alibaba",
        },
    );

    registry.insert(
        "qwen-2.5-7b",
        ModelInfo {
            name: "qwen-2.5-7b",
            tokenizer: TokenizerFamily::Qwen,
            context_window: 131072,
            max_input_tokens: Some(100000),
            provider: "Alibaba",
        },
    );

    // Mistral models
    registry.insert(
        "mistral-7b",
        ModelInfo {
            name: "mistral-7b",
            tokenizer: TokenizerFamily::Mistral,
            context_window: 32768,
            max_input_tokens: Some(24576),
            provider: "Mistral AI",
        },
    );

    registry.insert(
        "mixtral-8x7b",
        ModelInfo {
            name: "mixtral-8x7b",
            tokenizer: TokenizerFamily::Mistral,
            context_window: 32768,
            max_input_tokens: Some(24576),
            provider: "Mistral AI",
        },
    );

    registry
});

/// Look up model information by name
///
/// Performs fuzzy matching to handle common naming variations:
/// - Strips version suffixes (e.g., "llama-3.1-8b-instruct" → "llama-3.1-8b")
/// - Normalizes case
/// - Handles common aliases
pub fn lookup_model(name: &str) -> Option<&'static ModelInfo> {
    let normalized = name.to_lowercase();

    // Try exact match first
    if let Some(info) = MODEL_REGISTRY.get(normalized.as_str()) {
        return Some(info);
    }

    // Try fuzzy matching (strip common suffixes)
    let stripped = normalized
        .trim_end_matches("-instruct")
        .trim_end_matches("-chat")
        .trim_end_matches("-turbo")
        .trim_end_matches("-preview");

    if let Some(info) = MODEL_REGISTRY.get(stripped) {
        return Some(info);
    }

    // Try prefix matching (for versioned models)
    for (key, info) in MODEL_REGISTRY.iter() {
        if normalized.starts_with(key) {
            return Some(info);
        }
    }

    None
}

/// Get all registered model names
pub fn list_models() -> Vec<&'static str> {
    let mut models: Vec<_> = MODEL_REGISTRY.keys().copied().collect();
    models.sort_unstable();
    models
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenizer_family_chars_per_token() {
        assert_eq!(TokenizerFamily::Gpt.chars_per_token(), 4.0);
        assert_eq!(TokenizerFamily::Llama.chars_per_token(), 3.8);
        assert_eq!(TokenizerFamily::Qwen.chars_per_token(), 3.7);
    }

    #[test]
    fn test_lookup_exact_match() {
        let info = lookup_model("gpt-4").unwrap();
        assert_eq!(info.name, "gpt-4");
        assert_eq!(info.context_window, 8192);
    }

    #[test]
    fn test_lookup_with_suffix() {
        let info = lookup_model("llama-3.1-8b-instruct").unwrap();
        assert_eq!(info.name, "llama-3.1-8b");
        assert_eq!(info.tokenizer, TokenizerFamily::Llama);
    }

    #[test]
    fn test_lookup_case_insensitive() {
        let info = lookup_model("GPT-4-TURBO").unwrap();
        assert_eq!(info.name, "gpt-4-turbo");
    }

    #[test]
    fn test_lookup_unknown_model() {
        assert!(lookup_model("unknown-model-xyz").is_none());
    }

    #[test]
    fn test_model_estimate_tokens() {
        let info = lookup_model("gpt-4").unwrap();
        let text = "Hello, world!"; // 13 chars
        let tokens = info.estimate_tokens(text);
        assert_eq!(tokens, 4); // ceil(13 / 4.0) = 4

        let llama_info = lookup_model("llama-3.1-8b").unwrap();
        let llama_tokens = llama_info.estimate_tokens(text);
        assert_eq!(llama_tokens, 4); // ceil(13 / 3.8) = 4
    }

    #[test]
    fn test_list_models() {
        let models = list_models();
        assert!(models.contains(&"gpt-4"));
        assert!(models.contains(&"llama-3.1-8b"));
        assert!(models.contains(&"qwen-2.5-72b"));
    }
}
