//! OpenCode provider - Terminal AI coding assistant (now Crush)
//!
//! OpenCode was a Go-based terminal AI assistant with features like:
//! - TUI with Bubble Tea
//! - Multi-provider support (OpenAI, Anthropic, Google, AWS, Azure, Ollama)
//! - Auto-compact: Summarizes context at 95% token limit
//! - MCP integration for tool use
//! - LSP integration for code intelligence
//! - Session management
//!
//! Note: OpenCode has been archived and succeeded by Crush (github.com/ryboe/crush)
//!
//! Configuration via environment variables:
//! - `OPENCODE_MODEL`: Model to use (default: claude-sonnet-4-20250514)
//! - `ANTHROPIC_API_KEY`: For Claude models
//! - `OPENAI_API_KEY`: For OpenAI models
//! - `LOCAL_ENDPOINT`: For local models (Ollama, llama.cpp)

use anyhow::{Context as AnyhowContext, Result};
use async_trait::async_trait;
use std::io::Write;
use std::process::{Command, Stdio};
use std::time::Duration;

use super::safety::{get_timeout_secs, redact_secrets, truncate_output, MAX_OUTPUT_BYTES};
use super::{Context, Provider, ProviderResponse};

/// OpenCode execution mode
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum OpenCodeMode {
    /// Interactive TUI mode
    Interactive,
    /// Non-interactive prompt mode (--prompt)
    #[default]
    Prompt,
    /// Execute custom command
    Command,
}

/// OpenCode/Crush provider for terminal AI coding
///
/// Features:
/// - Multi-provider backend support
/// - Auto-compact for long conversations
/// - MCP tool integration
/// - Session persistence
pub struct OpenCodeProvider {
    /// Path to executable (opencode or crush)
    command: String,
    /// Execution mode
    mode: OpenCodeMode,
    /// Timeout for operations
    timeout: Duration,
    /// Model to use
    model: Option<String>,
    /// Additional arguments
    extra_args: Vec<String>,
}

impl OpenCodeProvider {
    /// Create new OpenCode provider
    pub fn new() -> Self {
        let model = std::env::var("OPENCODE_MODEL").ok();

        let extra_args: Vec<String> = std::env::var("OPENCODE_EXTRA_ARGS")
            .unwrap_or_default()
            .split_whitespace()
            .map(String::from)
            .collect();

        let timeout_secs = get_timeout_secs();

        // Try crush first (successor), then opencode
        let command = if which::which("crush").is_ok() {
            "crush".to_string()
        } else {
            "opencode".to_string()
        };

        Self {
            command,
            mode: OpenCodeMode::Prompt,
            timeout: Duration::from_secs(timeout_secs),
            model,
            extra_args,
        }
    }

    /// Create provider with interactive mode
    pub fn interactive() -> Self {
        let mut provider = Self::new();
        provider.mode = OpenCodeMode::Interactive;
        provider
    }

    /// Check if opencode/crush is installed
    fn check_available(&self) -> bool {
        which::which(&self.command).is_ok()
    }

    /// Build command with appropriate flags
    fn build_command(&self, prompt: &str, context: &Context) -> Command {
        let mut cmd = Command::new(&self.command);

        // Working directory
        cmd.current_dir(&context.working_dir);

        // Model selection
        if let Some(ref model) = self.model {
            cmd.arg("--model").arg(model);
        }

        // Mode-specific flags
        match self.mode {
            OpenCodeMode::Prompt => {
                cmd.arg("--prompt").arg(prompt);
            }
            OpenCodeMode::Command => {
                cmd.arg("--command").arg(prompt);
            }
            OpenCodeMode::Interactive => {
                // Interactive mode uses TUI
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

    /// Execute and capture output
    fn execute_command(&self, prompt: &str, context: &Context) -> Result<(bool, String, String)> {
        let mut cmd = self.build_command(prompt, context);

        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        if self.mode == OpenCodeMode::Interactive {
            cmd.stdin(Stdio::piped());
        }

        let mut child = cmd.spawn().with_context(|| {
            format!(
                "Failed to spawn {}. Install from: https://github.com/opencode-ai/opencode",
                self.command
            )
        })?;

        // For interactive mode, send prompt and exit
        if self.mode == OpenCodeMode::Interactive {
            if let Some(mut stdin) = child.stdin.take() {
                writeln!(stdin, "{}", prompt)?;
                writeln!(stdin, "/quit")?;
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
                            "OpenCode timed out after {} seconds.\n\
                             To increase: export VIBEANVIL_PROVIDER_TIMEOUT_SECS=<seconds>",
                            self.timeout.as_secs()
                        ));
                    }
                    std::thread::sleep(Duration::from_millis(100));
                }
                Err(e) => {
                    return Err(anyhow::anyhow!("Failed to wait for process: {}", e));
                }
            }
        }
    }
}

impl Default for OpenCodeProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Provider for OpenCodeProvider {
    async fn execute(&self, prompt: &str, context: &Context) -> Result<ProviderResponse> {
        if !self.is_available() {
            return Err(anyhow::anyhow!(
                "OpenCode/Crush not found.\n\n\
                 OpenCode has been archived. Install its successor Crush:\n  \
                 go install github.com/ryboe/crush@latest\n\n\
                 Or use the command provider with any CLI agent:\n  \
                 export VIBEANVIL_PROVIDER_COMMAND=<your-agent>\n\n\
                 Alternative terminal agents:\n  \
                 • aider: pip install aider-chat\n  \
                 • claude: npm install -g @anthropic-ai/claude-code"
            ));
        }

        let (success, stdout, stderr) = self.execute_command(prompt, context)?;

        // Redact secrets and truncate
        let stdout = redact_secrets(&stdout);
        let stderr = redact_secrets(&stderr);
        let stdout = truncate_output(&stdout, MAX_OUTPUT_BYTES);
        let stderr_truncated = truncate_output(&stderr, MAX_OUTPUT_BYTES);

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
            files_modified: vec![],
        })
    }

    fn name(&self) -> &str {
        "opencode"
    }

    fn is_available(&self) -> bool {
        self.check_available()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_provider_creation() {
        let provider = OpenCodeProvider::new();
        assert_eq!(provider.name(), "opencode");
    }

    #[test]
    fn test_interactive_mode() {
        let provider = OpenCodeProvider::interactive();
        assert_eq!(provider.mode, OpenCodeMode::Interactive);
    }

    #[test]
    fn test_command_building() {
        let provider = OpenCodeProvider::new();
        let context = Context {
            working_dir: PathBuf::from("/tmp/test"),
            session_id: "test-session".to_string(),
            contract_hash: None,
        };

        let cmd = provider.build_command("Fix bug", &context);
        assert!(cmd.get_program() == "crush" || cmd.get_program() == "opencode");
    }
}
