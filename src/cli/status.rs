//! Status command handler

use anyhow::Result;

use crate::state::State;
use crate::workspace;

pub async fn run(verbose: bool) -> Result<()> {
    let state_data = workspace::load_state().await?;

    println!("📊 VibeAnvil Status");
    println!();
    println!("  Current state: {}", state_data.current_state);
    println!("  Tool version:  {}", state_data.tool_version);

    if let Some(hash) = &state_data.spec_hash {
        println!("  Spec hash:     {}...", &hash[..16.min(hash.len())]);
    }

    println!();
    print_workflow_progress(&state_data.current_state);

    if verbose && !state_data.history.is_empty() {
        println!();
        println!("History (last 10):");
        for entry in state_data.recent_history(10) {
            println!(
                "  {} → {} at {} ({})",
                entry.from_state,
                entry.to_state,
                entry.timestamp.format("%Y-%m-%d %H:%M:%S"),
                entry.action
            );
        }
    }

    Ok(())
}

fn print_workflow_progress(current: &State) {
    let workflow = [
        (State::Init, "Init"),
        (State::IntakeCaptured, "Intake"),
        (State::BlueprintDrafted, "Blueprint"),
        (State::ContractDrafted, "Contract Draft"),
        (State::ContractLocked, "Contract Locked"),
        (State::PlanCreated, "Plan"),
        (State::BuildInProgress, "Build"),
        (State::BuildDone, "Build Done"),
        (State::ReviewPassed, "Review"),
        (State::Shipped, "Ship"),
    ];

    println!("Workflow Progress:");
    for (state, name) in &workflow {
        let marker = if current == state {
            "→"
        } else if current.is_at_least(*state) {
            "✓"
        } else {
            "○"
        };
        println!("  {} {}", marker, name);
    }
}
