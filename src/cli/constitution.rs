//! Constitution command - Project principles and governance (from Spec-Kit)
//!
//! Establishes the governing principles and development guidelines for the project.

use anyhow::Result;
use colored::*;

use crate::provider::{get_provider, Context};
use crate::workspace;

/// Run the constitution command to create or view principles
pub async fn run_constitution(
    provider: &str,
    guidelines: Option<&str>,
    view_only: bool,
) -> Result<()> {
    use crate::cli::style;

    style::header("Project Constitution");

    let anvil_dir = workspace::get_anvil_dir()?;
    let constitution_path = anvil_dir.join("constitution.md");

    // View only mode
    if view_only {
        if constitution_path.exists() {
            let content = tokio::fs::read_to_string(&constitution_path).await?;
            println!("\n{}", content);
        } else {
            style::warn("No constitution found. Run 'vibeanvil constitution' to create one.");
        }
        return Ok(());
    }

    // Check if constitution already exists
    if constitution_path.exists() && guidelines.is_none() {
        let content = tokio::fs::read_to_string(&constitution_path).await?;
        println!("\n{}", "Current Constitution:".cyan().bold());
        println!("{}\n", "─".repeat(40).dimmed());
        println!("{}", content);
        style::info("Constitution already exists. Use --guidelines to update.");
        return Ok(());
    }

    // Get guidelines from user or parameter
    let user_guidelines = if let Some(g) = guidelines {
        g.to_string()
    } else {
        // Interactive prompt
        println!("{}", "Enter your project principles and guidelines.".cyan());
        println!(
            "{}",
            "Focus on: code quality, testing, architecture, performance, security.".dimmed()
        );
        println!(
            "{}",
            "(Press Enter twice to finish, or Ctrl+C to cancel)".dimmed()
        );
        println!();

        let mut input = String::new();
        loop {
            let mut line = String::new();
            std::io::stdin().read_line(&mut line)?;
            if line.trim().is_empty() {
                if input.ends_with('\n') {
                    break;
                }
            }
            input.push_str(&line);
        }
        input.trim().to_string()
    };

    if user_guidelines.is_empty() {
        anyhow::bail!("No guidelines provided. Constitution not created.");
    }

    // Load existing context
    let intake = {
        let intake_path = anvil_dir.join("intake.md");
        if intake_path.exists() {
            Some(tokio::fs::read_to_string(&intake_path).await?)
        } else {
            None
        }
    };

    // Build constitution prompt
    let prompt = build_constitution_prompt(&user_guidelines, intake.as_deref());

    style::step("Generating project constitution...");

    let state = workspace::load_state().await?;
    let provider_instance = get_provider(provider)?;
    let context = Context {
        working_dir: std::env::current_dir()?,
        session_id: state.current_session_id.clone().unwrap_or_default(),
        contract_hash: state.spec_hash.clone(),
    };

    let response = provider_instance.execute(&prompt, &context).await?;

    if response.success {
        // Save constitution
        let constitution_content = format!(
            "# Project Constitution\n\nEstablished: {}\n\n---\n\n{}\n",
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
            response.output
        );
        tokio::fs::write(&constitution_path, &constitution_content).await?;

        println!("\n{}", "═".repeat(60).cyan());
        println!("{}", "Project Constitution".cyan().bold());
        println!("{}\n", "═".repeat(60).cyan());
        println!("{}", response.output);

        style::success("Constitution saved to .anvil/constitution.md");
        println!(
            "\n{}",
            "This constitution will guide all subsequent development.".yellow()
        );
    } else {
        for error in &response.errors {
            style::error(error);
        }
    }

    Ok(())
}

fn build_constitution_prompt(guidelines: &str, intake: Option<&str>) -> String {
    let intake_section = intake
        .map(|i| format!("\n## Project Context (from intake):\n{}", i))
        .unwrap_or_default();

    format!(
        r#"You are a software architect. Create a comprehensive project constitution based on the user's guidelines.

## User's Guidelines:
{}
{}

## Create a Project Constitution

The constitution should establish governing principles for all development on this project. Include:

### 1. Core Principles
- What are the fundamental values for this project?
- What trade-offs should developers make (speed vs quality, etc.)?

### 2. Code Quality Standards
- Coding style and conventions
- Documentation requirements
- Complexity limits
- Error handling approaches

### 3. Testing Requirements
- Test coverage expectations
- Types of tests required (unit, integration, e2e)
- Testing conventions

### 4. Architecture Guidelines
- Design patterns to use/avoid
- Module organization
- Dependency management
- API design principles

### 5. Performance Standards
- Performance budgets
- Optimization guidelines
- Caching strategies

### 6. Security Requirements
- Security best practices
- Authentication/authorization patterns
- Data handling rules
- Secrets management

### 7. Review Standards
- Code review requirements
- Approval criteria
- What blocks a merge

### 8. Development Workflow
- Branch naming conventions
- Commit message format
- PR requirements

Format this as a clear, actionable document that can be referenced during development.
Each section should have concrete, specific guidelines (not vague platitudes).
Use bullet points and numbered lists for clarity."#,
        guidelines, intake_section
    )
}

/// Load the constitution if it exists
pub async fn load_constitution() -> Result<Option<String>> {
    let anvil_dir = workspace::get_anvil_dir()?;
    let constitution_path = anvil_dir.join("constitution.md");

    if constitution_path.exists() {
        Ok(Some(tokio::fs::read_to_string(&constitution_path).await?))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_constitution_prompt() {
        let prompt = build_constitution_prompt("Focus on code quality", None);
        assert!(prompt.contains("Focus on code quality"));
        assert!(prompt.contains("Core Principles"));
        assert!(prompt.contains("Testing Requirements"));
    }

    #[test]
    fn test_build_constitution_prompt_with_intake() {
        let prompt = build_constitution_prompt("Focus on code quality", Some("Build a todo app"));
        assert!(prompt.contains("Build a todo app"));
        assert!(prompt.contains("Project Context"));
    }
}
