//! Snapshot command handler

use anyhow::Result;
use tokio::fs;

use crate::audit::{AuditLogger, generate_session_id};
use crate::evidence::EvidenceCollector;
use crate::workspace;

pub async fn run(message: Option<String>) -> Result<()> {
    let state_data = workspace::load_state().await?;

    let session_id = generate_session_id();
    let logger = AuditLogger::new(&session_id);

    // Create snapshot directory
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let snapshot_name = format!("snapshot_{}", timestamp);
    let snapshot_dir = workspace::sessions_path().join(&snapshot_name);
    fs::create_dir_all(&snapshot_dir).await?;

    // Capture git diff
    let evidence = EvidenceCollector::new(&session_id).await?;
    let _ = evidence.capture_git_diff().await;

    // Save snapshot metadata
    let metadata = serde_json::json!({
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "state": state_data.current_state,
        "spec_hash": state_data.spec_hash,
        "message": message.clone().unwrap_or_default(),
        "session_id": session_id,
    });

    let metadata_path = snapshot_dir.join("metadata.json");
    fs::write(&metadata_path, serde_json::to_string_pretty(&metadata)?).await?;

    // Copy current state
    let state_path = workspace::state_path();
    if state_path.exists() {
        fs::copy(&state_path, snapshot_dir.join("state.json")).await?;
    }

    // Copy contract if exists
    let contract_path = workspace::contracts_path().join("contract.json");
    if contract_path.exists() {
        fs::copy(&contract_path, snapshot_dir.join("contract.json")).await?;
    }

    logger.log_command("snapshot", vec![message.clone().unwrap_or_default()]).await?;

    println!("📸 Snapshot created: {}", snapshot_name);
    if let Some(msg) = message {
        println!("   Message: {}", msg);
    }
    println!("   Path: .vibeanvil/sessions/{}/", snapshot_name);

    Ok(())
}
