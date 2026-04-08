mod anthropic;
mod detection;
mod gemini;
mod logging;
mod ollama;
mod openai;

// Re-export public items
pub use anthropic::AnthropicProvider;
pub use detection::{create_provider, detect_provider_type};
pub use gemini::GeminiProvider;
pub use ollama::OllamaProvider;
pub use openai::OpenAIProvider;
