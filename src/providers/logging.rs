use crate::error::CliError;
use serde::Serialize;

/// Handle non-success HTTP responses uniformly across providers.
///
/// Maps 401 to `AuthenticationFailed` and all other errors to
/// `InvalidResponse` with the status code, reason, and API response body.
pub async fn handle_error_response(response: reqwest::Response) -> CliError {
    let status = response.status();

    let error_body = match response.text().await {
        Ok(body) => body,
        Err(e) => {
            log::warn!("Failed to read error response body: {e}");
            String::new()
        }
    };

    if status == 401 {
        let detail = if error_body.is_empty() {
            "Invalid or missing API key".to_string()
        } else {
            format!("Authentication failed: {error_body}")
        };
        return CliError::AuthenticationFailed(detail);
    }

    let error_msg = format!(
        "HTTP {} error: {}\nResponse from API: {}",
        status.as_u16(),
        status.canonical_reason().unwrap_or("Unknown error"),
        if error_body.is_empty() {
            "(error response body could not be read)"
        } else {
            &error_body
        }
    );

    CliError::InvalidResponse(error_msg)
}

/// Log request JSON for debugging (pretty-printed if possible)
pub fn log_request<T: Serialize>(request: &T) {
    if let Ok(request_json) = serde_json::to_string_pretty(request) {
        log::debug!("Request JSON sent to API:\n{request_json}");
    }
}

/// Log response JSON for debugging (pretty-printed if possible)
pub fn log_response(response_text: &str) {
    // Try to pretty-print if it's valid JSON, otherwise log as-is
    let display_text = serde_json::from_str::<serde_json::Value>(response_text)
        .and_then(|v| serde_json::to_string_pretty(&v))
        .unwrap_or_else(|_| response_text.to_string());
    log::debug!("Response JSON received from API:\n{display_text}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_request_valid_json() {
        #[derive(Serialize)]
        struct TestRequest {
            model: String,
            prompt: String,
        }

        let request = TestRequest {
            model: "test-model".to_string(),
            prompt: "test prompt".to_string(),
        };

        // Should not panic
        log_request(&request);
    }

    #[test]
    fn test_log_response_valid_json() {
        let response = r#"{"model":"test","response":"hello"}"#;
        // Should not panic
        log_response(response);
    }

    #[test]
    fn test_log_response_invalid_json() {
        let response = "not valid json";
        // Should not panic
        log_response(response);
    }
}
