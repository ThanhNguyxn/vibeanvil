//! Command provider - executes external CLI agents

use anyhow::{Context as AnyhowContext, Result};
use async_trait::async_trait;
use std::io::Write;
use std::process::{Command, Stdio};
use std::time::Duration;
use tempfile::NamedTempFile;

use super::{Context, Provider, ProviderResponse};

/// Mode for passing prompt to external command
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PromptMode {
    /// Pass prompt via stdin
    Stdin,
    /// Pass prompt as final argument
    Arg,
    /// Write prompt to temp file and pass path
    File,
}

impl PromptMode {
    fn from_env() -> Self {
        match std::env::var("VIBEANVIL_PROVIDER_MODE")
            .unwrap_or_default()
            .to_lowercase()
            .as_str()
        {
            "arg" => Self::Arg,
            "file" => Self::File,
            _ => Self::Stdin,
        }
    }
}

/// Command provider for external CLI agents
///
/// Configuration via environment variables:
/// - `VIBEANVIL_PROVIDER_COMMAND`: Command to run (required)
/// - `VIBEANVIL_PROVIDER_ARGS`: Additional arguments (optional, space-separated)
/// - `VIBEANVIL_PROVIDER_MODE`: How to pass prompt - `stdin`, `arg`, or `file` (default: stdin)
/// - `VIBEANVIL_PROVIDER_TIMEOUT`: Timeout in seconds (default: 300)
pub struct CommandProvider {
    command: Option<String>,
    args: Vec<String>,
    mode: PromptMode,
    timeout: Duration,
}

impl CommandProvider {
    pub fn new() -> Self {
        let command = std::env::var("VIBEANVIL_PROVIDER_COMMAND").ok();

        let args: Vec<String> = std::env::var("VIBEANVIL_PROVIDER_ARGS")
            .unwrap_or_default()
            .split_whitespace()
            .map(String::from)
            .collect();

        let mode = PromptMode::from_env();

        let timeout_secs: u64 = std::env::var("VIBEANVIL_PROVIDER_TIMEOUT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(300);

        Self {
            command,
            args,
            mode,
            timeout: Duration::from_secs(timeout_secs),
        }
    }

    /// Check if the configured command exists
    fn check_command_available(&self) -> bool {
        self.command
            .as_ref()
            .is_some_and(|cmd| which::which(cmd).is_ok())
    }

    /// Build and execute the command
    fn execute_command(&self, prompt: &str, context: &Context) -> Result<(bool, String, String)> {
        let cmd_name = self
            .command
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("VIBEANVIL_PROVIDER_COMMAND not set"))?;

        let mut cmd = Command::new(cmd_name);
        cmd.current_dir(&context.working_dir);

        // Add configured args
        for arg in &self.args {
            cmd.arg(arg);
        }

        // Set environment variables (redact sensitive info from logs)
        if let Some(hash) = &context.contract_hash {
            cmd.env("VIBEANVIL_CONTRACT_HASH", hash);
        }
        cmd.env("VIBEANVIL_SESSION_ID", &context.session_id);

        // Handle prompt based on mode
        match self.mode {
            PromptMode::Stdin => {
                cmd.stdin(Stdio::piped());
                cmd.stdout(Stdio::piped());
                cmd.stderr(Stdio::piped());

                let mut child = cmd
                    .spawn()
                    .with_context(|| format!("Failed to spawn command: {}", cmd_name))?;

                if let Some(mut stdin) = child.stdin.take() {
                    stdin
                        .write_all(prompt.as_bytes())
                        .with_context(|| "Failed to write prompt to stdin")?;
                }

                let output = child
                    .wait_with_output()
                    .with_context(|| "Failed to wait for command")?;

                Ok((
                    output.status.success(),
                    String::from_utf8_lossy(&output.stdout).to_string(),
                    String::from_utf8_lossy(&output.stderr).to_string(),
                ))
            }
            PromptMode::Arg => {
                cmd.arg(prompt);
                cmd.stdout(Stdio::piped());
                cmd.stderr(Stdio::piped());

                let output = cmd
                    .output()
                    .with_context(|| format!("Failed to execute command: {}", cmd_name))?;

                Ok((
                    output.status.success(),
                    String::from_utf8_lossy(&output.stdout).to_string(),
                    String::from_utf8_lossy(&output.stderr).to_string(),
                ))
            }
            PromptMode::File => {
                let mut temp_file = NamedTempFile::new()?;
                temp_file.write_all(prompt.as_bytes())?;
                let temp_path = temp_file.path().to_string_lossy().to_string();

                cmd.arg(&temp_path);
                cmd.stdout(Stdio::piped());
                cmd.stderr(Stdio::piped());

                let output = cmd
                    .output()
                    .with_context(|| format!("Failed to execute command: {}", cmd_name))?;

                Ok((
                    output.status.success(),
                    String::from_utf8_lossy(&output.stdout).to_string(),
                    String::from_utf8_lossy(&output.stderr).to_string(),
                ))
            }
        }
    }
}

