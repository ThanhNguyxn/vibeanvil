//! Blueprint command handler

use anyhow::Result;
use tokio::fs;

use crate::audit::{AuditLogger, generate_session_id};
use crate::state::State;
use crate::workspace;

pub async fn run(auto: bool) -> Result<()> {
    let state_data = workspace::load_state().await?;

    if !state_data.current_state.is_at_least(State::IntakeCaptured) {
        anyhow::bail!("Intake not captured. Run 'vibeanvil intake' first.");
    }

    if state_data.current_state.is_at_least(State::BlueprintDrafted) {
        println!("Blueprint already drafted. View with 'cat .vibeanvil/blueprints/blueprint.md'");
        return Ok(());
    }

    // Read intake
    let intake_path = workspace::workspace_path().join("intake.md");
    let intake = if intake_path.exists() {
        fs::read_to_string(&intake_path).await?
    } else {
        String::from("No intake found")
    };

    let blueprint_content = if auto {
        generate_blueprint(&intake)
    } else {
        println!("Creating empty blueprint template...");
        default_blueprint()
    };

    // Save blueprint
    let blueprints_dir = workspace::workspace_path().join("blueprints");
    fs::create_dir_all(&blueprints_dir).await?;
    let blueprint_path = blueprints_dir.join("blueprint.md");
    fs::write(&blueprint_path, &blueprint_content).await?;

    // Update state
    let session_id = generate_session_id();
    let mut state_data = workspace::load_state().await?;
    state_data.transition_to(State::BlueprintDrafted, "blueprint", &session_id)?;
    workspace::save_state(&state_data).await?;

    // Audit log
    let logger = AuditLogger::new(&session_id);
    logger.log_state_transition("blueprint", State::IntakeCaptured, State::BlueprintDrafted).await?;

    println!("✓ Blueprint drafted");
    println!("  → Saved to .vibeanvil/blueprints/blueprint.md");
    println!();
    println!("Next: vibeanvil contract create");

    Ok(())
}

fn generate_blueprint(intake: &str) -> String {
    format!(
        r#"# Project Blueprint

**Generated**: {}

## Overview

This blueprint is generated from the project intake.

## Source Intake

{}

## Architecture

<!-- Define the high-level architecture -->

- [ ] Core components
- [ ] Data flow
- [ ] External dependencies

## Technical Stack

<!-- List technology choices -->

## Implementation Phases

### Phase 1: Foundation
- [ ] Project setup
- [ ] Core infrastructure

### Phase 2: Core Features
- [ ] Feature implementation

### Phase 3: Polish
- [ ] Testing
- [ ] Documentation

## Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| TBD  | TBD    | TBD        |

## Success Criteria

- [ ] All requirements met
- [ ] Tests passing
- [ ] Documentation complete
"#,
        chrono::Utc::now().to_rfc3339(),
        intake
    )
}

fn default_blueprint() -> String {
    format!(
        r#"# Project Blueprint

**Created**: {}

## Overview

<!-- Describe the project -->

## Architecture

<!-- Define the high-level architecture -->

## Technical Stack

<!-- List technology choices -->

## Implementation Phases

### Phase 1
- [ ] TBD

## Success Criteria

- [ ] TBD
"#,
        chrono::Utc::now().to_rfc3339()
    )
}
