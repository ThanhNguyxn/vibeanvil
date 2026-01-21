//! Cody provider - Sourcegraph's AI coding assistant
//!
//! Cody is Sourcegraph's AI coding assistant with excellent context
//! awareness and repository understanding.
//!
//! Installation:
//! ```bash
//! npm install -g @sourcegraph/cody
//! cody auth login --web
//! ```
//!
//! Configuration:
//! - `SRC_ENDPOINT`: Sourcegraph endpoint (optional, defaults to sourcegraph.com)
//! - `SRC_ACCESS_TOKEN`: Access token (optional if using web auth)
//! - `CODY_MODEL`: Model to use (optional)
//! - `CODY_EXTRA_ARGS`: Additional CLI arguments

use super::{Context, Provider, ProviderResponse};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use std::process::Command;
use std::time::Duration;

/// Cody AI coding assistant provider (Sourcegraph)
pub struct CodyProvider {
    command: String,
    timeout: Duration,
}

impl Default for CodyProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl CodyProvider {
    /// Create a new Cody provider
    pub fn new() -> Self {
        let timeout_secs = std::env::var("CODY_TIMEOUT_SECS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(600);

        Self {
            command: std::env::var("CODY_COMMAND").unwrap_or_else(|_| "cody".to_string()),
            timeout: Duration::from_secs(timeout_secs),
        }
    }

    fn build_command(&self, prompt: &str, context: &Context) -> Command {
        let mut cmd = Command::new(&self.command);

        // Set working directory
        cmd.current_dir(&context.working_dir);

        // Use chat subcommand with message
        cmd.arg("chat");
        cmd.arg("-m").arg(prompt);

        // Add model if specified
        if let Ok(model) = std::env::var("CODY_MODEL") {
            cmd.arg("--model").arg(model);
        }

        // Add context files/repo if specified
        if let Ok(context_repo) = std::env::var("CODY_CONTEXT_REPO") {
            cmd.arg("--context-repo").arg(context_repo);
        }

        // Add extra arguments
        if let Ok(extra_args) = std::env::var("CODY_EXTRA_ARGS") {
            for arg in extra_args.split_whitespace() {
                cmd.arg(arg);
            }
        }

        cmd
    }
}

#[async_trait]
impl Provider for CodyProvider {
    async fn execute(&self, prompt: &str, context: &Context) -> Result<ProviderResponse> {
        let mut cmd = self.build_command(prompt, context);

        // Execute with timeout
        let output = tokio::time::timeout(self.timeout, async {
            tokio::task::spawn_blocking(move || cmd.output())
                .await
                .map_err(|e| anyhow!("Task join error: {}", e))?
                .map_err(|e| anyhow!("Failed to execute cody: {}", e))
        })
        .await
        .map_err(|_| anyhow!("Cody timed out after {:?}", self.timeout))??;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        let success = output.status.success();
        let errors = if stderr.is_empty() {
            vec![]
        } else {
            vec![stderr.clone()]
        };

        Ok(ProviderResponse {
            success,
            output: if success { stdout } else { stderr },
            errors,
            warnings: vec![],
            files_modified: vec![], // Cody chat doesn't modify files directly
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
        "cody"
    }

    fn is_available(&self) -> bool {
        which::which(&self.command).is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn cleanup_env() {
        std::env::remove_var("CODY_COMMAND");
        std::env::remove_var("CODY_MODEL");
        std::env::remove_var("CODY_EXTRA_ARGS");
        std::env::remove_var("CODY_TIMEOUT_SECS");
        std::env::remove_var("CODY_CONTEXT_REPO");
    }

    #[test]
    fn test_cody_provider_new() {
        cleanup_env();
        let provider = CodyProvider::new();
        assert_eq!(provider.command, "cody");
        assert_eq!(provider.timeout, Duration::from_secs(600));
    }

    #[test]
    fn test_cody_provider_name() {
        cleanup_env();
        let provider = CodyProvider::new();
        assert_eq!(provider.name(), "cody");
    }

    #[test]
    fn test_cody_build_command() {
        cleanup_env();
        let provider = CodyProvider::new();
        let context = Context {
            working_dir: PathBuf::from("/test"),
            session_id: "test-session".to_string(),
            contract_hash: None,
        };

        let cmd = provider.build_command("test prompt", &context);
        let args: Vec<_> = cmd.get_args().collect();

        assert!(args.contains(&std::ffi::OsStr::new("chat")));
        assert!(args.contains(&std::ffi::OsStr::new("-m")));
        assert!(args.contains(&std::ffi::OsStr::new("test prompt")));
    }
}
