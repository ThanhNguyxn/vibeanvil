//! Build execution modes: manual, auto, iterate

pub mod iterate;

use anyhow::Result;
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
    session_id: String,
    evidence: EvidenceCollector,
    started: bool,
}

impl ManualBuild {
    /// Create new manual build
    pub async fn new(session_id: &str) -> Result<Self> {
        let evidence = EvidenceCollector::new(session_id).await?;
        Ok(Self {
            session_id: session_id.to_string(),
            evidence,
            started: false,
        })
    }

    /// Start the manual build
    pub async fn start(&mut self) -> Result<()> {
        if self.started {
            anyhow::bail!("Build already started");
        }
        self.started = true;

        // Capture initial git diff
        let _ = self.evidence.capture_git_diff().await;

        println!("✓ Manual build started. Make your changes and run 'vibeanvil build manual evidence' to capture.");
        Ok(())
    }

    /// Capture evidence
    pub async fn capture_evidence(&self) -> Result<()> {
        if !self.started {
            anyhow::bail!("Build not started. Run 'vibeanvil build manual start' first.");
        }

        let evidence = self.evidence.capture_git_diff().await?;
        println!("✓ Captured evidence: {}", evidence.filename);

        Ok(())
    }

    /// Complete the build
    pub async fn complete(&self) -> Result<BuildResult> {
        if !self.started {
            anyhow::bail!("Build not started");
        }

        // Capture final diff
        let _ = self.evidence.capture_git_diff().await;

        Ok(BuildResult {
            success: true,
            iterations: 1,
            errors: vec![],
            warnings: vec![],
            evidence_files: vec![],
            output: "Manual build completed.".to_string(),
        })
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
