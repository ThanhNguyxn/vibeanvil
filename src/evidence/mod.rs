//! Evidence capture and management

use anyhow::{Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;
use tokio::fs;

use crate::workspace;

/// Evidence types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceType {
    GitDiff,
    BuildLog,
    TestLog,
    LintLog,
    Custom(String),
}

/// Evidence metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    /// Evidence type
    pub evidence_type: EvidenceType,
    /// Filename
    pub filename: String,
    /// Timestamp
    pub captured_at: chrono::DateTime<chrono::Utc>,
    /// Size in bytes
    pub size_bytes: u64,
    /// Was content redacted
    pub redacted: bool,
    /// Description
    pub description: Option<String>,
}

/// Secret patterns for redaction
const SECRET_PATTERNS: &[&str] = &[
    r#"(?i)(api[_-]?key\s*[=:]\s*)(['"]?)[\w\-]{20,}['"]?"#,
    r#"(?i)(secret[_-]?key\s*[=:]\s*)(['"]?)[\w\-]{20,}['"]?"#,
    r#"(?i)(password\s*[=:]\s*)(['"]?)[\w\-!@#$%^&*]{8,}['"]?"#,
    r#"(?i)(token\s*[=:]\s*)(['"]?)[\w\-]{20,}['"]?"#,
    r"(?i)(bearer\s+)[\w\-\.]+",
    r"(?i)(authorization:\s*)(basic|bearer)\s+[\w\-\.=]+",
    r"ghp_[a-zA-Z0-9]{36}",
    r"gho_[a-zA-Z0-9]{36}",
    r"sk-[a-zA-Z0-9]{48}",
    r"AKIA[0-9A-Z]{16}",
];

/// Redact secrets from content
pub fn redact_secrets(content: &str) -> (String, bool) {
    let mut result = content.to_string();
    let mut was_redacted = false;

    for pattern in SECRET_PATTERNS {
        if let Ok(re) = Regex::new(pattern) {
            if re.is_match(&result) {
                was_redacted = true;
                result = re.replace_all(&result, "[REDACTED]").to_string();
            }
        }
    }

    (result, was_redacted)
}

/// Evidence collector for a session
pub struct EvidenceCollector {
    session_id: String,
    evidence_dir: PathBuf,
}

impl EvidenceCollector {
    /// Create a new evidence collector
    pub async fn new(session_id: &str) -> Result<Self> {
        let evidence_dir = workspace::evidence_path(session_id);
        fs::create_dir_all(&evidence_dir).await?;

        Ok(Self {
            session_id: session_id.to_string(),
            evidence_dir,
        })
    }

    /// Capture git diff
    pub async fn capture_git_diff(&self) -> Result<Evidence> {
        let output = Command::new("git")
            .args(["diff", "HEAD"])
            .output()
            .context("Failed to run git diff")?;

        let content = String::from_utf8_lossy(&output.stdout);
        let (redacted_content, was_redacted) = redact_secrets(&content);

        let filename = format!(
            "git_diff_{}.txt",
            chrono::Utc::now().format("%Y%m%d_%H%M%S")
        );
        let filepath = self.evidence_dir.join(&filename);

        fs::write(&filepath, &redacted_content).await?;

        Ok(Evidence {
            evidence_type: EvidenceType::GitDiff,
            filename,
            captured_at: chrono::Utc::now(),
            size_bytes: redacted_content.len() as u64,
            redacted: was_redacted,
            description: Some("Git diff from HEAD".to_string()),
        })
    }

    /// Capture build log
    pub async fn capture_build_log(&self, content: &str) -> Result<Evidence> {
        let (redacted_content, was_redacted) = redact_secrets(content);

        let filename = format!(
            "build_log_{}.txt",
            chrono::Utc::now().format("%Y%m%d_%H%M%S")
        );
        let filepath = self.evidence_dir.join(&filename);

        fs::write(&filepath, &redacted_content).await?;

        Ok(Evidence {
            evidence_type: EvidenceType::BuildLog,
            filename,
            captured_at: chrono::Utc::now(),
            size_bytes: redacted_content.len() as u64,
            redacted: was_redacted,
            description: Some("Build output log".to_string()),
        })
    }

