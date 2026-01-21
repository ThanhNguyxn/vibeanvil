//! Ollama provider - Local LLM inference
//!
//! Ollama (https://github.com/ollama/ollama) provides local LLM inference
//! with an OpenAI-compatible API. Features:
//! - 100+ models available
//! - OpenAI-compatible endpoints
//! - GGUF model import
//! - Custom Modelfiles
//!
//! Configuration via environment variables:
//! - `OLLAMA_HOST`: Ollama server URL (default: http://localhost:11434)
//! - `OLLAMA_MODEL`: Model to use (default: llama3.2)
//! - `OLLAMA_CONTEXT_SIZE`: Context window size (default: 8192)
//! - `OLLAMA_TEMPERATURE`: Sampling temperature (default: 0.7)

use anyhow::{Context as AnyhowContext, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use super::safety::{get_timeout_secs, truncate_output, MAX_OUTPUT_BYTES};
use super::{Context, Provider, ProviderResponse};

/// Ollama API request for /api/generate
#[derive(Debug, Serialize)]
struct OllamaGenerateRequest {
    model: String,
    prompt: String,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    options: OllamaOptions,
}

/// Ollama generation options
#[derive(Debug, Serialize)]
struct OllamaOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    num_ctx: Option<u32>,
}

/// Ollama API response from /api/generate
#[derive(Debug, Deserialize)]
struct OllamaGenerateResponse {
    response: String,
    done: bool,
    #[serde(default)]
    context: Vec<i64>,
    #[serde(default)]
    total_duration: u64,
    #[serde(default)]
    eval_count: u32,
}

/// Chat message for /api/chat
#[derive(Debug, Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

/// Ollama chat request
#[derive(Debug, Serialize)]
struct OllamaChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    stream: bool,
    options: OllamaOptions,
}

/// Ollama chat response
#[derive(Debug, Deserialize)]
struct OllamaChatResponse {
    message: ChatMessage,
    done: bool,
}

/// Ollama provider for local LLM inference
///
/// Supports:
/// - Text generation via /api/generate
/// - Chat via /api/chat (OpenAI compatible)
/// - Embeddings via /api/embed
pub struct OllamaProvider {
    /// Ollama server URL
    host: String,
    /// Model name
    model: String,
    /// Context window size
    context_size: u32,
    /// Sampling temperature
    temperature: f32,
    /// Request timeout
    timeout: Duration,
    /// System prompt for chat
    system_prompt: Option<String>,
    /// HTTP client
    client: reqwest::Client,
}

impl OllamaProvider {
    /// Create new Ollama provider with default settings
    pub fn new() -> Self {
        let host = std::env::var("OLLAMA_HOST")
            .unwrap_or_else(|_| "http://localhost:11434".to_string());

        let model = std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "llama3.2".to_string());

        let context_size: u32 = std::env::var("OLLAMA_CONTEXT_SIZE")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(8192);

        let temperature: f32 = std::env::var("OLLAMA_TEMPERATURE")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(0.7);

        let timeout_secs = get_timeout_secs();

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .build()
            .unwrap_or_default();

        Self {
            host,
            model,
            context_size,
            temperature,
            timeout: Duration::from_secs(timeout_secs),
            system_prompt: None,
            client,
        }
    }

    /// Create provider with specific model
    pub fn with_model(model: &str) -> Self {
        let mut provider = Self::new();
        provider.model = model.to_string();
        provider
    }

    /// Set system prompt
    pub fn with_system_prompt(mut self, prompt: &str) -> Self {
        self.system_prompt = Some(prompt.to_string());
        self
    }

    /// Check if Ollama server is available
    async fn check_available(&self) -> bool {
        let url = format!("{}/api/tags", self.host);
        self.client.get(&url).send().await.is_ok()
    }

    /// Generate text using /api/generate endpoint
    async fn generate(&self, prompt: &str) -> Result<String> {
        let url = format!("{}/api/generate", self.host);

        let request = OllamaGenerateRequest {
            model: self.model.clone(),
            prompt: prompt.to_string(),
            stream: false,
            system: self.system_prompt.clone(),
            options: OllamaOptions {
                temperature: Some(self.temperature),
                num_ctx: Some(self.context_size),
            },
        };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .with_context(|| format!("Failed to connect to Ollama at {}", self.host))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "Ollama request failed with status {}: {}",
                status,
                error_text
            ));
        }

        let result: OllamaGenerateResponse = response
            .json()
            .await
            .with_context(|| "Failed to parse Ollama response")?;

        Ok(result.response)
    }

    /// Chat using /api/chat endpoint (OpenAI compatible)
    async fn chat(&self, messages: Vec<ChatMessage>) -> Result<String> {
        let url = format!("{}/api/chat", self.host);

        let request = OllamaChatRequest {
            model: self.model.clone(),
            messages,
            stream: false,
            options: OllamaOptions {
                temperature: Some(self.temperature),
                num_ctx: Some(self.context_size),
            },
        };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .with_context(|| format!("Failed to connect to Ollama at {}", self.host))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "Ollama chat failed with status {}: {}",
                status,
                error_text
            ));
        }

        let result: OllamaChatResponse = response
            .json()
            .await
            .with_context(|| "Failed to parse Ollama chat response")?;

        Ok(result.message.content)
    }

    /// List available models
    pub async fn list_models(&self) -> Result<Vec<String>> {
        let url = format!("{}/api/tags", self.host);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .with_context(|| format!("Failed to connect to Ollama at {}", self.host))?;

        #[derive(Deserialize)]
        struct TagsResponse {
            models: Vec<ModelInfo>,
        }

        #[derive(Deserialize)]
        struct ModelInfo {
            name: String,
        }

        let result: TagsResponse = response.json().await?;
        Ok(result.models.into_iter().map(|m| m.name).collect())
    }
}

