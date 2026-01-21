//! Analyze command - Cross-artifact consistency check (from Spec-Kit)
//!
//! Validates that intake, contract, plan, and tasks are all aligned.

use anyhow::Result;
use colored::*;

use crate::provider::{get_provider, Context};
use crate::workspace;

/// Analysis result for a single artifact
#[derive(Debug)]
pub struct ArtifactAnalysis {
    pub name: String,
    pub exists: bool,
    pub issues: Vec<String>,
    pub suggestions: Vec<String>,
}

/// Full analysis result
#[derive(Debug)]
pub struct AnalysisResult {
    pub artifacts: Vec<ArtifactAnalysis>,
    pub consistency_issues: Vec<String>,
    pub coverage_gaps: Vec<String>,
    pub recommendations: Vec<String>,
    pub overall_score: u8, // 0-100
}

/// Run the analyze command
pub async fn run_analyze(provider: &str) -> Result<()> {
    use crate::cli::style;

    style::header("Analyze Artifacts");

    // Load current state
    let state = workspace::load_state().await?;

    // Check minimum state
    if !state
        .current_state
        .is_at_least(crate::state::State::IntakeCaptured)
    {
        anyhow::bail!("No intake captured yet. Run 'vibeanvil intake' first.");
    }

    // Load all artifacts
    let anvil_dir = workspace::get_anvil_dir()?;

    let intake = load_artifact(&anvil_dir, "intake.md").await;
    let blueprint = load_artifact(&anvil_dir, "blueprint.md").await;
    let contract = load_artifact(&anvil_dir, "contract.md").await;
    let plan = load_artifact(&anvil_dir, "plan.md").await;
    let tasks = load_artifact(&anvil_dir, "tasks.md").await;
    let clarify = load_artifact(&anvil_dir, "clarify.md").await;

    // Build analysis prompt
    let prompt = build_analyze_prompt(&intake, &blueprint, &contract, &plan, &tasks, &clarify);

    style::step("Analyzing artifact consistency...");

    let provider_instance = get_provider(provider)?;
    let context = Context {
        working_dir: std::env::current_dir()?,
        session_id: state.current_session_id.clone().unwrap_or_default(),
        contract_hash: state.spec_hash.clone(),
    };

    let response = provider_instance.execute(&prompt, &context).await?;

    if response.success {
        println!("\n{}", "═".repeat(60).cyan());
        println!("{}", "Analysis Report".cyan().bold());
        println!("{}\n", "═".repeat(60).cyan());

        // Show artifact status
        display_artifact_status(&intake, "Intake");
        display_artifact_status(&blueprint, "Blueprint");
        display_artifact_status(&contract, "Contract");
        display_artifact_status(&plan, "Plan");
        display_artifact_status(&tasks, "Tasks");
        display_artifact_status(&clarify, "Clarify");

        println!("\n{}", "─".repeat(60).dimmed());
        println!("{}", "Analysis Results".cyan().bold());
        println!("{}\n", "─".repeat(60).dimmed());

        println!("{}", response.output);

        // Save analysis
        let analysis_path = anvil_dir.join("analysis.md");
        let analysis_content = format!(
            "# Artifact Analysis\n\nGenerated: {}\n\n## Artifacts Present\n\n{}\n\n## Analysis\n\n{}\n",
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
            format_artifact_list(&intake, &blueprint, &contract, &plan, &tasks, &clarify),
            response.output
        );
        tokio::fs::write(&analysis_path, &analysis_content).await?;

        style::success("Analysis saved to .anvil/analysis.md");
    } else {
        for error in &response.errors {
            style::error(error);
        }
    }

    Ok(())
}

async fn load_artifact(anvil_dir: &std::path::Path, filename: &str) -> Option<String> {
    let path = anvil_dir.join(filename);
    if path.exists() {
        tokio::fs::read_to_string(&path).await.ok()
    } else {
        None
    }
}

fn display_artifact_status(artifact: &Option<String>, name: &str) {
    if let Some(content) = artifact {
        let lines = content.lines().count();
        println!(
            "  {} {} ({} lines)",
            "✓".green(),
            name,
            lines.to_string().dimmed()
        );
    } else {
        println!("  {} {} {}", "○".dimmed(), name, "(not created)".dimmed());
    }
}

