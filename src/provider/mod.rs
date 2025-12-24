//! Provider plugins for AI coding assistants

pub mod claude_code;

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Provider context
#[derive(Debug, Clone)]
pub struct Context {
    /// Working directory
    pub working_dir: PathBuf,
    /// Session ID
    pub session_id: String,
    /// Contract hash (if locked)
    pub contract_hash: Option<String>,
}

/// Provider response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderResponse {
    /// Whether the operation succeeded
    pub success: bool,
    /// Output text
    pub output: String,
    /// Errors encountered
    pub errors: Vec<String>,
    /// Warnings
    pub warnings: Vec<String>,
    /// Files modified
    pub files_modified: Vec<String>,
}

/// Provider trait - implement for each AI provider
#[async_trait]
pub trait Provider: Send + Sync {
    /// Execute a prompt with the provider
    async fn execute(&self, prompt: &str, context: &Context) -> Result<ProviderResponse>;
    
    /// Get provider name
    fn name(&self) -> &str;
    
    /// Check if provider is available
    fn is_available(&self) -> bool;
}

/// Get a provider by name
pub fn get_provider(name: &str) -> Result<Box<dyn Provider>> {
    match name {
        "claude-code" | "claude" => Ok(Box::new(claude_code::ClaudeCodeProvider::new())),
        "mock" => Ok(Box::new(MockProvider)),
        _ => Err(anyhow!("Unknown provider: {}. Available: claude-code, mock", name)),
    }
}

/// List available providers
pub fn list_providers() -> Vec<&'static str> {
    vec!["claude-code", "mock"]
}

/// Mock provider for testing
pub struct MockProvider;

#[async_trait]
impl Provider for MockProvider {
    async fn execute(&self, prompt: &str, _context: &Context) -> Result<ProviderResponse> {
        Ok(ProviderResponse {
            success: true,
            output: format!("[MOCK] Would execute: {}", &prompt[..prompt.len().min(100)]),
            errors: vec![],
            warnings: vec!["Using mock provider - no actual changes made".to_string()],
            files_modified: vec![],
        })
    }

    fn name(&self) -> &str {
        "mock"
    }

    fn is_available(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_provider() {
        assert!(get_provider("claude-code").is_ok());
        assert!(get_provider("mock").is_ok());
        assert!(get_provider("unknown").is_err());
    }

    #[tokio::test]
    async fn test_mock_provider() {
        let provider = MockProvider;
        let context = Context {
            working_dir: PathBuf::from("."),
            session_id: "test".to_string(),
            contract_hash: None,
        };
        
        let response = provider.execute("test prompt", &context).await.unwrap();
        assert!(response.success);
        assert!(response.output.contains("MOCK"));
    }
}
