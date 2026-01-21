//! Provider plugins for AI coding assistants

pub mod aider;
pub mod claude_code;
pub mod cline;
pub mod cody;
pub mod command;
pub mod continue_dev;
pub mod copilot;
pub mod cursor;
pub mod gemini;
pub mod goose;
pub mod human;
pub mod jetbrains;
pub mod kiro;
pub mod ollama;
pub mod opencode;
pub mod patch;
pub mod safety;
pub mod supermaven;
pub mod tabby;
pub mod trae;
pub mod windsurf;
pub mod zed;

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

    /// Generate a commit message based on a diff
    async fn generate_commit_message(&self, diff: &str, _context: &Context) -> Result<String> {
        // Default implementation: simple message
        Ok(format!(
            "update: changes based on diff of length {}",
            diff.len()
        ))
    }

    /// Get provider name
    fn name(&self) -> &str;

    /// Check if provider is available
    fn is_available(&self) -> bool;
}

/// Get a provider by name
pub fn get_provider(name: &str) -> Result<Box<dyn Provider>> {
    match name {
        // Claude providers
        "claude-code" | "claude" => Ok(Box::new(claude_code::ClaudeCodeProvider::new())),
        
        // IDE AI assistants (with CLI)
        "cursor" => Ok(Box::new(cursor::CursorProvider::new())),
        "cline" => Ok(Box::new(cline::ClineProvider::new())),
        "continue" | "cn" => Ok(Box::new(continue_dev::ContinueProvider::new())),
        "cody" => Ok(Box::new(cody::CodyProvider::new())),
        
        // IDE AI assistants (prompt-based, no CLI)
        "copilot" | "github-copilot" => Ok(Box::new(copilot::CopilotProvider::new())),
        "zed" | "zed-ai" => Ok(Box::new(zed::ZedProvider::new())),
        "windsurf" => Ok(Box::new(windsurf::WindsurfProvider::new())),
        "trae" => Ok(Box::new(trae::TraeProvider::new())),
        "jetbrains" | "intellij" | "idea" => Ok(Box::new(jetbrains::JetbrainsProvider::new())),
        "supermaven" => Ok(Box::new(supermaven::SupermavenProvider::new())),
        "gemini-assist" | "gemini-code" => Ok(Box::new(gemini::GeminiProvider::new())),
        
        // Terminal AI agents
        "aider" => Ok(Box::new(aider::AiderProvider::new())),
        "opencode" | "crush" => Ok(Box::new(opencode::OpenCodeProvider::new())),
        "goose" => Ok(Box::new(goose::GooseProvider::new())),
        
        // Cloud AI assistants
        "kiro" | "amazon-q" => Ok(Box::new(kiro::KiroProvider::new())),
        
        // Self-hosted AI
        "tabby" => Ok(Box::new(tabby::TabbyProvider::new())),
        
        // Local AI providers - Ollama with dynamic model selection
        "ollama" => Ok(Box::new(ollama::OllamaProvider::new())),
        
        // Ollama model shortcuts (no version hardcoding)
        // General purpose
        s if s.starts_with("ollama/") => {
            let model = s.strip_prefix("ollama/").unwrap();
            Ok(Box::new(ollama::OllamaProvider::with_model(model)))
        }
        
        // Tabby with specific model
        s if s.starts_with("tabby/") => {
            let model = s.strip_prefix("tabby/").unwrap();
            Ok(Box::new(tabby::TabbyProvider::with_model(model)))
        }
        
        // Popular model shortcuts (Ollama)
        "llama" | "llama3" => Ok(Box::new(ollama::OllamaProvider::with_model("llama3.2"))),
        "codellama" => Ok(Box::new(ollama::OllamaProvider::with_model("codellama"))),
        "deepseek" | "deepseek-coder" => Ok(Box::new(ollama::OllamaProvider::with_model("deepseek-coder-v2"))),
        "qwen" | "qwen2" => Ok(Box::new(ollama::OllamaProvider::with_model("qwen2.5-coder"))),
        "mistral" => Ok(Box::new(ollama::OllamaProvider::with_model("mistral"))),
        "mixtral" => Ok(Box::new(ollama::OllamaProvider::with_model("mixtral"))),
        "phi" | "phi3" => Ok(Box::new(ollama::OllamaProvider::with_model("phi3"))),
        "gemma" | "gemma2" => Ok(Box::new(ollama::OllamaProvider::with_model("gemma2"))),
        "starcoder" | "starcoder2" => Ok(Box::new(ollama::OllamaProvider::with_model("starcoder2"))),
        "codegemma" => Ok(Box::new(ollama::OllamaProvider::with_model("codegemma"))),
        "wizardcoder" => Ok(Box::new(ollama::OllamaProvider::with_model("wizardcoder"))),
        "codestral" => Ok(Box::new(ollama::OllamaProvider::with_model("codestral"))),
        "granite-code" => Ok(Box::new(ollama::OllamaProvider::with_model("granite-code"))),
        "yi-coder" => Ok(Box::new(ollama::OllamaProvider::with_model("yi-coder"))),
        "stable-code" => Ok(Box::new(ollama::OllamaProvider::with_model("stable-code"))),
        
        // Generic providers
        "human" => Ok(Box::new(human::HumanProvider::new())),
        "command" | "cmd" => Ok(Box::new(command::CommandProvider::new())),
        "patch" | "diff" => Ok(Box::new(patch::PatchProvider::new())),
        "mock" => Ok(Box::new(MockProvider)),
        
        _ => Err(anyhow!(
            "Unknown provider: '{}'\n\n\
             Available providers:\n\n\
             IDE AI (with CLI):\n  \
             • cursor       - Cursor AI editor\n  \
             • cline        - Cline autonomous agent\n  \
             • continue     - Continue dev assistant\n  \
             • cody         - Sourcegraph Cody\n\n\
             IDE AI (prompt-based):\n  \
             • copilot      - GitHub Copilot\n  \
             • zed          - Zed AI\n  \
             • windsurf     - Codeium Windsurf\n  \
             • trae         - ByteDance Trae\n  \
             • jetbrains    - JetBrains AI\n  \
             • supermaven   - Supermaven\n  \
             • gemini-assist - Google Gemini\n\n\
             Terminal Agents:\n  \
             • claude-code  - Claude Code CLI\n  \
             • aider        - AI pair programming\n  \
             • opencode     - Terminal TUI\n  \
             • goose        - Block's AI agent\n\n\
             Cloud AI:\n  \
             • kiro         - AWS Kiro (Amazon Q)\n\n\
             Self-Hosted AI:\n  \
             • tabby        - Tabby (tabby/<model>)\n  \
             • ollama       - Ollama (ollama/<model>)\n\n\
             Model Shortcuts (Ollama):\n  \
             • llama, codellama, deepseek, qwen, mistral, mixtral\n  \
             • phi, gemma, starcoder, codegemma, wizardcoder\n  \
             • codestral, granite-code, yi-coder, stable-code\n\n\
             Generic:\n  \
             • human   - Generic IDE prompt\n  \
             • command - External CLI agent\n  \
             • patch   - Apply diffs\n\n\
             Run 'vibeanvil providers' for setup instructions.",
            name
        )),
    }
}

