//! Aider provider - AI pair programming in your terminal
//!
//! Aider (https://github.com/Aider-AI/aider) is a powerful terminal-based
//! AI coding assistant with features like:
//! - RepoMap: Automatic codebase mapping
//! - Auto-commit: Git commits with AI messages
//! - Watch mode: Monitor files for AI instructions
//! - Multi-model: Claude, GPT-4, DeepSeek, local models
//!
//! Configuration via environment variables:
//! - `AIDER_MODEL`: Model to use (default: claude-3-sonnet-20240229)
//! - `ANTHROPIC_API_KEY`: For Claude models
//! - `OPENAI_API_KEY`: For OpenAI models
//! - `AIDER_AUTO_COMMITS`: Enable auto-commits (default: true)
//! - `AIDER_MAP_TOKENS`: Tokens for repo map (default: 1024)

use anyhow::{Context as AnyhowContext, Result};
use async_trait::async_trait;
use std::io::Write;
use std::process::{Command, Stdio};
use std::time::Duration;

use super::safety::{get_timeout_secs, redact_secrets, truncate_output, MAX_OUTPUT_BYTES};
use super::{Context, Provider, ProviderResponse};

/// Aider execution mode
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AiderMode {
    /// Interactive chat mode
    Chat,
    /// Single message mode (--message)
    Message,
    /// Watch mode for file-based instructions
    Watch,
}

impl Default for AiderMode {
    fn default() -> Self {
        Self::Message
    }
}

/// Aider provider for AI pair programming
///
/// Features:
/// - Uses aider's RepoMap for context-aware edits
/// - Supports auto-commit with meaningful messages
/// - Can use local models via Ollama
pub struct AiderProvider {
    /// Path to aider executable
    command: String,
    /// Execution mode
    mode: AiderMode,
    /// Timeout for operations
    timeout: Duration,
    /// Enable auto-commits
    auto_commits: bool,
    /// Model to use
    model: Option<String>,
    /// Additional arguments
    extra_args: Vec<String>,
}

impl AiderProvider {
    /// Create new Aider provider with default settings
    pub fn new() -> Self {
        let auto_commits = std::env::var("AIDER_AUTO_COMMITS")
            .map(|v| v != "false" && v != "0")
            .unwrap_or(true);

        let model = std::env::var("AIDER_MODEL").ok();

        let extra_args: Vec<String> = std::env::var("AIDER_EXTRA_ARGS")
            .unwrap_or_default()
            .split_whitespace()
            .map(String::from)
            .collect();

        let timeout_secs = get_timeout_secs();

        Self {
            command: "aider".to_string(),
            mode: AiderMode::Message,
            timeout: Duration::from_secs(timeout_secs),
            auto_commits,
            model,
            extra_args,
        }
    }

    /// Create provider with watch mode enabled
    pub fn with_watch_mode() -> Self {
        let mut provider = Self::new();
        provider.mode = AiderMode::Watch;
        provider
    }

    /// Check if aider is installed
    fn check_aider_available(&self) -> bool {
        which::which(&self.command).is_ok()
    }

    /// Build aider command with appropriate flags
    fn build_command(&self, prompt: &str, context: &Context) -> Command {
        let mut cmd = Command::new(&self.command);

        // Working directory
        cmd.current_dir(&context.working_dir);

        // Model selection
        if let Some(ref model) = self.model {
            cmd.arg("--model").arg(model);
        }

        // Auto-commit settings
        if self.auto_commits {
            cmd.arg("--auto-commits");
        } else {
            cmd.arg("--no-auto-commits");
        }

        // Mode-specific flags
        match self.mode {
            AiderMode::Message => {
                cmd.arg("--message").arg(prompt);
                cmd.arg("--yes"); // Auto-accept changes
            }
            AiderMode::Watch => {
                cmd.arg("--watch");
            }
            AiderMode::Chat => {
                // Interactive mode - prompt via stdin
            }
        }

        // Extra args
        for arg in &self.extra_args {
            cmd.arg(arg);
        }

        // VibeAnvil context
        if let Some(hash) = &context.contract_hash {
            cmd.env("VIBEANVIL_CONTRACT_HASH", hash);
        }
        cmd.env("VIBEANVIL_SESSION_ID", &context.session_id);

        cmd
    }

