use thiserror::Error;

#[derive(Debug, Error)]
pub enum CliError {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("Context limit exceeded: {required} tokens required but limit is {limit} tokens (excess: {excess})")]
    ContextLimitExceeded {
        required: usize,
        limit: usize,
        excess: usize,
    },

    #[error("Invalid API response: {0}")]
    InvalidResponse(String),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Invalid arguments: {0}")]
    InvalidArguments(String),

    #[error("API authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("PDF processing failed: {0}")]
    PdfProcessingFailed(String),
}

impl CliError {
    /// Get the error code string for JSON output
    pub fn code(&self) -> &'static str {
        match self {
            Self::ContextLimitExceeded { .. } => "CONTEXT_LIMIT_EXCEEDED",
            Self::HttpError(_) => "HTTP_ERROR",
            Self::InvalidResponse(_) => "INVALID_RESPONSE",
            Self::FileNotFound(_) => "FILE_NOT_FOUND",
            Self::InvalidArguments(_) => "INVALID_ARGUMENTS",
            Self::AuthenticationFailed(_) => "AUTH_FAILED",
            Self::PdfProcessingFailed(_) => "PDF_PROCESSING_FAILED",
        }
    }

    /// Get the exit code for this error
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::ContextLimitExceeded { .. } => 2,
            Self::HttpError(_) => 3,
            Self::InvalidResponse(_) => 4,
            Self::FileNotFound(_) => 5,
            Self::InvalidArguments(_) => 6,
            Self::AuthenticationFailed(_) => 7,
            Self::PdfProcessingFailed(_) => 8,
        }
    }
}