impl Default for OllamaProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Provider for OllamaProvider {
    async fn execute(&self, prompt: &str, context: &Context) -> Result<ProviderResponse> {
        // Build messages for chat
        let mut messages = Vec::new();

        // Add system prompt if configured
        if let Some(ref system) = self.system_prompt {
            messages.push(ChatMessage {
                role: "system".to_string(),
                content: system.clone(),
            });
        } else {
            // Default coding assistant prompt
            messages.push(ChatMessage {
                role: "system".to_string(),
                content: format!(
                    "You are an expert coding assistant. \
                     You are working in directory: {}. \
                     Provide clear, correct code with explanations. \
                     Use markdown for code blocks.",
                    context.working_dir.display()
                ),
            });
        }

        // Add user prompt
        messages.push(ChatMessage {
            role: "user".to_string(),
            content: prompt.to_string(),
        });

        // Execute chat
        match self.chat(messages).await {
            Ok(response) => {
                let output = truncate_output(&response, MAX_OUTPUT_BYTES);
                Ok(ProviderResponse {
                    success: true,
                    output,
                    errors: vec![],
                    warnings: vec![format!("Using local model: {}", self.model)],
                    files_modified: vec![],
                })
            }
            Err(e) => {
                // Check if it's a connection error
                let error_msg = e.to_string();
                if error_msg.contains("connect") || error_msg.contains("Connection refused") {
                    Err(anyhow::anyhow!(
                        "Cannot connect to Ollama server.\n\n\
                         Make sure Ollama is running:\n  \
                         ollama serve\n\n\
                         Or start it with a specific model:\n  \
                         ollama run {}\n\n\
                         Install Ollama from: https://ollama.com",
                        self.model
                    ))
                } else {
                    Err(e)
                }
            }
        }
    }

    async fn generate_commit_message(&self, diff: &str, _context: &Context) -> Result<String> {
        let prompt = format!(
            "Generate a concise git commit message for this diff. \
             Use conventional commits format (type: description). \
             Output only the commit message, nothing else.\n\n\
             ```diff\n{}\n```",
            &diff[..diff.len().min(4000)] // Limit diff size
        );

        let response = self.generate(&prompt).await?;

        // Extract first non-empty line
        Ok(response
            .lines()
            .find(|l| !l.trim().is_empty())
            .unwrap_or("update: apply changes")
            .trim()
            .to_string())
    }

    fn name(&self) -> &str {
        "ollama"
    }

    fn is_available(&self) -> bool {
        // We can't do async check here, so just check if host is configured
        !self.host.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_creation() {
        // Clear env var if set by other tests
        std::env::remove_var("OLLAMA_MODEL");
        let provider = OllamaProvider::new();
        assert_eq!(provider.name(), "ollama");
        assert_eq!(provider.model, "llama3.2");
    }

    #[test]
    fn test_with_model() {
        let provider = OllamaProvider::with_model("codellama");
        assert_eq!(provider.model, "codellama");
    }

    #[test]
    fn test_with_system_prompt() {
        let provider = OllamaProvider::new().with_system_prompt("You are a Rust expert");
        assert!(provider.system_prompt.is_some());
    }

    #[test]
    fn test_is_available() {
        let provider = OllamaProvider::new();
        // Should return true if host is configured (even if not actually running)
        assert!(provider.is_available());
    }

    #[test]
    fn test_env_config() {
        // Test that env vars are read correctly
        std::env::set_var("OLLAMA_MODEL", "test-model");
        let provider = OllamaProvider::new();
        assert_eq!(provider.model, "test-model");
        // Cleanup
        std::env::remove_var("OLLAMA_MODEL");
        std::env::remove_var("OLLAMA_MODEL");
    }
}
