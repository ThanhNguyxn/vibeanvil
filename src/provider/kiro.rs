//! Kiro provider - AWS's AI coding assistant (formerly Amazon Q)
//!
//! Kiro is AWS's AI-powered coding assistant with CLI support,
//! MCP integration, and enterprise features.
//!
//! Installation:
//! - macOS/Linux: `curl -fsSL https://cli.kiro.dev/install | bash`
//! - Ubuntu: Download .deb from AWS
//!
//! Configuration:
//! - AWS authentication (via `kiro-cli login`)
//! - `KIRO_EXTRA_ARGS`: Additional CLI arguments

use super::{Context, Provider, ProviderResponse};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use std::process::Command;
use std::time::Duration;

/// Kiro AI coding assistant provider (AWS)
pub struct KiroProvider {
    command: String,
    timeout: Duration,
}

impl Default for KiroProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl KiroProvider {
    /// Create a new Kiro provider
    pub fn new() -> Self {
        let timeout_secs = std::env::var("KIRO_TIMEOUT_SECS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(600);

        Self {
            command: std::env::var("KIRO_COMMAND").unwrap_or_else(|_| "kiro-cli".to_string()),
            timeout: Duration::from_secs(timeout_secs),
        }
    }

    fn build_command(&self, prompt: &str, context: &Context) -> Command {
        let mut cmd = Command::new(&self.command);

        // Set working directory
        cmd.current_dir(&context.working_dir);

        // Use chat subcommand with prompt
        cmd.arg("chat");
        cmd.arg("--message").arg(prompt);

        // Add extra arguments
        if let Ok(extra_args) = std::env::var("KIRO_EXTRA_ARGS") {
            for arg in extra_args.split_whitespace() {
                cmd.arg(arg);
            }
        }

        cmd
    }
}

#[async_trait]
impl Provider for KiroProvider {
    async fn execute(&self, prompt: &str, context: &Context) -> Result<ProviderResponse> {
        let mut cmd = self.build_command(prompt, context);

        // Execute with timeout
        let output = tokio::time::timeout(self.timeout, async {
            tokio::task::spawn_blocking(move || cmd.output())
                .await
                .map_err(|e| anyhow!("Task join error: {}", e))?
                .map_err(|e| anyhow!("Failed to execute kiro-cli: {}", e))
        })
        .await
        .map_err(|_| anyhow!("Kiro timed out after {:?}", self.timeout))??;

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
        "kiro"
    }

    fn is_available(&self) -> bool {
        which::which(&self.command).is_ok()
    }
}

/// Parse modified files from Kiro output
fn parse_modified_files(output: &str) -> Vec<String> {
    let mut files = Vec::new();
    for line in output.lines() {
        if let Some(path) = line.strip_prefix("Created: ") {
            files.push(path.trim().to_string());
        } else if let Some(path) = line.strip_prefix("Modified: ") {
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
        std::env::remove_var("KIRO_COMMAND");
        std::env::remove_var("KIRO_EXTRA_ARGS");
        std::env::remove_var("KIRO_TIMEOUT_SECS");
    }

    #[test]
    fn test_kiro_provider_new() {
        cleanup_env();
        let provider = KiroProvider::new();
        assert_eq!(provider.command, "kiro-cli");
        assert_eq!(provider.timeout, Duration::from_secs(600));
    }

    #[test]
    fn test_kiro_provider_name() {
        cleanup_env();
        let provider = KiroProvider::new();
        assert_eq!(provider.name(), "kiro");
    }

    #[test]
    fn test_kiro_build_command() {
        cleanup_env();
        let provider = KiroProvider::new();
        let context = Context {
            working_dir: PathBuf::from("/test"),
            session_id: "test-session".to_string(),
            contract_hash: None,
        };

        let cmd = provider.build_command("test prompt", &context);
        let args: Vec<_> = cmd.get_args().collect();

        assert!(args.contains(&std::ffi::OsStr::new("chat")));
        assert!(args.contains(&std::ffi::OsStr::new("--message")));
        assert!(args.contains(&std::ffi::OsStr::new("test prompt")));
    }
}
