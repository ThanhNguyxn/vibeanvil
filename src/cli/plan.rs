//! Plan command handler

use anyhow::Result;
use tokio::fs;

use crate::audit::{generate_session_id, AuditLogger};
use crate::provider::{get_provider, Context};
use crate::state::State;
use crate::workspace;
use colored::*;

pub async fn run(provider_name: String) -> Result<()> {
    let state_data = workspace::load_state().await?;

    if !state_data.current_state.is_at_least(State::ContractLocked) {
        anyhow::bail!("Contract not locked. Run 'vibeanvil contract lock' first.");
    }

    if state_data.current_state.is_at_least(State::PlanCreated) {
        println!("Plan already created. View with 'cat .vibeanvil/plan.md'");
        return Ok(());
    }

    // Read contract
    let contract_path = workspace::contracts_path().join("contract.json");
    let contract = fs::read_to_string(&contract_path).await?;

    println!(
        "📋 Creating implementation plan with {} provider...",
        provider_name
    );

    let provider = get_provider(&provider_name)?;

    // Scan repository map
    crate::cli::style::step("Context Awareness");
    let mut repo_map = crate::brain::map::RepositoryMap::new();
    let workspace_path = workspace::workspace_path();
    let root = workspace_path.parent().unwrap_or(std::path::Path::new("."));

    if let Err(e) = repo_map.scan(root) {
        crate::cli::style::warn(&format!("Failed to scan repository: {}", e));
    } else {
        crate::cli::style::success(&format!(
            "Repository map generated ({} files)",
            repo_map.files.len()
        ));
    }

    if !provider.is_available() {
        println!(
            "⚠️  Provider '{}' not available, generating template plan.",
            provider_name
        );
        let plan = generate_template_plan(&contract);
        save_plan(&plan).await?;
    } else {
        let session_id = generate_session_id();
        let context = Context {
            working_dir: std::env::current_dir()?,
            session_id: session_id.clone(),
            contract_hash: state_data.spec_hash.clone(),
        };

        let map_markdown = repo_map.to_markdown();
        let prompt = format!(
            "Based on this contract and the current codebase structure, create a detailed implementation plan.\n\nCONTRACT:\n{}\n\nCODEBASE STRUCTURE:\n{}",
            contract,
            map_markdown
        );

        println!("{}", "🤖 Generating plan with AI...".cyan());
        let response = provider.execute(&prompt, &context).await?;
        save_plan(&response.output).await?;
    }

    // Update state
    let session_id = generate_session_id();
    let mut state_data = workspace::load_state().await?;
    state_data.transition_to(State::PlanCreated, "plan", &session_id)?;
    workspace::save_state(&state_data).await?;

    // Audit
    let logger = AuditLogger::new(&session_id);
    logger
        .log_state_transition("plan", State::ContractLocked, State::PlanCreated)
        .await?;

    println!("✓ Implementation plan created");
    println!("  → View at .vibeanvil/plan.md");
    println!();
    println!("Next: vibeanvil build [manual|auto|iterate]");

    Ok(())
}

async fn save_plan(content: &str) -> Result<()> {
    let plan_path = workspace::workspace_path().join("plan.md");
    fs::write(&plan_path, content).await?;
    Ok(())
}

fn generate_template_plan(contract: &str) -> String {
    format!(
        r#"# Implementation Plan

**Generated**: {}

## Contract Reference

```json
{}
```

## Implementation Steps

### Step 1: Project Setup
- [ ] Initialize project structure
- [ ] Configure dependencies
- [ ] Set up development environment

### Step 2: Core Implementation
- [ ] Implement core features
- [ ] Add unit tests

### Step 3: Integration
- [ ] Integrate components
- [ ] Add integration tests

### Step 4: Polish
- [ ] Documentation
- [ ] Final testing
- [ ] Review

## Estimated Timeline

| Phase | Duration | Owner |
|-------|----------|-------|
| Setup | 1 day    | TBD   |
| Core  | TBD      | TBD   |
| Integration | TBD | TBD |
| Polish | TBD     | TBD   |

## Dependencies

- None identified

## Risks

- TBD
"#,
        chrono::Utc::now().to_rfc3339(),
        contract
    )
}
