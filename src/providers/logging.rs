use serde::Serialize;

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