fn format_artifact_list(
    intake: &Option<String>,
    blueprint: &Option<String>,
    contract: &Option<String>,
    plan: &Option<String>,
    tasks: &Option<String>,
    clarify: &Option<String>,
) -> String {
    let mut list = Vec::new();

    if intake.is_some() {
        list.push("- [x] Intake");
    } else {
        list.push("- [ ] Intake");
    }

    if blueprint.is_some() {
        list.push("- [x] Blueprint");
    } else {
        list.push("- [ ] Blueprint");
    }

    if contract.is_some() {
        list.push("- [x] Contract");
    } else {
        list.push("- [ ] Contract");
    }

    if plan.is_some() {
        list.push("- [x] Plan");
    } else {
        list.push("- [ ] Plan");
    }

    if tasks.is_some() {
        list.push("- [x] Tasks");
    } else {
        list.push("- [ ] Tasks");
    }

    if clarify.is_some() {
        list.push("- [x] Clarify");
    } else {
        list.push("- [ ] Clarify");
    }

    list.join("\n")
}

fn build_analyze_prompt(
    intake: &Option<String>,
    blueprint: &Option<String>,
    contract: &Option<String>,
    plan: &Option<String>,
    tasks: &Option<String>,
    clarify: &Option<String>,
) -> String {
    let mut artifacts = String::new();

    if let Some(content) = intake {
        artifacts.push_str(&format!("## INTAKE:\n{}\n\n", content));
    }

    if let Some(content) = blueprint {
        artifacts.push_str(&format!("## BLUEPRINT:\n{}\n\n", content));
    }

    if let Some(content) = contract {
        artifacts.push_str(&format!("## CONTRACT:\n{}\n\n", content));
    }

    if let Some(content) = plan {
        artifacts.push_str(&format!("## PLAN:\n{}\n\n", content));
    }

    if let Some(content) = tasks {
        artifacts.push_str(&format!("## TASKS:\n{}\n\n", content));
    }

    if let Some(content) = clarify {
        artifacts.push_str(&format!("## CLARIFICATION NOTES:\n{}\n\n", content));
    }

    format!(
        r#"You are a project analyst. Analyze the following project artifacts for consistency and completeness.

{}

## Your Analysis:

Provide a comprehensive analysis covering:

### 1. Consistency Check
- Are all artifacts aligned with each other?
- Does the plan address everything in the contract?
- Do the tasks cover everything in the plan?
- Are there contradictions between artifacts?

### 2. Coverage Analysis
- What requirements from intake are not addressed?
- What parts of the contract have no corresponding plan items?
- What plan items have no corresponding tasks?

### 3. Gap Identification
- What's missing from each artifact?
- What edge cases are not covered?
- What non-functional requirements are overlooked?

### 4. Risk Assessment
- What are the high-risk areas?
- What dependencies could cause delays?
- What technical challenges are underestimated?

### 5. Recommendations
- What should be added or clarified?
- What order should tasks be executed?
- What should be prioritized?

### 6. Overall Score
Give an overall readiness score from 0-100, where:
- 0-40: Major gaps, not ready to proceed
- 41-60: Some gaps, needs refinement
- 61-80: Minor gaps, can proceed with caution
- 81-100: Well-defined, ready to implement

Format your output clearly with headers and bullet points."#,
        artifacts
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_artifact_list() {
        let list = format_artifact_list(
            &Some("intake".to_string()),
            &None,
            &Some("contract".to_string()),
            &None,
            &None,
            &None,
        );
        assert!(list.contains("[x] Intake"));
        assert!(list.contains("[ ] Blueprint"));
        assert!(list.contains("[x] Contract"));
    }

    #[test]
    fn test_build_analyze_prompt() {
        let prompt = build_analyze_prompt(
            &Some("intake content".to_string()),
            &None,
            &Some("contract content".to_string()),
            &None,
            &None,
            &None,
        );
        assert!(prompt.contains("INTAKE:"));
        assert!(prompt.contains("intake content"));
        assert!(prompt.contains("CONTRACT:"));
        assert!(!prompt.contains("BLUEPRINT:"));
    }
}