/// List available providers
pub fn list_providers() -> Vec<&'static str> {
    vec![
        // IDE AI (with CLI)
        "cursor",
        "cline",
        "continue",
        "cody",
        // IDE AI (prompt-based)
        "copilot",
        "zed",
        "windsurf",
        "trae",
        "jetbrains",
        "supermaven",
        "gemini-assist",
        // Terminal agents
        "claude-code",
        "aider",
        "opencode",
        "goose",
        // Cloud AI
        "kiro",
        // Self-hosted AI
        "tabby",
        "tabby/<model>",
        // Local AI (Ollama)
        "ollama",
        "ollama/<model>",
        // Model shortcuts (popular models)
        "llama",
        "codellama",
        "deepseek",
        "qwen",
        "mistral",
        "mixtral",
        "phi",
        "gemma",
        "starcoder",
        "codegemma",
        "wizardcoder",
        "codestral",
        "granite-code",
        "yi-coder",
        "stable-code",
        // Generic
        "human",
        "command",
        "patch",
        "mock",
    ]
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

    #[test]
    fn test_get_provider_ide_assistants() {
        // Test IDE AI assistants (with CLI)
        assert!(get_provider("cursor").is_ok());
        assert!(get_provider("cline").is_ok());
        assert!(get_provider("continue").is_ok());
        assert!(get_provider("cn").is_ok()); // continue alias
        assert!(get_provider("cody").is_ok());
    }

    #[test]
    fn test_get_provider_ide_prompt_based() {
        // Test IDE AI assistants (prompt-based, no CLI)
        assert!(get_provider("copilot").is_ok());
        assert!(get_provider("github-copilot").is_ok()); // alias
        assert!(get_provider("zed").is_ok());
        assert!(get_provider("zed-ai").is_ok()); // alias
        assert!(get_provider("windsurf").is_ok());
        assert!(get_provider("trae").is_ok());
        assert!(get_provider("jetbrains").is_ok());
        assert!(get_provider("intellij").is_ok()); // alias
        assert!(get_provider("idea").is_ok()); // alias
        assert!(get_provider("supermaven").is_ok());
        assert!(get_provider("gemini-assist").is_ok());
        assert!(get_provider("gemini-code").is_ok()); // alias
    }

    #[test]
    fn test_get_provider_terminal_agents() {
        // Test terminal AI agents
        assert!(get_provider("claude-code").is_ok());
        assert!(get_provider("claude").is_ok()); // alias
        assert!(get_provider("aider").is_ok());
        assert!(get_provider("opencode").is_ok());
        assert!(get_provider("crush").is_ok()); // opencode alias
        assert!(get_provider("goose").is_ok());
    }

    #[test]
    fn test_get_provider_cloud_and_selfhosted() {
        // Test cloud and self-hosted providers
        assert!(get_provider("kiro").is_ok());
        assert!(get_provider("amazon-q").is_ok()); // kiro alias
        assert!(get_provider("tabby").is_ok());
        assert!(get_provider("tabby/StarCoder-7B").is_ok());
    }

    #[test]
    fn test_get_provider_ollama_dynamic() {
        // Test dynamic ollama/<model> syntax
        assert!(get_provider("ollama/llama3.2").is_ok());
        assert!(get_provider("ollama/codellama").is_ok());
        assert!(get_provider("ollama/deepseek-coder-v2:16b").is_ok());
        assert!(get_provider("ollama/qwen2.5-coder:7b").is_ok());
        assert!(get_provider("ollama/any-model-name").is_ok());
    }

    #[test]
    fn test_get_provider_model_shortcuts() {
        // Test model shortcuts (no version hardcoding)
        assert!(get_provider("llama").is_ok());
        assert!(get_provider("codellama").is_ok());
        assert!(get_provider("deepseek").is_ok());
        assert!(get_provider("qwen").is_ok());
        assert!(get_provider("mistral").is_ok());
        assert!(get_provider("mixtral").is_ok());
        assert!(get_provider("phi").is_ok());
        assert!(get_provider("gemma").is_ok());
        assert!(get_provider("starcoder").is_ok());
        assert!(get_provider("codegemma").is_ok());
        assert!(get_provider("wizardcoder").is_ok());
        assert!(get_provider("codestral").is_ok());
        assert!(get_provider("granite-code").is_ok());
        assert!(get_provider("yi-coder").is_ok());
        assert!(get_provider("stable-code").is_ok());
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
