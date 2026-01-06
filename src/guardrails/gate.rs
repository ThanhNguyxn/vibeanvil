//! Approval Gate - Interactive and file-based approval for changes

use anyhow::{Context, Result};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::io::{self, Write};
use std::path::PathBuf;
use tokio::fs;

use super::capsule::{ApprovalMethod, Capsule};
use super::{GuardrailsConfig, GuardrailsMode, RiskLevel};
use crate::audit::AuditLogger;

/// File-based approval token for CI environments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalToken {
    /// Capsule ID this approval is for
    pub capsule_id: String,

    /// Whether approved
    pub approved: bool,

    /// Approver identifier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approved_by: Option<String>,

    /// Reason for denial (if denied)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub denial_reason: Option<String>,
}

/// Approval gate for managing change approvals
pub struct ApprovalGate<'a> {
    config: &'a GuardrailsConfig,
    logger: &'a AuditLogger,
}

impl<'a> ApprovalGate<'a> {
    /// Create a new approval gate
    pub fn new(config: &'a GuardrailsConfig, logger: &'a AuditLogger) -> Self {
        Self { config, logger }
    }

    /// Process a capsule through the gate
    /// Returns true if approved, false if denied
    pub async fn process(&self, capsule: &mut Capsule) -> Result<bool> {
        let risk = capsule.meta.risk;

        // Log classification
        self.log_classification(capsule).await?;

        // Check if this level can auto-approve
        if !self.requires_approval(risk) {
            capsule
                .meta
                .approve("system", ApprovalMethod::Auto);
            self.log_approval(capsule, "auto").await?;
            return Ok(true);
        }

        // Present the diff
        self.present_diff(capsule).await?;

        // Try file-based approval first (for CI)
        if let Some(token_path) = self.find_approval_token(capsule) {
            return self.process_file_approval(capsule, &token_path).await;
        }

        // Interactive approval
        self.request_interactive_approval(capsule).await
    }

    /// Check if this risk level requires approval
    fn requires_approval(&self, risk: RiskLevel) -> bool {
        match self.config.mode {
            GuardrailsMode::Off => false,
            GuardrailsMode::Normal => risk.requires_approval(self.config.auto_approve_level_a),
            GuardrailsMode::Strict => true, // All levels need approval in strict mode
        }
    }

    /// Present the diff to the user
    async fn present_diff(&self, capsule: &Capsule) -> Result<()> {
        let (added, removed) = capsule.diff_stats();

        println!();
        println!(
            "{}",
            "╔═══════════════════════════════════════════════════════════════╗".cyan()
        );
        println!(
            "{}",
            "║                    📋 Change Gate Review                      ║".cyan()
        );
        println!(
            "{}",
            "╚═══════════════════════════════════════════════════════════════╝".cyan()
        );
        println!();

        // Risk level with color
        let risk_display = match capsule.meta.risk {
            RiskLevel::A => "🟢 Level A (Safe)".green(),
            RiskLevel::B => "🟡 Level B (Logic)".yellow(),
            RiskLevel::C => "🔴 Level C (High-Impact)".red(),
        };

        println!("  {} {}", "Risk:".bold(), risk_display);
        println!();

        // Reasons
        if !capsule.meta.reasons.is_empty() {
            println!("  {}:", "Reasons".bold());
            for reason in &capsule.meta.reasons {
                println!("    • {}", reason);
            }
            println!();
        }

        // Files
        println!("  {}:", "Touched Files".bold());
        for file in &capsule.meta.touched_files {
            println!("    • {}", file);
        }
        println!();

        // Stats
        println!(
            "  {} +{} / -{}",
            "Changes:".bold(),
            added.to_string().green(),
            removed.to_string().red()
        );
        println!();

        // Show diff preview (first 30 lines)
        println!("  {}:", "Diff Preview".bold());
        println!("  {}", "─".repeat(60).dimmed());
        for (i, line) in capsule.diff.lines().take(30).enumerate() {
            let colored_line = if line.starts_with('+') && !line.starts_with("+++") {
                line.green().to_string()
            } else if line.starts_with('-') && !line.starts_with("---") {
                line.red().to_string()
            } else if line.starts_with("@@") {
                line.cyan().to_string()
            } else {
                line.dimmed().to_string()
            };
            println!("  {}", colored_line);

            if i == 29 && capsule.diff.lines().count() > 30 {
                println!("  {} (showing 30/{} lines)", "...".dimmed(), capsule.diff.lines().count());
            }
        }
        println!("  {}", "─".repeat(60).dimmed());
        println!();

        // Log diff presented
        self.logger
            .log_command(
                "DIFF_PRESENTED",
                vec![
                    capsule.meta.capsule_id.clone(),
                    format!("+{}", added),
                    format!("-{}", removed),
                ],
            )
            .await?;

        Ok(())
    }

