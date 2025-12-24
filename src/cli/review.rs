//! Review command handler

use anyhow::Result;

use crate::audit::{generate_session_id, AuditLogger};
use crate::cli::ReviewAction;
use crate::state::State;
use crate::workspace;

pub async fn run(action: ReviewAction) -> Result<()> {
    match action {
        ReviewAction::Start => start_review().await,
        ReviewAction::Pass => pass_review().await,
        ReviewAction::Fail => fail_review().await,
        ReviewAction::Status => show_status().await,
    }
}

async fn start_review() -> Result<()> {
    let state_data = workspace::load_state().await?;

    if !state_data.current_state.is_at_least(State::BuildDone) {
        anyhow::bail!("Build not done. Run 'vibeanvil build' first.");
    }

    println!("📋 Starting review...");
    println!();
    println!("Review checklist:");
    println!("  [ ] Code quality meets standards");
    println!("  [ ] All tests pass");
    println!("  [ ] Documentation updated");
    println!("  [ ] Contract requirements met");
    println!();
    println!("When ready:");
    println!("  vibeanvil review pass   - to approve");
    println!("  vibeanvil review fail   - to reject and retry");

    Ok(())
}

async fn pass_review() -> Result<()> {
    let state_data = workspace::load_state().await?;

    if !state_data.current_state.is_at_least(State::BuildDone) {
        anyhow::bail!("Build not done. Complete build first.");
    }

    if state_data.current_state == State::ReviewPassed {
        println!("Review already passed.");
        return Ok(());
    }

    let session_id = generate_session_id();
    let mut state_data = workspace::load_state().await?;
    state_data.transition_to(State::ReviewPassed, "review pass", &session_id)?;
    workspace::save_state(&state_data).await?;

    let logger = AuditLogger::new(&session_id);
    logger
        .log_state_transition("review pass", State::BuildDone, State::ReviewPassed)
        .await?;

    println!("✓ Review PASSED");
    println!();
    println!("Next steps:");
    println!("  vibeanvil snapshot --message \"Description\"");
    println!("  vibeanvil ship --tag v1.0.0");

    Ok(())
}

async fn fail_review() -> Result<()> {
    let state_data = workspace::load_state().await?;

    if !state_data.current_state.is_at_least(State::BuildDone) {
        anyhow::bail!("Build not done. Complete build first.");
    }

    let session_id = generate_session_id();
    let mut state_data = workspace::load_state().await?;
    state_data.transition_to(State::ReviewFailed, "review fail", &session_id)?;
    workspace::save_state(&state_data).await?;

    let logger = AuditLogger::new(&session_id);
    logger
        .log_state_transition("review fail", State::BuildDone, State::ReviewFailed)
        .await?;

    println!("✗ Review FAILED");
    println!();
    println!("Return to build phase:");
    println!("  vibeanvil build [mode]");

    Ok(())
}

async fn show_status() -> Result<()> {
    let state_data = workspace::load_state().await?;

    println!("Review Status:");
    match state_data.current_state {
        State::ReviewPassed => println!("  ✓ PASSED"),
        State::ReviewFailed => println!("  ✗ FAILED"),
        State::BuildDone => println!("  ⏳ Pending review"),
        _ => println!(
            "  ⚠️  Not ready for review (state: {})",
            state_data.current_state
        ),
    }

    Ok(())
}
