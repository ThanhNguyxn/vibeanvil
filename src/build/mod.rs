//! Build execution modes: manual, auto, iterate

pub mod iterate;

use anyhow::Result;
use colored::*;
use serde::{Deserialize, Serialize};

use crate::evidence::EvidenceCollector;

/// Build mode configuration
#[derive(Debug, Clone)]
pub struct BuildConfig {
    /// Build mode
    pub mode: BuildMode,
    /// Provider name
    pub provider: String,
    /// Maximum iterations (for iterate mode)
    pub max_iterations: u32,
    /// Strict mode - fail on first error
    pub strict: bool,
    /// Timeout per iteration in seconds
    pub timeout_secs: u64,
    /// Skip tests
    pub skip_tests: bool,
    /// Skip lint
    pub skip_lint: bool,
    /// Capture evidence
    pub capture_evidence: bool,
}

/// Build mode enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BuildMode {
    Manual,
    Auto,
    Iterate,
}

/// Build result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildResult {
    /// Whether build succeeded
    pub success: bool,
    /// Number of iterations (for iterate mode)
    pub iterations: u32,
    /// Errors encountered
    pub errors: Vec<String>,
    /// Warnings
    pub warnings: Vec<String>,
    /// Evidence collected
    pub evidence_files: Vec<String>,
    /// Build output
    pub output: String,
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            mode: BuildMode::Manual,
            provider: "claude-code".to_string(),
            max_iterations: 5,
            strict: false,
            timeout_secs: 300,
            skip_tests: false,
            skip_lint: false,
            capture_evidence: true,
        }
    }
}

/// Manual build handler
pub struct ManualBuild {
    #[allow(dead_code)]
    session_id: String,
    evidence: EvidenceCollector,
}

impl ManualBuild {
    /// Create new manual build
    pub async fn new(session_id: &str) -> Result<Self> {
        let evidence = EvidenceCollector::new(session_id).await?;
        Ok(Self {
            session_id: session_id.to_string(),
            evidence,
        })
    }

    /// Check if a build is in progress by reading state from disk
    async fn is_build_in_progress() -> bool {
        if let Ok(state) = crate::workspace::load_state().await {
            state.current_state == crate::state::State::BuildInProgress
        } else {
            false
        }
    }

    /// Start the manual build
    pub async fn start(&mut self) -> Result<()> {
        if Self::is_build_in_progress().await {
            anyhow::bail!(
                "Build already in progress. Run 'vibeanvil build manual complete' to finish."
            );
        }

        // Capture initial git diff
        let _ = self.evidence.capture_git_diff().await;

        println!("✓ Manual build started. Make your changes and run 'vibeanvil build manual evidence' to capture.");
        Ok(())
    }

    /// Capture evidence
    pub async fn capture_evidence(&self) -> Result<()> {
        if !Self::is_build_in_progress().await {
            anyhow::bail!("Build not started. Run 'vibeanvil build manual start' first.");
        }

        let evidence = self.evidence.capture_git_diff().await?;
        println!("✓ Captured evidence: {}", evidence.filename);

        Ok(())
    }

    /// Complete the build
    pub async fn complete(&self) -> Result<BuildResult> {
        if !Self::is_build_in_progress().await {
            anyhow::bail!("Build not started. Run 'vibeanvil build manual start' first.");
        }

        // Capture final diff
        let evidence_file = self.evidence.capture_git_diff().await?;

        // Auto-Commit Logic
        crate::cli::style::step("Vibe Commit");

        // Read the diff content
        let diff_content = std::fs::read_to_string(&evidence_file.filename).unwrap_or_default();

        if diff_content.trim().is_empty() {
            crate::cli::style::warn("No changes detected, skipping auto-commit.");
        } else {
            // Get provider for commit message generation
            use crate::provider::{get_provider, Context};
            let provider = get_provider("claude-code").or_else(|_| get_provider("mock"))?;

            let context = Context {
                working_dir: std::env::current_dir()?,
                session_id: self.session_id.clone(),
                contract_hash: None,
            };

            let pb = crate::cli::style::spinner("Dreaming up a commit message...");
            match provider
                .generate_commit_message(&diff_content, &context)
                .await
            {
                Ok(msg) => {
                    pb.finish_and_clear();

                    // Interactive prompt
                    let options = vec!["Confirm", "Edit", "Cancel"];
                    let ans = inquire::Select::new(
                        &format!("Proposed commit: \"{}\"", msg.cyan()),
                        options,
                    )
                    .prompt();

                    match ans {
                        Ok("Confirm") => {
                            Self::execute_commit(&msg)?;
                        }
                        Ok("Edit") => {
                            let edited = inquire::Text::new("Edit commit message:")
                                .with_initial_value(&msg)
                                .prompt()?;
                            Self::execute_commit(&edited)?;
                        }
                        _ => {
                            crate::cli::style::info("Commit cancelled.");
                        }
                    }
                }
                Err(e) => {
                    pb.finish_with_message("Failed");
                    crate::cli::style::error(&format!("Failed to generate commit message: {}", e));
                }
            }
        }

        Ok(BuildResult {
            success: true,
            iterations: 1,
            errors: vec![],
            warnings: vec![],
            evidence_files: vec![evidence_file.filename],
            output: "Manual build completed.".to_string(),
        })
    }

    fn execute_commit(msg: &str) -> Result<()> {
        let output = std::process::Command::new("git")
            .arg("commit")
            .arg("-am")
            .arg(msg)
            .output()?;

        if output.status.success() {
            crate::cli::style::success("Auto-commit successful");
        } else {
            crate::cli::style::error(&format!(
                "Auto-commit failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        Ok(())
    }
}

/// Auto build handler - single provider invocation
pub struct AutoBuild {
    config: BuildConfig,
    session_id: String,
}

impl AutoBuild {
    pub fn new(config: BuildConfig, session_id: &str) -> Self {
        Self {
            config,
            session_id: session_id.to_string(),
        }
    }

    /// Execute auto build
    pub async fn execute(&self, prompt: &str) -> Result<BuildResult> {
        use crate::provider::{get_provider, Context};

        let provider = get_provider(&self.config.provider)?;

        let context = Context {
            working_dir: std::env::current_dir()?,
            session_id: self.session_id.clone(),
            contract_hash: None,
        };

        let response = provider.execute(prompt, &context).await?;

        let evidence = EvidenceCollector::new(&self.session_id).await?;
        evidence.capture_build_log(&response.output).await?;

        Ok(BuildResult {
            success: response.success,
            iterations: 1,
            errors: response.errors,
            warnings: response.warnings,
            evidence_files: vec![],
            output: response.output,
        })
    }
}