    /// Request interactive approval from user
    async fn request_interactive_approval(&self, capsule: &mut Capsule) -> Result<bool> {
        // Level C requires impact analysis
        if capsule.meta.risk == RiskLevel::C && self.config.require_impact_for_c {
            if capsule.meta.impact.is_none() {
                println!(
                    "  {} Level C requires impact analysis.",
                    "⚠".yellow()
                );
                println!("  Please provide impact analysis:");
                print!("  > ");
                io::stdout().flush()?;

                let mut impact = String::new();
                io::stdin().read_line(&mut impact)?;
                capsule.meta.impact = Some(impact.trim().to_string());
            }

            if let Some(impact) = &capsule.meta.impact {
                println!();
                println!("  {}: {}", "Impact".bold(), impact);
            }
        }

        // Ask for approval
        println!();
        print!(
            "  {} Apply this change? [y/N]: ",
            "→".cyan()
        );
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        let approved = input.trim().to_lowercase() == "y" || input.trim().to_lowercase() == "yes";

        if approved {
            capsule.meta.approve("user", ApprovalMethod::Interactive);
            self.log_approval(capsule, "interactive").await?;
            println!("  {} Change approved!", "✓".green());
        } else {
            capsule.meta.deny();
            self.log_denial(capsule, "User denied").await?;
            println!("  {} Change denied.", "✗".red());
        }

        Ok(approved)
    }

    /// Look for file-based approval token
    fn find_approval_token(&self, capsule: &Capsule) -> Option<PathBuf> {
        let token_path = capsule.path().join("approve.json");
        if token_path.exists() {
            Some(token_path)
        } else {
            None
        }
    }

    /// Process file-based approval
    async fn process_file_approval(
        &self,
        capsule: &mut Capsule,
        token_path: &PathBuf,
    ) -> Result<bool> {
        let content = fs::read_to_string(token_path)
            .await
            .context("Failed to read approval token")?;

        let token: ApprovalToken =
            serde_json::from_str(&content).context("Failed to parse approval token")?;

        if token.capsule_id != capsule.meta.capsule_id {
            anyhow::bail!("Approval token ID mismatch");
        }

        if token.approved {
            let by = token.approved_by.unwrap_or_else(|| "token".to_string());
            capsule.meta.approve(&by, ApprovalMethod::Token);
            self.log_approval(capsule, "token").await?;
            println!("  {} Change approved via token.", "✓".green());
            Ok(true)
        } else {
            capsule.meta.deny();
            let reason = token
                .denial_reason
                .unwrap_or_else(|| "Denied by token".to_string());
            self.log_denial(capsule, &reason).await?;
            println!("  {} Change denied via token: {}", "✗".red(), reason);
            Ok(false)
        }
    }

    /// Log risk classification
    async fn log_classification(&self, capsule: &Capsule) -> Result<()> {
        let metadata = serde_json::json!({
            "capsule_id": capsule.meta.capsule_id,
            "risk": capsule.meta.risk,
            "reasons": capsule.meta.reasons,
            "touched_files": capsule.meta.touched_files,
        });

        self.logger
            .log_command("RISK_CLASSIFIED", vec![serde_json::to_string(&metadata)?])
            .await?;

        Ok(())
    }

