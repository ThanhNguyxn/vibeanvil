//! Build progress tracking for resume functionality

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::workspace;

const PROGRESS_FILE: &str = "build_progress.json";

/// Build progress data for resume functionality
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildProgress {
    /// Session ID of the build
    pub session_id: String,
    /// Build mode (auto, iterate, manual)
    pub mode: String,
    /// Provider being used
    pub provider: String,
    /// Current iteration (for iterate mode)
    pub current_iteration: u32,
    /// Maximum iterations
    pub max_iterations: u32,
    /// Current step within iteration
    pub current_step: BuildStep,
    /// Timestamp when build started
    pub started_at: DateTime<Utc>,
    /// Timestamp of last update
    pub updated_at: DateTime<Utc>,
    /// Whether tests passed in last iteration
    pub tests_passed: bool,
    /// Whether lint passed in last iteration
    pub lint_passed: bool,
    /// Last error message if any
    pub last_error: Option<String>,
    /// Accumulated changes/files modified
    pub files_modified: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum BuildStep {
    #[default]
    NotStarted,
    RunningTests,
    RunningLint,
    Fixing,
    Verifying,
    Completed,
    Failed,
}

impl BuildProgress {
    /// Create new build progress
    pub fn new(session_id: &str, mode: &str, provider: &str, max_iterations: u32) -> Self {
        let now = Utc::now();
        Self {
            session_id: session_id.to_string(),
            mode: mode.to_string(),
            provider: provider.to_string(),
            current_iteration: 0,
            max_iterations,
            current_step: BuildStep::NotStarted,
            started_at: now,
            updated_at: now,
            tests_passed: false,
            lint_passed: false,
            last_error: None,
            files_modified: vec![],
        }
    }

    /// Get progress file path
    fn progress_path() -> PathBuf {
        workspace::workspace_path().join(PROGRESS_FILE)
    }

    /// Save progress to file
    pub async fn save(&self) -> Result<()> {
        let path = Self::progress_path();
        let json = serde_json::to_string_pretty(self)?;
        tokio::fs::write(&path, json)
            .await
            .context("Failed to save build progress")?;
        Ok(())
    }

    /// Load progress from file
    pub async fn load() -> Result<Option<Self>> {
        let path = Self::progress_path();

        if !path.exists() {
            return Ok(None);
        }

        let content = tokio::fs::read_to_string(&path)
            .await
            .context("Failed to read build progress")?;

        let progress: Self =
            serde_json::from_str(&content).context("Failed to parse build progress")?;

        Ok(Some(progress))
    }

    /// Check if there's resumable progress
    pub async fn has_resumable() -> bool {
        let path = Self::progress_path();
        if !path.exists() {
            return false;
        }

        if let Ok(Some(progress)) = Self::load().await {
            // Only resume if not completed or failed
            matches!(
                progress.current_step,
                BuildStep::RunningTests
                    | BuildStep::RunningLint
                    | BuildStep::Fixing
                    | BuildStep::Verifying
            )
        } else {
            false
        }
    }

    /// Clear progress file
    pub async fn clear() -> Result<()> {
        let path = Self::progress_path();
        if path.exists() {
            tokio::fs::remove_file(&path)
                .await
                .context("Failed to remove build progress")?;
        }
        Ok(())
    }

    /// Update progress step
    pub fn set_step(&mut self, step: BuildStep) {
        self.current_step = step;
        self.updated_at = Utc::now();
    }

    /// Increment iteration
    pub fn next_iteration(&mut self) {
        self.current_iteration += 1;
        self.updated_at = Utc::now();
    }

    /// Record error
    pub fn set_error(&mut self, error: &str) {
        self.last_error = Some(error.to_string());
        self.current_step = BuildStep::Failed;
        self.updated_at = Utc::now();
    }

    /// Record file modification
    pub fn add_modified_file(&mut self, path: &str) {
        if !self.files_modified.contains(&path.to_string()) {
            self.files_modified.push(path.to_string());
        }
        self.updated_at = Utc::now();
    }

    /// Mark as completed
    pub fn complete(&mut self) {
        self.current_step = BuildStep::Completed;
        self.updated_at = Utc::now();
    }
}

impl std::fmt::Display for BuildStep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuildStep::NotStarted => write!(f, "Not Started"),
            BuildStep::RunningTests => write!(f, "Running Tests"),
            BuildStep::RunningLint => write!(f, "Running Lint"),
            BuildStep::Fixing => write!(f, "Fixing Issues"),
            BuildStep::Verifying => write!(f, "Verifying"),
            BuildStep::Completed => write!(f, "Completed"),
            BuildStep::Failed => write!(f, "Failed"),
        }
    }
}