    /// Capture test log
    pub async fn capture_test_log(&self, content: &str) -> Result<Evidence> {
        let (redacted_content, was_redacted) = redact_secrets(content);

        let filename = format!(
            "test_log_{}.txt",
            chrono::Utc::now().format("%Y%m%d_%H%M%S")
        );
        let filepath = self.evidence_dir.join(&filename);

        fs::write(&filepath, &redacted_content).await?;

        Ok(Evidence {
            evidence_type: EvidenceType::TestLog,
            filename,
            captured_at: chrono::Utc::now(),
            size_bytes: redacted_content.len() as u64,
            redacted: was_redacted,
            description: Some("Test output log".to_string()),
        })
    }

    /// Capture lint log
    pub async fn capture_lint_log(&self, content: &str) -> Result<Evidence> {
        let (redacted_content, was_redacted) = redact_secrets(content);

        let filename = format!(
            "lint_log_{}.txt",
            chrono::Utc::now().format("%Y%m%d_%H%M%S")
        );
        let filepath = self.evidence_dir.join(&filename);

        fs::write(&filepath, &redacted_content).await?;

        Ok(Evidence {
            evidence_type: EvidenceType::LintLog,
            filename,
            captured_at: chrono::Utc::now(),
            size_bytes: redacted_content.len() as u64,
            redacted: was_redacted,
            description: Some("Lint output log".to_string()),
        })
    }

    /// Capture custom evidence
    pub async fn capture_custom(
        &self,
        name: &str,
        content: &str,
        description: &str,
    ) -> Result<Evidence> {
        let (redacted_content, was_redacted) = redact_secrets(content);

        let filename = format!(
            "{}_{}.txt",
            name,
            chrono::Utc::now().format("%Y%m%d_%H%M%S")
        );
        let filepath = self.evidence_dir.join(&filename);

        fs::write(&filepath, &redacted_content).await?;

        Ok(Evidence {
            evidence_type: EvidenceType::Custom(name.to_string()),
            filename,
            captured_at: chrono::Utc::now(),
            size_bytes: redacted_content.len() as u64,
            redacted: was_redacted,
            description: Some(description.to_string()),
        })
    }

    /// Save evidence manifest
    pub async fn save_manifest(&self, evidence: &[Evidence]) -> Result<()> {
        let manifest_path = self.evidence_dir.join("manifest.json");
        let content = serde_json::to_string_pretty(evidence)?;
        fs::write(manifest_path, content).await?;
        Ok(())
    }

    /// Load evidence manifest
    pub async fn load_manifest(&self) -> Result<Vec<Evidence>> {
        let manifest_path = self.evidence_dir.join("manifest.json");
        if !manifest_path.exists() {
            return Ok(vec![]);
        }

        let content = fs::read_to_string(manifest_path).await?;
        let evidence: Vec<Evidence> = serde_json::from_str(&content)?;
        Ok(evidence)
    }

    /// Get session ID
    pub fn session_id(&self) -> &str {
        &self.session_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secret_redaction() {
        let content = "API_KEY=sk-1234567890abcdefghijklmnopqrstuvwxyz1234567890ab";
        let (redacted, was_redacted) = redact_secrets(content);
        assert!(was_redacted);
        assert!(redacted.contains("[REDACTED]"));
        assert!(!redacted.contains("sk-"));
    }

    #[test]
    fn test_github_token_redaction() {
        let content = "ghp_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx is my token";
        let (redacted, was_redacted) = redact_secrets(content);
        assert!(was_redacted);
        assert!(redacted.contains("[REDACTED]"));
    }

    #[test]
    fn test_no_redaction_needed() {
        let content = "This is normal content without secrets";
        let (redacted, was_redacted) = redact_secrets(content);
        assert!(!was_redacted);
        assert_eq!(redacted, content);
    }
}
