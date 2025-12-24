//! Contract command handler

use anyhow::Result;
use tokio::fs;

use crate::audit::{AuditLogger, generate_session_id};
use crate::cli::ContractAction;
use crate::contract::{self, Contract, Priority};
use crate::state::State;
use crate::workspace;

pub async fn run(action: ContractAction) -> Result<()> {
    match action {
        ContractAction::Create => create_contract().await,
        ContractAction::Validate => validate_contract().await,
        ContractAction::Lock => lock_contract().await,
        ContractAction::Status => show_status().await,
    }
}

async fn create_contract() -> Result<()> {
    let state_data = workspace::load_state().await?;

    if !state_data.current_state.is_at_least(State::BlueprintDrafted) {
        anyhow::bail!("Blueprint not drafted. Run 'vibeanvil blueprint' first.");
    }

    if state_data.current_state.is_at_least(State::ContractDrafted) {
        println!("Contract already exists. View with 'cat .vibeanvil/contracts/contract.json'");
        return Ok(());
    }

    // Read blueprint for context
    let blueprint_path = workspace::workspace_path().join("blueprints/blueprint.md");
    let _blueprint = if blueprint_path.exists() {
        fs::read_to_string(&blueprint_path).await?
    } else {
        String::new()
    };

    // Create contract
    let mut contract = Contract::new("Project Name");
    contract.description = "Project description from intake and blueprint".to_string();
    contract.add_goal("Primary project goal");
    contract.add_requirement("REQ-001", "First requirement", Priority::Must);
    contract.add_acceptance_criterion("Acceptance criterion 1");

    contract::save_contract(&contract).await?;

    // Update state
    let session_id = generate_session_id();
    let mut state_data = workspace::load_state().await?;
    state_data.transition_to(State::ContractDrafted, "contract create", &session_id)?;
    workspace::save_state(&state_data).await?;

    // Audit
    let logger = AuditLogger::new(&session_id);
    logger.log_state_transition("contract create", State::BlueprintDrafted, State::ContractDrafted).await?;

    println!("✓ Contract created");
    println!("  → Edit at .vibeanvil/contracts/contract.json");
    println!();
    println!("Next: vibeanvil contract validate");
    println!("Then: vibeanvil contract lock");

    Ok(())
}

async fn validate_contract() -> Result<()> {
    let contract = contract::load_contract().await?;
    let validation = contract.validate();

    if validation.valid {
        println!("✓ Contract is valid");
    } else {
        println!("✗ Contract validation failed:");
        for error in &validation.errors {
            println!("  - {}", error);
        }
    }

    if !validation.warnings.is_empty() {
        println!();
        println!("Warnings:");
        for warning in &validation.warnings {
            println!("  ⚠️  {}", warning);
        }
    }

    Ok(())
}

async fn lock_contract() -> Result<()> {
    let state_data = workspace::load_state().await?;

    if !state_data.current_state.is_at_least(State::ContractDrafted) {
        anyhow::bail!("No contract to lock. Run 'vibeanvil contract create' first.");
    }

    if state_data.current_state.is_at_least(State::ContractLocked) {
        let lock = contract::load_lock().await?;
        println!("Contract already locked.");
        println!("  Hash: {}", lock.hash);
        println!("  Locked at: {}", lock.locked_at);
        return Ok(());
    }

    let mut contract = contract::load_contract().await?;
    let tool_version = env!("CARGO_PKG_VERSION");
    
    let lock = contract.lock(tool_version)?;
    contract::save_contract(&contract).await?;
    contract::save_lock(&lock).await?;

    // Update state
    let session_id = generate_session_id();
    let mut state_data = workspace::load_state().await?;
    state_data.spec_hash = Some(lock.hash.clone());
    state_data.transition_to(State::ContractLocked, "contract lock", &session_id)?;
    workspace::save_state(&state_data).await?;

    // Audit
    let logger = AuditLogger::new(&session_id);
    logger.log_state_transition("contract lock", State::ContractDrafted, State::ContractLocked).await?;

    println!("🔒 Contract LOCKED");
    println!();
    println!("  Spec Hash: {}", lock.hash);
    println!("  Locked at: {}", lock.locked_at);
    println!();
    println!("\"Contract LOCKED = License to Build\"");
    println!();
    println!("Next: vibeanvil plan");

    Ok(())
}

async fn show_status() -> Result<()> {
    match contract::load_contract().await {
        Ok(contract) => {
            println!("Contract Status: {:?}", contract.status);
            println!("Project: {}", contract.project_name);
            println!("Goals: {}", contract.goals.len());
            println!("Requirements: {}", contract.requirements.len());
            
            if contract.is_locked() {
                if let Ok(lock) = contract::load_lock().await {
                    println!();
                    println!("Lock Hash: {}", lock.hash);
                    println!("Locked at: {}", lock.locked_at);
                }
            }
        }
        Err(_) => {
            println!("No contract found. Run 'vibeanvil contract create'");
        }
    }

    Ok(())
}