impl Default for CommandProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Provider for CommandProvider {
    async fn execute(&self, prompt: &str, context: &Context) -> Result<ProviderResponse> {
        if !self.is_available() {
            let cmd = self.command.as_deref().unwrap_or("<not set>");
            return Err(anyhow::anyhow!(
                "Command provider not available.\n\
                 Command '{}' not found.\n\n\
                 Configuration:\n\
                 - Set VIBEANVIL_PROVIDER_COMMAND to your CLI agent command\n\
                 - Optionally set VIBEANVIL_PROVIDER_ARGS for extra arguments\n\
                 - Optionally set VIBEANVIL_PROVIDER_MODE to 'stdin', 'arg', or 'file'\n\n\
                 Example:\n\
                 export VIBEANVIL_PROVIDER_COMMAND=aider\n\
                 export VIBEANVIL_PROVIDER_ARGS=\"--yes --message\"\n\
                 export VIBEANVIL_PROVIDER_MODE=arg",
                cmd
            ));
        }

        let (success, stdout, stderr) = self.execute_command(prompt, context)?;

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
            files_modified: vec![], // Would need to detect from command output
        })
    }

    fn name(&self) -> &str {
        "command"
    }

    fn is_available(&self) -> bool {
        self.check_command_available()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_provider_name() {
        let provider = CommandProvider::new();
        assert_eq!(provider.name(), "command");
    }

    #[test]
    fn test_not_available_without_env() {
        // Clear env var
        std::env::remove_var("VIBEANVIL_PROVIDER_COMMAND");
        let provider = CommandProvider::new();
        assert!(!provider.is_available());
    }

    #[test]
    fn test_prompt_mode_from_env() {
        std::env::set_var("VIBEANVIL_PROVIDER_MODE", "arg");
        assert_eq!(PromptMode::from_env(), PromptMode::Arg);

        std::env::set_var("VIBEANVIL_PROVIDER_MODE", "file");
        assert_eq!(PromptMode::from_env(), PromptMode::File);

        std::env::set_var("VIBEANVIL_PROVIDER_MODE", "stdin");
        assert_eq!(PromptMode::from_env(), PromptMode::Stdin);

        std::env::remove_var("VIBEANVIL_PROVIDER_MODE");
        assert_eq!(PromptMode::from_env(), PromptMode::Stdin);
    }

    #[tokio::test]
    async fn test_execute_without_command_set() {
        std::env::remove_var("VIBEANVIL_PROVIDER_COMMAND");
        let provider = CommandProvider::new();

        let context = Context {
            working_dir: PathBuf::from("."),
            session_id: "test".to_string(),
            contract_hash: None,
        };

        let result = provider.execute("test", &context).await;
        assert!(result.is_err());
    }
}