    /// Execute aider and capture output
    fn execute_aider(&self, prompt: &str, context: &Context) -> Result<(bool, String, String)> {
        let mut cmd = self.build_command(prompt, context);

        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        if self.mode == AiderMode::Chat {
            cmd.stdin(Stdio::piped());
        }

        let mut child = cmd.spawn().with_context(|| {
            "Failed to spawn aider. Is it installed? Run: pip install aider-chat"
        })?;

        // For chat mode, write prompt to stdin
        if self.mode == AiderMode::Chat {
            if let Some(mut stdin) = child.stdin.take() {
                writeln!(stdin, "{}", prompt)?;
                writeln!(stdin, "/exit")?; // Exit after processing
            }
        }

        // Wait with timeout
        let start = std::time::Instant::now();
        loop {
            match child.try_wait() {
                Ok(Some(status)) => {
                    let output = child.wait_with_output()?;
                    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                    return Ok((status.success(), stdout, stderr));
                }
                Ok(None) => {
                    if start.elapsed() > self.timeout {
                        let _ = child.kill();
                        let _ = child.wait();
                        return Err(anyhow::anyhow!(
                            "Aider timed out after {} seconds.\n\n\
                             The operation did not complete in time.\n\
                             To increase timeout: export VIBEANVIL_PROVIDER_TIMEOUT_SECS=<seconds>",
                            self.timeout.as_secs()
                        ));
                    }
                    std::thread::sleep(Duration::from_millis(100));
                }
                Err(e) => {
                    return Err(anyhow::anyhow!("Failed to wait for aider: {}", e));
                }
            }
        }
    }

    /// Parse aider output to extract modified files
    fn parse_modified_files(&self, output: &str) -> Vec<String> {
        let mut files = Vec::new();

        for line in output.lines() {
            // Aider outputs "Wrote <file>" when it modifies files
            if let Some(file) = line.strip_prefix("Wrote ") {
                files.push(file.trim().to_string());
            }
            // Also check for "Added <file>" pattern
            if let Some(file) = line.strip_prefix("Added ") {
                files.push(file.trim().to_string());
            }
        }

        files
    }
}

impl Default for AiderProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Provider for AiderProvider {
    async fn execute(&self, prompt: &str, context: &Context) -> Result<ProviderResponse> {
        if !self.is_available() {
            return Err(anyhow::anyhow!(
                "Aider not found.\n\n\
                 Install with: pip install aider-chat\n\
                 Or: pipx install aider-chat\n\n\
                 Then set your API key:\n  \
                 export ANTHROPIC_API_KEY=<your-key>  # For Claude\n  \
                 export OPENAI_API_KEY=<your-key>     # For OpenAI\n\n\
                 For local models, install Ollama and run:\n  \
                 aider --model ollama/llama3.2"
            ));
        }

        let (success, stdout, stderr) = self.execute_aider(prompt, context)?;

        // Redact secrets and truncate output
        let stdout = redact_secrets(&stdout);
        let stderr = redact_secrets(&stderr);
        let stdout = truncate_output(&stdout, MAX_OUTPUT_BYTES);
        let stderr_truncated = truncate_output(&stderr, MAX_OUTPUT_BYTES);

        let files_modified = self.parse_modified_files(&stdout);

        let errors = if success {
            vec![]
        } else {
            vec![stderr_truncated]
        };

        Ok(ProviderResponse {
            success,
            output: stdout,
            errors,
            warnings: vec![],
            files_modified,
        })
    }

    async fn generate_commit_message(&self, diff: &str, context: &Context) -> Result<String> {
        // Use aider's built-in commit message generation
        let prompt = format!(
            "Generate a concise git commit message for these changes. \
             Follow conventional commits format (type: description). \
             Be specific about what changed.\n\n```diff\n{}\n```",
            diff
        );

        let response = self.execute(&prompt, context).await?;

        if response.success {
            // Extract first line as commit message
            Ok(response
                .output
                .lines()
                .find(|l| !l.is_empty())
                .unwrap_or("update: apply changes")
                .to_string())
        } else {
            Ok(format!("update: changes ({} bytes)", diff.len()))
        }
    }

    fn name(&self) -> &str {
        "aider"
    }

    fn is_available(&self) -> bool {
        self.check_aider_available()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_provider_creation() {
        let provider = AiderProvider::new();
        assert_eq!(provider.name(), "aider");
        assert!(provider.auto_commits);
    }

    #[test]
    fn test_command_building() {
        let provider = AiderProvider::new();
        let context = Context {
            working_dir: PathBuf::from("/tmp/test"),
            session_id: "test-session".to_string(),
            contract_hash: Some("abc123".to_string()),
        };

        let cmd = provider.build_command("Fix the bug", &context);
        // Command should be configured correctly
        assert_eq!(cmd.get_program(), "aider");
    }

    #[test]
    fn test_parse_modified_files() {
        let provider = AiderProvider::new();
        let output = "Thinking...\nWrote src/main.rs\nAdded tests/test.rs\nDone.";
        let files = provider.parse_modified_files(output);
        assert_eq!(files, vec!["src/main.rs", "tests/test.rs"]);
    }

    #[test]
    fn test_watch_mode() {
        let provider = AiderProvider::with_watch_mode();
        assert_eq!(provider.mode, AiderMode::Watch);
    }
}
