//! Ship command handler

use anyhow::Result;
use tokio::fs;

use crate::audit::{AuditLogger, generate_session_id};
use crate::state::State;
use crate::workspace;

pub async fn run(tag: Option<String>, message: Option<String>) -> Result<()> {
    let state_data = workspace::load_state().await?;

    if !state_data.current_state.is_at_least(State::ReviewPassed) {
        anyhow::bail!("Review not passed. Run 'vibeanvil review pass' first.");
    }

    if state_data.current_state == State::Shipped {
        println!("Already shipped!");
        return Ok(());
    }

    let session_id = generate_session_id();

    // Update state to shipped
    let mut state_data = workspace::load_state().await?;
    state_data.transition_to(State::Shipped, "ship", &session_id)?;
    workspace::save_state(&state_data).await?;

    // Log
    let logger = AuditLogger::new(&session_id);
    logger.log_state_transition("ship", State::ReviewPassed, State::Shipped).await?;

    // Save ship record
    let ship_record = serde_json::json!({
        "shipped_at": chrono::Utc::now().to_rfc3339(),
        "tag": tag.clone(),
        "message": message.clone(),
        "spec_hash": state_data.spec_hash,
        "session_id": session_id,
    });

    let ship_path = workspace::workspace_path().join("shipped.json");
    fs::write(&ship_path, serde_json::to_string_pretty(&ship_record)?).await?;

    println!("🚀 SHIPPED!");
    println!();
    if let Some(t) = tag {
        println!("   Tag: {}", t);
    }
    if let Some(m) = message {
        println!("   Message: {}", m);
    }
    if let Some(hash) = state_data.spec_hash {
        println!("   Spec Hash: {}...", &hash[..16]);
    }
    println!();
    println!("Congratulations! Your project has been shipped.");
    println!();
    println!("To start a new iteration:");
    println!("  vibeanvil init --force");

    Ok(())
}
