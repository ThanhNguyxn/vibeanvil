//! Command provider - executes external CLI agents with safety hardening
//!
//! Features:
//! - Timeout protection with process kill
//! - Output capture with size limits
//! - Secret redaction in outputs
//! - Actionable error messages

use anyhow::{Context as AnyhowContext, Result};
use async_trait::async_trait;
use std::io::Write;
use std::process::{Command, Stdio};
use std::time::Duration;
use tempfile::NamedTempFile;

use super::safety::{
    get_timeout_secs, redact_secrets, tail_lines, truncate_output, ERROR_TAIL_BYTES,
    ERROR_TAIL_LINES, MAX_OUTPUT_BYTES,
};
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
/// - `VIBEANVIL_PROVIDER_TIMEOUT_SECS`: Timeout in seconds (default: 600)
///
/// Safety features:
/// - Automatic timeout with process termination
/// - Output size limits (256KB per stream)
/// - Secret redaction in all outputs
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
        let timeout_secs = get_timeout_secs();

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

    /// Build and execute the command with timeout
    fn execute_command(&self, prompt: &str, context: &Context) -> Result<(bool, String, String)> {
        let cmd_name = self.command.as_ref().ok_or_else(|| {
            anyhow::anyhow!(
                "VIBEANVIL_PROVIDER_COMMAND not set.\n\n\
                 To use the command provider, set:\n  \
                 export VIBEANVIL_PROVIDER_COMMAND=<your-agent>\n\n\
                 Example:\n  \
                 export VIBEANVIL_PROVIDER_COMMAND=aider\n  \
                 export VIBEANVIL_PROVIDER_ARGS=\"--yes\"\n  \
                 export VIBEANVIL_PROVIDER_MODE=stdin"
            )
        })?;

        let mut cmd = Command::new(cmd_name);
        cmd.current_dir(&context.working_dir);

        // Add configured args
        for arg in &self.args {
            cmd.arg(arg);
        }

        // Set environment variables
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

                // Wait with timeout
                let start = std::time::Instant::now();
                loop {
                    match child.try_wait() {
                        Ok(Some(status)) => {
                            // Process finished
                            let output = child.wait_with_output()?;
                            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                            return Ok((status.success(), stdout, stderr));
                        }
                        Ok(None) => {
                            // Still running - check timeout
                            if start.elapsed() > self.timeout {
                                // Kill the process
                                let _ = child.kill();
                                let _ = child.wait(); // Reap zombie
                                return Err(anyhow::anyhow!(
                                    "Provider command timed out after {} seconds.\n\n\
                                     The command '{}' did not complete in time and was terminated.\n\n\
                                     To increase the timeout, set:\n  \
                                     export VIBEANVIL_PROVIDER_TIMEOUT_SECS=<seconds>\n\n\
                                     Current timeout: {} seconds",
                                    self.timeout.as_secs(),
                                    cmd_name,
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
            PromptMode::Arg => {
                cmd.arg(prompt);
                cmd.stdout(Stdio::piped());
                cmd.stderr(Stdio::piped());

                let mut child = cmd
                    .spawn()
                    .with_context(|| format!("Failed to spawn command: {}", cmd_name))?;

                // Wait with timeout (same loop as above)
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
                                    "Provider command timed out after {} seconds.\n\n\
                                     The command '{}' did not complete in time and was terminated.\n\n\
                                     To increase the timeout, set:\n  \
                                     export VIBEANVIL_PROVIDER_TIMEOUT_SECS=<seconds>",
                                    self.timeout.as_secs(),
                                    cmd_name
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
            PromptMode::File => {
                let mut temp_file = NamedTempFile::new()?;
                temp_file.write_all(prompt.as_bytes())?;
                let temp_path = temp_file.path().to_string_lossy().to_string();

                cmd.arg(&temp_path);
                cmd.stdout(Stdio::piped());
                cmd.stderr(Stdio::piped());

                let mut child = cmd
                    .spawn()
                    .with_context(|| format!("Failed to spawn command: {}", cmd_name))?;

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
                                    "Provider command timed out after {} seconds.\n\n\
                                     The command '{}' did not complete in time and was terminated.\n\n\
                                     To increase the timeout, set:\n  \
                                     export VIBEANVIL_PROVIDER_TIMEOUT_SECS=<seconds>",
                                    self.timeout.as_secs(),
                                    cmd_name
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
                "Command provider not available.\n\n\
                 Command '{}' not found in PATH.\n\n\
                 Configuration:\n  \
                 1. Set VIBEANVIL_PROVIDER_COMMAND to your CLI agent command\n  \
                 2. Optionally set VIBEANVIL_PROVIDER_ARGS for extra arguments\n  \
                 3. Optionally set VIBEANVIL_PROVIDER_MODE to 'stdin', 'arg', or 'file'\n\n\
                 Example:\n  \
                 export VIBEANVIL_PROVIDER_COMMAND=aider\n  \
                 export VIBEANVIL_PROVIDER_ARGS=\"--yes --message\"\n  \
                 export VIBEANVIL_PROVIDER_MODE=arg",
                cmd
            ));
        }

        let (success, stdout, stderr) = self.execute_command(prompt, context)?;

        // Apply safety transformations
        let stdout_safe = redact_secrets(&truncate_output(&stdout, MAX_OUTPUT_BYTES));
        let stderr_safe = redact_secrets(&truncate_output(&stderr, MAX_OUTPUT_BYTES));

        let errors = if success {
            vec![]
        } else {
            // Show tail of stderr for debugging
            let stderr_tail = tail_lines(&stderr_safe, ERROR_TAIL_LINES, ERROR_TAIL_BYTES);
            vec![format!(
                "Command failed.\n\nStderr (last {} lines):\n{}",
                ERROR_TAIL_LINES, stderr_tail
            )]
        };

        Ok(ProviderResponse {
            success,
            output: stdout_safe,
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
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("not available") || err_msg.contains("not found"));
    }

    #[tokio::test]
    #[cfg(unix)]
    async fn test_timeout_unix() {
        std::env::set_var("VIBEANVIL_PROVIDER_COMMAND", "sh");
        std::env::set_var("VIBEANVIL_PROVIDER_ARGS", "-c");
        std::env::set_var("VIBEANVIL_PROVIDER_MODE", "arg");
        std::env::set_var("VIBEANVIL_PROVIDER_TIMEOUT_SECS", "1");

        let provider = CommandProvider::new();
        let context = Context {
            working_dir: PathBuf::from("."),
            session_id: "test".to_string(),
            contract_hash: None,
        };

        // Sleep for 3 seconds should timeout after 1 second
        let result = provider.execute("sleep 3; echo done", &context).await;

        // Clean up
        std::env::remove_var("VIBEANVIL_PROVIDER_COMMAND");
        std::env::remove_var("VIBEANVIL_PROVIDER_ARGS");
        std::env::remove_var("VIBEANVIL_PROVIDER_MODE");
        std::env::remove_var("VIBEANVIL_PROVIDER_TIMEOUT_SECS");

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("timed out"));
    }

    #[tokio::test]
    #[cfg(windows)]
    async fn test_timeout_windows() {
        std::env::set_var("VIBEANVIL_PROVIDER_COMMAND", "powershell");
        std::env::set_var("VIBEANVIL_PROVIDER_ARGS", "-Command");
        std::env::set_var("VIBEANVIL_PROVIDER_MODE", "arg");
        std::env::set_var("VIBEANVIL_PROVIDER_TIMEOUT_SECS", "1");

        let provider = CommandProvider::new();
        let context = Context {
            working_dir: PathBuf::from("."),
            session_id: "test".to_string(),
            contract_hash: None,
        };

        // Sleep for 3 seconds should timeout after 1 second
        let result = provider
            .execute("Start-Sleep -Seconds 3; Write-Output done", &context)
            .await;

        // Clean up
        std::env::remove_var("VIBEANVIL_PROVIDER_COMMAND");
        std::env::remove_var("VIBEANVIL_PROVIDER_ARGS");
        std::env::remove_var("VIBEANVIL_PROVIDER_MODE");
        std::env::remove_var("VIBEANVIL_PROVIDER_TIMEOUT_SECS");

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("timed out"));
    }

    #[tokio::test]
    #[cfg(unix)]
    async fn test_successful_command_unix() {
        std::env::set_var("VIBEANVIL_PROVIDER_COMMAND", "echo");
        std::env::set_var("VIBEANVIL_PROVIDER_MODE", "arg");
        std::env::remove_var("VIBEANVIL_PROVIDER_ARGS");

        let provider = CommandProvider::new();
        let context = Context {
            working_dir: PathBuf::from("."),
            session_id: "test".to_string(),
            contract_hash: None,
        };

        let result = provider.execute("hello world", &context).await;

        std::env::remove_var("VIBEANVIL_PROVIDER_COMMAND");
        std::env::remove_var("VIBEANVIL_PROVIDER_MODE");

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.success);
        assert!(response.output.contains("hello world"));
    }
}
