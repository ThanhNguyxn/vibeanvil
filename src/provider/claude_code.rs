//! Claude Code CLI provider adapter

use anyhow::{Context as AnyhowContext, Result};
use async_trait::async_trait;
use std::process::Command;

use super::{Context, Provider, ProviderResponse};

/// Claude Code CLI provider
pub struct ClaudeCodeProvider {
    command: String,
}

impl ClaudeCodeProvider {
    /// Create new Claude Code provider
    pub fn new() -> Self {
        Self {
            command: "claude".to_string(),
        }
    }

    /// Check if claude command is available
    fn check_claude_available(&self) -> bool {
        which::which(&self.command).is_ok()
    }

    /// Build the claude command with arguments
    fn build_command(&self, prompt: &str, context: &Context) -> Command {
        let mut cmd = Command::new(&self.command);

        // Set working directory
        cmd.current_dir(&context.working_dir);

        // Add prompt as argument
        cmd.arg("--print");
        cmd.arg(prompt);

        // Add session context if available
        if let Some(hash) = &context.contract_hash {
            cmd.env("VIBEANVIL_CONTRACT_HASH", hash);
        }
        cmd.env("VIBEANVIL_SESSION_ID", &context.session_id);

        cmd
    }
}

impl Default for ClaudeCodeProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Provider for ClaudeCodeProvider {
    async fn execute(&self, prompt: &str, context: &Context) -> Result<ProviderResponse> {
        if !self.is_available() {
            return Err(anyhow::anyhow!(
                "Claude Code CLI not found. Install with: npm install -g @anthropic-ai/claude-code"
            ));
        }

        let mut cmd = self.build_command(prompt, context);

        let output = cmd
            .output()
            .with_context(|| "Failed to execute claude command")?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        let success = output.status.success();
        let errors = if success {
            vec![]
        } else {
            vec![stderr.clone()]
        };

        Ok(ProviderResponse {
            success,
            output: stdout,
            errors,
            warnings: vec![],
            files_modified: vec![], // Would need to parse claude output for this
        })
    }

    fn name(&self) -> &str {
        "claude-code"
    }

    fn is_available(&self) -> bool {
        self.check_claude_available()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_provider_creation() {
        let provider = ClaudeCodeProvider::new();
        assert_eq!(provider.name(), "claude-code");
    }

    #[test]
    fn test_command_building() {
        let provider = ClaudeCodeProvider::new();
        let context = Context {
            working_dir: PathBuf::from("/tmp"),
            session_id: "test-session".to_string(),
            contract_hash: Some("abc123".to_string()),
        };

        let cmd = provider.build_command("test prompt", &context);
        // Command is built successfully
        assert!(cmd.get_program().to_string_lossy().contains("claude"));
    }
}
