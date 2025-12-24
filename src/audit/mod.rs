//! Audit logging in JSONL format

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;

use crate::state::State;
use crate::workspace;

/// A single audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// Timestamp of the event
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Command that was executed
    pub command: String,
    /// Command arguments
    pub args: Vec<String>,
    /// Previous state (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prev_state: Option<State>,
    /// Next/current state (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_state: Option<State>,
    /// Session ID
    pub session_id: String,
    /// Additional metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    /// Success status
    pub success: bool,
    /// Error message if failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl AuditEntry {
    /// Create a new audit entry for a command
    pub fn new(command: &str, args: Vec<String>, session_id: &str) -> Self {
        Self {
            timestamp: chrono::Utc::now(),
            command: command.to_string(),
            args,
            prev_state: None,
            next_state: None,
            session_id: session_id.to_string(),
            metadata: None,
            success: true,
            error: None,
        }
    }

    /// Set state transition info
    pub fn with_state_transition(mut self, prev: State, next: State) -> Self {
        self.prev_state = Some(prev);
        self.next_state = Some(next);
        self
    }

    /// Set metadata
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Mark as failed
    pub fn with_error(mut self, error: &str) -> Self {
        self.success = false;
        self.error = Some(error.to_string());
        self
    }

    /// Serialize to JSONL (single line)
    pub fn to_jsonl(&self) -> Result<String> {
        serde_json::to_string(self).context("Failed to serialize audit entry")
    }
}

/// Logger for audit trail
pub struct AuditLogger {
    session_id: String,
}

impl AuditLogger {
    /// Create a new audit logger
    pub fn new(session_id: &str) -> Self {
        Self {
            session_id: session_id.to_string(),
        }
    }

    /// Log a command execution
    pub async fn log_command(&self, command: &str, args: Vec<String>) -> Result<AuditEntry> {
        let entry = AuditEntry::new(command, args, &self.session_id);
        self.write_entry(&entry).await?;
        Ok(entry)
    }

    /// Log a state transition
    pub async fn log_state_transition(
        &self,
        command: &str,
        prev: State,
        next: State,
    ) -> Result<AuditEntry> {
        let entry = AuditEntry::new(command, vec![], &self.session_id)
            .with_state_transition(prev, next);
        self.write_entry(&entry).await?;
        Ok(entry)
    }

    /// Log an error
    pub async fn log_error(&self, command: &str, error: &str) -> Result<AuditEntry> {
        let entry = AuditEntry::new(command, vec![], &self.session_id)
            .with_error(error);
        self.write_entry(&entry).await?;
        Ok(entry)
    }

    /// Log with custom entry
    pub async fn log(&self, entry: &AuditEntry) -> Result<()> {
        self.write_entry(entry).await
    }

    /// Write entry to audit log file
    async fn write_entry(&self, entry: &AuditEntry) -> Result<()> {
        let path = workspace::audit_log_path();
        
        // Ensure logs directory exists
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let line = format!("{}\n", entry.to_jsonl()?);

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .await
            .context("Failed to open audit log")?;

        file.write_all(line.as_bytes())
            .await
            .context("Failed to write to audit log")?;

        Ok(())
    }
}

/// Read audit log entries
pub async fn read_audit_log(limit: Option<usize>) -> Result<Vec<AuditEntry>> {
    let path = workspace::audit_log_path();
    
    if !path.exists() {
        return Ok(vec![]);
    }

    let content = tokio::fs::read_to_string(&path)
        .await
        .context("Failed to read audit log")?;

    let entries: Vec<AuditEntry> = content
        .lines()
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect();

    match limit {
        Some(n) => Ok(entries.into_iter().rev().take(n).rev().collect()),
        None => Ok(entries),
    }
}

/// Generate a new session ID
pub fn generate_session_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_entry_serialization() {
        let entry = AuditEntry::new("test", vec!["arg1".to_string()], "session-1")
            .with_state_transition(State::Init, State::IntakeCaptured);
        
        let json = entry.to_jsonl().unwrap();
        assert!(json.contains("\"command\":\"test\""));
        assert!(json.contains("\"session_id\":\"session-1\""));
    }

    #[test]
    fn test_error_entry() {
        let entry = AuditEntry::new("fail", vec![], "session-1")
            .with_error("Something went wrong");
        
        assert!(!entry.success);
        assert_eq!(entry.error.as_deref(), Some("Something went wrong"));
    }
}
