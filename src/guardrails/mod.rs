pub mod config;
pub mod gpt_oss_safeguard;
pub mod hybrid;
pub mod input;
pub mod llama_guard;
pub mod llama_prompt_guard;
pub mod output;
pub mod patterns;
pub mod provider;

// Re-export core trait types
pub use provider::{
    GptOssSafeguardResult, GuardrailProvider, GuardrailResult, LlamaGuardResult,
    ProviderSpecificResult, Severity, Violation,
};

// Re-export concrete implementations
pub use config::{
    create_guardrail_provider, create_output_guardrail_provider, AggregationMode, ExecutionMode,
    GuardrailConfig, GuardrailProviderConfig, OutputGuardrailProviderConfig,
};
pub use gpt_oss_safeguard::{GptOssSafeguardConfig, GptOssSafeguardProvider};
pub use hybrid::HybridGuardrail;
pub use input::{InputGuardrail, InputGuardrailConfig};
pub use llama_guard::{LlamaGuardCategory, LlamaGuardConfig, LlamaGuardProvider};
pub use llama_prompt_guard::{
    LlamaPromptGuardConfig, LlamaPromptGuardProvider, LlamaPromptGuardResult,
};
pub use output::{OutputGuardrail, OutputGuardrailConfig};
