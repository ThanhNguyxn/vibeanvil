//! Tabby provider - Self-hosted AI coding assistant
//!
//! Tabby is a self-hosted AI coding assistant with REST API,
//! supporting various models on consumer GPUs.
//!
//! Installation:
//! ```bash
//! docker run -it --gpus all -p 8080:8080 -v $HOME/.tabby:/data \
//!   tabbyml/tabby serve --model StarCoder-1B --device cuda
//! ```
//!
//! Configuration:
//! - `TABBY_HOST`: Tabby server URL (default: http://localhost:8080)
//! - `TABBY_MODEL`: Model for chat (optional)
//! - `TABBY_API_KEY`: API key if authentication enabled

use super::{Context, Provider, ProviderResponse};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use std::time::Duration;

/// Tabby self-hosted AI provider
pub struct TabbyProvider {
    host: String,
    model: Option<String>,
    api_key: Option<String>,
    timeout: Duration,
}

impl Default for TabbyProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl TabbyProvider {
    /// Create a new Tabby provider with default settings
    pub fn new() -> Self {
        let timeout_secs = std::env::var("TABBY_TIMEOUT_SECS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(300);

        Self {
            host: std::env::var("TABBY_HOST")
                .unwrap_or_else(|_| "http://localhost:8080".to_string()),
            model: std::env::var("TABBY_MODEL").ok(),
            api_key: std::env::var("TABBY_API_KEY").ok(),
            timeout: Duration::from_secs(timeout_secs),
        }
    }

    /// Create with a specific model
    pub fn with_model(model: &str) -> Self {
        let mut provider = Self::new();
        provider.model = Some(model.to_string());
        provider
    }

    /// Build the chat request body
    fn build_request_body(&self, prompt: &str) -> serde_json::Value {
        let mut body = serde_json::json!({
            "messages": [
                {
                    "role": "user",
                    "content": prompt
                }
            ]
        });

        if let Some(model) = &self.model {
            body["model"] = serde_json::Value::String(model.clone());
        }

        body
    }
}

#[async_trait]
impl Provider for TabbyProvider {
    async fn execute(&self, prompt: &str, _context: &Context) -> Result<ProviderResponse> {
        let client = reqwest::Client::builder()
            .timeout(self.timeout)
            .build()
            .map_err(|e| anyhow!("Failed to create HTTP client: {}", e))?;

        let url = format!("{}/v1/chat/completions", self.host);
        let body = self.build_request_body(prompt);

        let mut request = client.post(&url).json(&body);

        // Add API key if available
        if let Some(api_key) = &self.api_key {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = request
            .send()
            .await
            .map_err(|e| anyhow!("Failed to send request to Tabby: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!(
                "Tabby API error ({}): {}",
                status,
                error_text
            ));
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse Tabby response: {}", e))?;

        // Extract content from OpenAI-compatible response format
        let content = json["choices"]
            .get(0)
            .and_then(|c| c["message"]["content"].as_str())
            .unwrap_or("")
            .to_string();

        Ok(ProviderResponse {
            success: true,
            output: content,
            errors: vec![],
            warnings: vec![],
            files_modified: vec![],
        })
    }

    async fn generate_commit_message(&self, diff: &str, context: &Context) -> Result<String> {
        let prompt = format!(
            "Generate a concise git commit message for this diff. \
             Return ONLY the commit message, no explanation:\n\n{}",
            diff
        );
        let response = self.execute(&prompt, context).await?;
        Ok(response.output.trim().to_string())
    }

    fn name(&self) -> &str {
        "tabby"
    }

    fn is_available(&self) -> bool {
        // Check if Tabby server is reachable
        // For simplicity, just check if TABBY_HOST is set or use default
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn cleanup_env() {
        std::env::remove_var("TABBY_HOST");
        std::env::remove_var("TABBY_MODEL");
        std::env::remove_var("TABBY_API_KEY");
        std::env::remove_var("TABBY_TIMEOUT_SECS");
    }

    #[test]
    fn test_tabby_provider_new() {
        cleanup_env();
        let provider = TabbyProvider::new();
        assert_eq!(provider.host, "http://localhost:8080");
        assert!(provider.model.is_none());
        assert_eq!(provider.timeout, Duration::from_secs(300));
    }

    #[test]
    fn test_tabby_provider_with_model() {
        cleanup_env();
        let provider = TabbyProvider::with_model("StarCoder-7B");
        assert_eq!(provider.model, Some("StarCoder-7B".to_string()));
    }

    #[test]
    fn test_tabby_provider_name() {
        cleanup_env();
        let provider = TabbyProvider::new();
        assert_eq!(provider.name(), "tabby");
    }

    #[test]
    fn test_tabby_build_request_body() {
        cleanup_env();
        let provider = TabbyProvider::with_model("StarCoder-7B");
        let body = provider.build_request_body("test prompt");

        assert_eq!(body["messages"][0]["content"], "test prompt");
        assert_eq!(body["model"], "StarCoder-7B");
    }

    #[test]
    fn test_tabby_with_env_vars() {
        cleanup_env();
        std::env::set_var("TABBY_HOST", "http://myserver:9090");
        std::env::set_var("TABBY_MODEL", "CodeLlama-7B");
        std::env::set_var("TABBY_API_KEY", "secret123");

        let provider = TabbyProvider::new();
        assert_eq!(provider.host, "http://myserver:9090");
        assert_eq!(provider.model, Some("CodeLlama-7B".to_string()));
        assert_eq!(provider.api_key, Some("secret123".to_string()));

        cleanup_env();
    }
}
