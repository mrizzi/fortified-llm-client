//! Application constants and default values
//!
//! Centralizes magic numbers and configuration defaults for maintainability.

/// Token estimation constants
pub mod token_estimation {
    /// Average characters per token (based on GPT tokenizer for English text)
    /// Different tokenizers may vary: GPT ~4.0, Llama ~3.5, Code ~2.5
    pub const CHARS_PER_TOKEN: f64 = 4.0;

    /// Safety margin multiplier for token estimates (10%)
    /// Helps account for tokenizer variations and special characters
    pub const SAFETY_MARGIN: f64 = 1.1;
}

/// Input validation limits
pub mod input_limits {
    /// Maximum input size in bytes (1MB)
    /// Protects against memory exhaustion from extremely large inputs
    pub const MAX_INPUT_BYTES: usize = 1_048_576;

    /// Maximum estimated tokens for input validation (200K tokens)
    /// Protects against context window overflow
    pub const MAX_TOKENS_ESTIMATED: usize = 200_000;
}

/// Output validation limits
pub mod output_limits {
    /// Maximum output size in bytes (500KB)
    /// Protects against memory exhaustion from extremely large responses
    pub const MAX_OUTPUT_BYTES: usize = 512_000;
}

/// PDF processing limits
pub mod pdf_limits {
    /// Maximum PDF file size in bytes (10MB)
    /// Prevents DoS attacks from enormous PDF uploads
    pub const MAX_PDF_SIZE_BYTES: u64 = 10_485_760;

    /// Maximum PDF extraction timeout in seconds (60s)
    /// Prevents hanging on malformed or extremely complex PDFs
    pub const MAX_EXTRACTION_TIMEOUT_SECS: u64 = 60;
}

/// LLM invocation defaults
pub mod llm_defaults {
    /// Default temperature for LLM sampling (0.0 = deterministic)
    pub const DEFAULT_TEMPERATURE: f32 = 0.0;

    /// Minimum allowed temperature
    pub const MIN_TEMPERATURE: f32 = 0.0;

    /// Maximum allowed temperature
    pub const MAX_TEMPERATURE: f32 = 2.0;

    /// Default maximum tokens for LLM response
    pub const DEFAULT_MAX_TOKENS: u32 = 4000;

    /// Default timeout for LLM API calls in seconds
    pub const DEFAULT_TIMEOUT_SECS: u64 = 300;
}

/// HTTP client configuration
pub mod http {
    /// Connection pool idle timeout in seconds
    pub const POOL_IDLE_TIMEOUT_SECS: u64 = 90;

    /// Maximum number of redirects to follow
    pub const MAX_REDIRECTS: usize = 10;
}

/// Guardrails defaults
pub mod guardrails {
    /// Default minimum quality score for output (0-10 scale)
    pub const DEFAULT_MIN_QUALITY_SCORE: f32 = 5.0;

    /// Maximum quality score (ceiling)
    pub const MAX_QUALITY_SCORE: f32 = 10.0;

    /// Minimum quality score (floor)
    pub const MIN_QUALITY_SCORE: f32 = 0.0;
}
