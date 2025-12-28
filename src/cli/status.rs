//! Status command handler with beautiful output

use anyhow::Result;
use colored::Colorize;
use serde::Serialize;

use crate::state::State;
use crate::workspace;

#[derive(Serialize)]
struct StatusJson {
    current_state: String,
    tool_version: String,
    spec_hash: Option<String>,
}

pub async fn run(verbose: bool, json: bool) -> Result<()> {
    let state_data = workspace::load_state().await?;

    if json {
        let output = StatusJson {
            current_state: state_data.current_state.to_string(),
            tool_version: state_data.tool_version.clone(),
            spec_hash: state_data.spec_hash.clone(),
        };
        println!("{}", serde_json::to_string_pretty(&output)?);
        return Ok(());
    }

    // Print beautiful header
    println!();
    println!(
        "{}",
        "╔═══════════════════════════════════════════════════════════════╗".cyan()
    );
    println!(
        "{}",
        "║               📊 VibeAnvil Status Dashboard                   ║".cyan()
    );
    println!(
        "{}",
        "╚═══════════════════════════════════════════════════════════════╝".cyan()
    );
    println!();

    // Status box
    let state_str = format!("{}", state_data.current_state);
    let state_icon = get_state_icon(&state_data.current_state);

    println!(
        "  {} {}  {}",
        "Current State:".white().bold(),
        state_icon,
        state_str.green().bold()
    );
    println!(
        "  {} v{}",
        "Tool Version: ".white().bold(),
        state_data.tool_version.cyan()
    );

    if let Some(hash) = &state_data.spec_hash {
        println!(
            "  {} {}...",
            "Spec Hash:    ".white().bold(),
            hash[..16.min(hash.len())].dimmed()
        );
    }

    println!();
    print_workflow_progress(&state_data.current_state);

    if verbose && !state_data.history.is_empty() {
        println!();
        println!(
            "{}",
            "┌─────────────────────────────────────────────────────────┐".dimmed()
        );
        println!(
            "{}",
            "│  📜 History (last 10)                                   │".dimmed()
        );
        println!(
            "{}",
            "├─────────────────────────────────────────────────────────┤".dimmed()
        );
        for entry in state_data.recent_history(10) {
            println!(
                "{}  {} {} {} {} {}",
                "│".dimmed(),
                entry.from_state.to_string().yellow(),
                "→".dimmed(),
                entry.to_state.to_string().green(),
                format!("({})", entry.action).dimmed(),
                entry.timestamp.format("%m/%d %H:%M").to_string().dimmed()
            );
        }
        println!(
            "{}",
            "└─────────────────────────────────────────────────────────┘".dimmed()
        );
    }

    // Footer with tips
    println!();
    println!("{}", "─".repeat(50).dimmed());
    println!("{}", next_step_hint(&state_data.current_state).dimmed());
    println!();

    Ok(())
}

fn get_state_icon(state: &State) -> &'static str {
    match state {
        State::Init => "📁",
        State::IntakeCaptured => "📝",
        State::BlueprintDrafted => "📐",
        State::ContractDrafted => "📜",
        State::ContractLocked => "🔒",
        State::PlanCreated => "📋",
        State::BuildInProgress => "🔨",
        State::BuildDone => "✅",
        State::ReviewPassed => "👀",
        State::Shipped => "🚀",
        State::ReviewFailed => "❌",
    }
}

fn next_step_hint(state: &State) -> String {
    match state {
        State::Init => "💡 Next: vibeanvil intake -m \"Your requirements\"".to_string(),
        State::IntakeCaptured => "💡 Next: vibeanvil blueprint --auto".to_string(),
        State::BlueprintDrafted => "💡 Next: vibeanvil contract create".to_string(),
        State::ContractDrafted => "💡 Next: vibeanvil contract lock".to_string(),
        State::ContractLocked => "💡 Next: vibeanvil plan".to_string(),
        State::PlanCreated => "💡 Next: vibeanvil build iterate --max 5".to_string(),
        State::BuildInProgress => "💡 Build in progress...".to_string(),
        State::BuildDone => "💡 Next: vibeanvil review start".to_string(),
        State::ReviewPassed => "💡 Next: vibeanvil ship --tag v1.0.0".to_string(),
        State::Shipped => "🎉 Project shipped! Congratulations!".to_string(),
        State::ReviewFailed => "💡 Next: vibeanvil build iterate (fix issues)".to_string(),
    }
}

fn print_workflow_progress(current: &State) {
    let workflow = [
        (State::Init, "Init", "📁"),
        (State::IntakeCaptured, "Intake", "📝"),
        (State::BlueprintDrafted, "Blueprint", "📐"),
        (State::ContractDrafted, "Contract Draft", "📜"),
        (State::ContractLocked, "Contract Locked", "🔒"),
        (State::PlanCreated, "Plan", "📋"),
        (State::BuildInProgress, "Build", "🔨"),
        (State::BuildDone, "Build Done", "✅"),
        (State::ReviewPassed, "Review", "👀"),
        (State::Shipped, "Shipped", "🚀"),
    ];

    println!("{}", "  Workflow Progress:".white().bold());
    println!();

    for (state, name, icon) in workflow.iter() {
        let is_current = current == state;
        let is_done = current.is_at_least(*state);

        if is_current {
            println!("    {} {} {}", "▶".cyan().bold(), icon, name.cyan().bold());
        } else if is_done {
            println!("    {} {} {}", "✓".green(), icon, name.green());
        } else {
            println!("    {} {} {}", "○".dimmed(), icon, name.dimmed());
        }
    }
}
