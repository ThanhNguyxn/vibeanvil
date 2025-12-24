//! Intake command handler

use anyhow::Result;
use std::io::{self, Read};
use tokio::fs;

use crate::audit::{generate_session_id, AuditLogger};
use crate::state::State;
use crate::workspace;

pub async fn run(message: Option<String>) -> Result<()> {
    let state_data = workspace::load_state().await?;

    if state_data.current_state != State::Init {
        println!(
            "⚠️  Intake already captured. Current state: {}",
            state_data.current_state
        );
        return Ok(());
    }

    let intake_content = match message {
        Some(msg) => msg,
        None => {
            println!("Enter your requirements (Ctrl+D or Ctrl+Z when done):");
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer)?;
            buffer.trim().to_string()
        }
    };

    if intake_content.is_empty() {
        anyhow::bail!("Intake message cannot be empty");
    }

    // Save intake
    let intake_path = workspace::workspace_path().join("intake.md");
    let intake_doc = format!(
        "# Project Intake\n\n**Captured**: {}\n\n## Requirements\n\n{}\n",
        chrono::Utc::now().to_rfc3339(),
        intake_content
    );
    fs::write(&intake_path, &intake_doc).await?;

    // Update state
    let session_id = generate_session_id();
    let mut state_data = workspace::load_state().await?;
    state_data.transition_to(State::IntakeCaptured, "intake", &session_id)?;
    workspace::save_state(&state_data).await?;

    // Audit log
    let logger = AuditLogger::new(&session_id);
    logger
        .log_state_transition("intake", State::Init, State::IntakeCaptured)
        .await?;

    println!("✓ Intake captured");
    println!("  → Saved to .vibeanvil/intake.md");
    println!();
    println!("Next: vibeanvil blueprint --auto");

    Ok(())
}