    /// Log approval
    async fn log_approval(&self, capsule: &Capsule, method: &str) -> Result<()> {
        let metadata = serde_json::json!({
            "capsule_id": capsule.meta.capsule_id,
            "approved_by": capsule.meta.approved_by,
            "method": method,
        });

        self.logger
            .log_command("APPROVAL_GRANTED", vec![serde_json::to_string(&metadata)?])
            .await?;

        Ok(())
    }

    /// Log denial
    async fn log_denial(&self, capsule: &Capsule, reason: &str) -> Result<()> {
        let metadata = serde_json::json!({
            "capsule_id": capsule.meta.capsule_id,
            "reason": reason,
        });

        self.logger
            .log_command("APPROVAL_DENIED", vec![serde_json::to_string(&metadata)?])
            .await?;

        Ok(())
    }

    /// Log capsule applied
    pub async fn log_applied(&self, capsule: &Capsule) -> Result<()> {
        let metadata = serde_json::json!({
            "capsule_id": capsule.meta.capsule_id,
            "files_changed": capsule.meta.touched_files,
        });

        self.logger
            .log_command("CAPSULE_APPLIED", vec![serde_json::to_string(&metadata)?])
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::guardrails::capsule::CapsuleMeta;

    #[test]
    fn test_approval_token_serialization() {
        let token = ApprovalToken {
            capsule_id: "test-123".to_string(),
            approved: true,
            approved_by: Some("ci".to_string()),
            denial_reason: None,
        };

        let json = serde_json::to_string(&token).unwrap();
        assert!(json.contains("\"approved\":true"));

        let parsed: ApprovalToken = serde_json::from_str(&json).unwrap();
        assert!(parsed.approved);
    }

    #[test]
    fn test_requires_approval() {
        let config = GuardrailsConfig::default();
        let logger = AuditLogger::new("test");
        let gate = ApprovalGate::new(&config, &logger);

        // Normal mode: A doesn't require, B/C do
        assert!(!gate.requires_approval(RiskLevel::A));
        assert!(gate.requires_approval(RiskLevel::B));
        assert!(gate.requires_approval(RiskLevel::C));
    }

    #[tokio::test]
    async fn test_auto_approve_level_a() {
        let config = GuardrailsConfig::default();
        let logger = AuditLogger::new("test");
        let gate = ApprovalGate::new(&config, &logger);

        let mut capsule = Capsule::new(
            CapsuleMeta::new("test".to_string(), RiskLevel::A, vec![], vec![], false),
            "diff".to_string(),
            "session".to_string(),
        );

        // Should auto-approve without interaction
        let result = gate.process(&mut capsule).await.unwrap();
        assert!(result);
        assert_eq!(capsule.meta.approval_status, crate::guardrails::capsule::ApprovalStatus::Approved);
    }

    #[tokio::test]
    async fn test_file_approval() {
        let config = GuardrailsConfig::default();
        let logger = AuditLogger::new("test");
        let gate = ApprovalGate::new(&config, &logger);

        // Create a temporary capsule directory
        let session_id = "test-session";
        let capsule_id = "test-capsule";
        let capsule_dir = crate::guardrails::capsule::capsule_path(session_id, capsule_id);
        tokio::fs::create_dir_all(&capsule_dir).await.unwrap();

        let mut capsule = Capsule::new(
            CapsuleMeta::new(capsule_id.to_string(), RiskLevel::B, vec![], vec![], false),
            "diff".to_string(),
            session_id.to_string(),
        );

        // Create approval token
        let token = ApprovalToken {
            capsule_id: capsule_id.to_string(),
            approved: true,
            approved_by: Some("ci".to_string()),
            denial_reason: None,
        };
        let token_json = serde_json::to_string(&token).unwrap();
        tokio::fs::write(capsule_dir.join("approve.json"), token_json).await.unwrap();

        // Should approve via token
        let result = gate.process(&mut capsule).await.unwrap();
        assert!(result);
        assert_eq!(capsule.meta.approval_status, crate::guardrails::capsule::ApprovalStatus::Approved);
        
        // Cleanup
        let _ = tokio::fs::remove_dir_all(crate::guardrails::capsule::capsules_path(session_id)).await;
    }
}
