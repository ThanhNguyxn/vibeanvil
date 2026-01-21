//! Clarify command - Interactive Q&A to clarify requirements (from Spec-Kit)
//!
//! This helps identify underspecified areas in the intake/contract
//! before moving to planning phase.

use anyhow::Result;
use colored::*;

use crate::provider::{get_provider, Context};
use crate::workspace;

/// Run the clarify command
pub async fn run_clarify(provider: &str) -> Result<()> {
    use crate::cli::style;

    style::header("Clarify Requirements");

    // Load current state
    let state = workspace::load_state().await?;

    // Check if we have intake
    if !state
        .current_state
        .is_at_least(crate::state::State::IntakeCaptured)
    {
        anyhow::bail!("No intake captured yet. Run 'vibeanvil intake' first.");
    }

    // Load intake content
    let intake_path = workspace::get_anvil_dir()?.join("intake.md");
    let intake_content = if intake_path.exists() {
        tokio::fs::read_to_string(&intake_path).await?
    } else {
        "No intake file found.".to_string()
    };

    // Load contract if available
    let contract_content = if state
        .current_state
        .is_at_least(crate::state::State::ContractDrafted)
    {
        let contract_path = workspace::get_anvil_dir()?.join("contract.md");
        if contract_path.exists() {
            Some(tokio::fs::read_to_string(&contract_path).await?)
        } else {
            None
        }
    } else {
        None
    };

    // Build clarification prompt
    let prompt = build_clarify_prompt(&intake_content, contract_content.as_deref());

    style::step("Analyzing requirements for gaps...");

    let provider_instance = get_provider(provider)?;
    let context = Context {
        working_dir: std::env::current_dir()?,
        session_id: state.current_session_id.clone().unwrap_or_default(),
        contract_hash: state.spec_hash.clone(),
    };

    let response = provider_instance.execute(&prompt, &context).await?;

    if response.success {
        println!("\n{}", "═".repeat(60).cyan());
        println!("{}", "Clarification Questions".cyan().bold());
        println!("{}\n", "═".repeat(60).cyan());
        println!("{}", response.output);

        // Save clarification output
        let clarify_path = workspace::get_anvil_dir()?.join("clarify.md");
        let clarify_content = format!(
            "# Clarification Questions\n\nGenerated: {}\n\n{}\n",
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
            response.output
        );
        tokio::fs::write(&clarify_path, &clarify_content).await?;

        style::success("Clarification questions saved to .anvil/clarify.md");
        println!(
            "\n{}",
            "Answer these questions to refine your requirements before planning.".yellow()
        );
    } else {
        for error in &response.errors {
            style::error(error);
        }
    }

    Ok(())
}

fn build_clarify_prompt(intake: &str, contract: Option<&str>) -> String {
    let contract_section = contract
        .map(|c| format!("\n## Current Contract:\n{}", c))
        .unwrap_or_default();

    format!(
        r#"You are a requirements analyst. Analyze the following requirements and identify areas that need clarification.

## Intake/Requirements:
{}
{}

## Your Task:
Generate a list of clarifying questions that would help refine these requirements. Focus on:

1. **Ambiguous Requirements**: Where the meaning is unclear or could be interpreted multiple ways
2. **Missing Details**: What information is needed but not specified?
3. **Edge Cases**: What happens in unusual situations?
4. **Technical Constraints**: Are there any constraints not explicitly mentioned?
5. **User Experience**: Any UX considerations that need clarification?
6. **Scale & Performance**: What are the expected usage patterns?
7. **Security**: Any security considerations not addressed?
8. **Integration**: How does this interact with existing systems?

Format your output as a numbered list of questions, grouped by category. For each question:
- Explain WHY this needs clarification
- Suggest 2-3 possible answers if applicable

Be specific and actionable. These questions should help the user refine their requirements before implementation."#,
        intake, contract_section
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_clarify_prompt() {
        let prompt = build_clarify_prompt("Build a todo app", None);
        assert!(prompt.contains("todo app"));
        assert!(prompt.contains("Ambiguous Requirements"));
        assert!(prompt.contains("Missing Details"));
    }

    #[test]
    fn test_build_clarify_prompt_with_contract() {
        let prompt = build_clarify_prompt("Build a todo app", Some("Contract content here"));
        assert!(prompt.contains("Current Contract"));
        assert!(prompt.contains("Contract content here"));
    }
}
