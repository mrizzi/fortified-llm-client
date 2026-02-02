mod detection;
mod logging;
mod ollama;
mod openai;

// Re-export public items
pub use detection::{create_provider, detect_provider_type};
pub use ollama::OllamaProvider;
pub use openai::OpenAIProvider;
