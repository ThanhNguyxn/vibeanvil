//! Change Capsule - metadata + diff storage for gated changes

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;

use super::RiskLevel;
use crate::workspace;

/// Approval status for a capsule
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ApprovalStatus {
    Pending,
    Approved,
    Denied,
}

/// How the approval was granted
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ApprovalMethod {
    /// User approved interactively
    Interactive,
    /// Approved via file token (CI)
    Token,
    /// Auto-approved (Level A in normal mode)
    Auto,
}

/// Change Capsule metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapsuleMeta {
    /// Unique capsule ID
    pub capsule_id: String,

    /// Creation timestamp
    pub timestamp: DateTime<Utc>,

    /// Risk classification
    pub risk: RiskLevel,

    /// Classification reasons
    pub reasons: Vec<String>,

    /// Why this change is being made
    #[serde(skip_serializing_if = "Option::is_none")]
    pub why: Option<String>,

    /// Impact analysis (required for Level C)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub impact: Option<String>,

    /// Validation plan
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validation_plan: Option<String>,

    /// Alternatives considered
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub alternatives: Vec<String>,

    /// Files touched by this change
    pub touched_files: Vec<String>,

    /// Whether public API surface is affected
    #[serde(default)]
    pub public_surface_changes: bool,

    /// Provider that generated this change
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,

    /// Related contract IDs
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub related_contract_ids: Vec<String>,

    /// Current approval status
    pub approval_status: ApprovalStatus,

    /// Who/what approved
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approved_by: Option<String>,

    /// When approved
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approved_at: Option<DateTime<Utc>>,

    /// Approval method used
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approval_method: Option<ApprovalMethod>,
}

impl CapsuleMeta {
    /// Create a new capsule with classification results
    pub fn new(
        capsule_id: String,
        risk: RiskLevel,
        reasons: Vec<String>,
        touched_files: Vec<String>,
        public_surface_changes: bool,
    ) -> Self {
        Self {
            capsule_id,
            timestamp: Utc::now(),
            risk,
            reasons,
            why: None,
            impact: None,
            validation_plan: None,
            alternatives: Vec::new(),
            touched_files,
            public_surface_changes,
            provider: None,
            related_contract_ids: Vec::new(),
            approval_status: ApprovalStatus::Pending,
            approved_by: None,
            approved_at: None,
            approval_method: None,
        }
    }

    /// Set the why field
    pub fn with_why(mut self, why: impl Into<String>) -> Self {
        self.why = Some(why.into());
        self
    }

    /// Set the impact field
    pub fn with_impact(mut self, impact: impl Into<String>) -> Self {
        self.impact = Some(impact.into());
        self
    }

    /// Set the provider
    pub fn with_provider(mut self, provider: impl Into<String>) -> Self {
        self.provider = Some(provider.into());
        self
    }

    /// Mark as approved
    pub fn approve(&mut self, by: impl Into<String>, method: ApprovalMethod) {
        self.approval_status = ApprovalStatus::Approved;
        self.approved_by = Some(by.into());
        self.approved_at = Some(Utc::now());
        self.approval_method = Some(method);
    }

    /// Mark as denied
    pub fn deny(&mut self) {
        self.approval_status = ApprovalStatus::Denied;
    }
}

/// A complete Change Capsule (metadata + diff)
pub struct Capsule {
    /// Capsule metadata
    pub meta: CapsuleMeta,

    /// The unified diff content
    pub diff: String,

    /// Session this capsule belongs to
    pub session_id: String,
}

impl Capsule {
    /// Create a new capsule
    pub fn new(meta: CapsuleMeta, diff: String, session_id: String) -> Self {
        Self {
            meta,
            diff,
            session_id,
        }
    }

    /// Get the capsule directory path
    pub fn path(&self) -> PathBuf {
        capsule_path(&self.session_id, &self.meta.capsule_id)
    }

    /// Save the capsule to disk
    pub async fn save(&self) -> Result<PathBuf> {
        let path = self.path();
        fs::create_dir_all(&path).await?;

        // Save meta.json
        let meta_json =
            serde_json::to_string_pretty(&self.meta).context("Failed to serialize capsule meta")?;
        fs::write(path.join("meta.json"), meta_json).await?;

        // Save patch.diff
        fs::write(path.join("patch.diff"), &self.diff).await?;

        Ok(path)
    }

    /// Load a capsule from disk
    pub async fn load(session_id: &str, capsule_id: &str) -> Result<Self> {
        let path = capsule_path(session_id, capsule_id);

        let meta_content = fs::read_to_string(path.join("meta.json"))
            .await
            .context("Failed to read capsule meta")?;
        let meta: CapsuleMeta =
            serde_json::from_str(&meta_content).context("Failed to parse capsule meta")?;

        let diff = fs::read_to_string(path.join("patch.diff"))
            .await
            .context("Failed to read capsule diff")?;

        Ok(Self {
            meta,
            diff,
            session_id: session_id.to_string(),
        })
    }

    /// Get line statistics from the diff
    pub fn diff_stats(&self) -> (usize, usize) {
        let mut added = 0;
        let mut removed = 0;

        for line in self.diff.lines() {
            if line.starts_with('+') && !line.starts_with("+++") {
                added += 1;
            } else if line.starts_with('-') && !line.starts_with("---") {
                removed += 1;
            }
        }

        (added, removed)
    }
}

/// Get path to capsules directory for a session
pub fn capsules_path(session_id: &str) -> PathBuf {
    workspace::session_path(session_id).join("capsules")
}

/// Get path to a specific capsule
pub fn capsule_path(session_id: &str, capsule_id: &str) -> PathBuf {
    capsules_path(session_id).join(capsule_id)
}

/// Generate a new capsule ID
pub fn generate_capsule_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capsule_meta_creation() {
        let meta = CapsuleMeta::new(
            "test-123".to_string(),
            RiskLevel::B,
            vec!["Logic change".to_string()],
            vec!["src/main.rs".to_string()],
            false,
        );

        assert_eq!(meta.risk, RiskLevel::B);
        assert_eq!(meta.approval_status, ApprovalStatus::Pending);
    }

    #[test]
    fn test_capsule_approval() {
        let mut meta =
            CapsuleMeta::new("test-123".to_string(), RiskLevel::A, vec![], vec![], false);

        meta.approve("user", ApprovalMethod::Interactive);

        assert_eq!(meta.approval_status, ApprovalStatus::Approved);
        assert!(meta.approved_at.is_some());
    }

    #[test]
    fn test_diff_stats() {
        let capsule = Capsule::new(
            CapsuleMeta::new("test".to_string(), RiskLevel::A, vec![], vec![], false),
            "+added line 1\n+added line 2\n-removed line\n context".to_string(),
            "session-1".to_string(),
        );

        let (added, removed) = capsule.diff_stats();
        assert_eq!(added, 2);
        assert_eq!(removed, 1);
    }
}
