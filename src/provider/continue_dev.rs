//! Continue provider - Open source AI coding assistant
//!
//! Continue is an open-source AI coding assistant with VS Code, JetBrains
//! extensions and a CLI tool with headless mode for automation.
//!
//! Installation:
//! ```bash
//! npm i -g @continuedev/cli
//! ```
//!
//! Configuration:
//! - Model-specific API keys (ANTHROPIC_API_KEY, OPENAI_API_KEY, etc.)
//! - `CONTINUE_MODEL`: Model to use (optional)
//! - `CONTINUE_EXTRA_ARGS`: Additional CLI arguments

use super::{Context, Provider, ProviderResponse};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use std::process::Command;
use std::time::Duration;

/// Continue AI coding assistant provider
pub struct ContinueProvider {
    command: String,
    timeout: Duration,
}

impl Default for ContinueProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl ContinueProvider {
    /// Create a new Continue provider
    pub fn new() -> Self {
        let timeout_secs = std::env::var("CONTINUE_TIMEOUT_SECS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(600);

        Self {
            command: std::env::var("CONTINUE_COMMAND").unwrap_or_else(|_| "cn".to_string()),
            timeout: Duration::from_secs(timeout_secs),
        }
    }

    fn build_command(&self, prompt: &str, context: &Context) -> Command {
        let mut cmd = Command::new(&self.command);

        // Set working directory
        cmd.current_dir(&context.working_dir);

        // Use -p for headless mode (single prompt execution)
        cmd.arg("-p").arg(prompt);

        // Add model if specified
        if let Ok(model) = std::env::var("CONTINUE_MODEL") {
            cmd.arg("--model").arg(model);
        }

        // Add extra arguments
        if let Ok(extra_args) = std::env::var("CONTINUE_EXTRA_ARGS") {
            for arg in extra_args.split_whitespace() {
                cmd.arg(arg);
            }
        }

        cmd
    }
}

#[async_trait]
impl Provider for ContinueProvider {
    async fn execute(&self, prompt: &str, context: &Context) -> Result<ProviderResponse> {
        let mut cmd = self.build_command(prompt, context);

        // Execute with timeout
        let output = tokio::time::timeout(self.timeout, async {
            tokio::task::spawn_blocking(move || cmd.output())
                .await
                .map_err(|e| anyhow!("Task join error: {}", e))?
                .map_err(|e| anyhow!("Failed to execute continue: {}", e))
        })
        .await
        .map_err(|_| anyhow!("Continue timed out after {:?}", self.timeout))??;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        let success = output.status.success();
        let errors = if stderr.is_empty() {
            vec![]
        } else {
            vec![stderr.clone()]
        };

        // Parse modified files from output
        let files_modified = parse_modified_files(&stdout);

        Ok(ProviderResponse {
            success,
            output: if success { stdout } else { stderr },
            errors,
            warnings: vec![],
            files_modified,
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
        "continue"
    }

    fn is_available(&self) -> bool {
        which::which(&self.command).is_ok()
    }
}

/// Parse modified files from Continue output
fn parse_modified_files(output: &str) -> Vec<String> {
    let mut files = Vec::new();
    for line in output.lines() {
        // Look for file operation patterns
        if let Some(path) = line.strip_prefix("Created: ") {
            files.push(path.trim().to_string());
        } else if let Some(path) = line.strip_prefix("Modified: ") {
            files.push(path.trim().to_string());
        } else if let Some(path) = line.strip_prefix("Wrote: ") {
            files.push(path.trim().to_string());
        }
    }
    files
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn cleanup_env() {
        std::env::remove_var("CONTINUE_COMMAND");
        std::env::remove_var("CONTINUE_MODEL");
        std::env::remove_var("CONTINUE_EXTRA_ARGS");
        std::env::remove_var("CONTINUE_TIMEOUT_SECS");
    }

    #[test]
    fn test_continue_provider_new() {
        cleanup_env();
        let provider = ContinueProvider::new();
        assert_eq!(provider.command, "cn");
        assert_eq!(provider.timeout, Duration::from_secs(600));
    }

    #[test]
    fn test_continue_provider_name() {
        cleanup_env();
        let provider = ContinueProvider::new();
        assert_eq!(provider.name(), "continue");
    }

    #[test]
    fn test_continue_build_command() {
        cleanup_env();
        let provider = ContinueProvider::new();
        let context = Context {
            working_dir: PathBuf::from("/test"),
            session_id: "test-session".to_string(),
            contract_hash: None,
        };

        let cmd = provider.build_command("test prompt", &context);
        let args: Vec<_> = cmd.get_args().collect();

        // Should use -p for headless mode
        assert!(args.contains(&std::ffi::OsStr::new("-p")));
        assert!(args.contains(&std::ffi::OsStr::new("test prompt")));
    }

    #[test]
    fn test_parse_modified_files() {
        let output = r#"
Continue: Starting task...
Created: src/main.rs
Modified: src/lib.rs
Wrote: tests/test.rs
Done.
"#;
        let files = parse_modified_files(output);
        assert_eq!(files.len(), 3);
        assert!(files.contains(&"src/main.rs".to_string()));
    }
}
