//! Implement command - Execute tasks automatically (from Spec-Kit)
//!
//! Executes the task list to build the feature according to the plan.

use anyhow::Result;
use colored::*;

use crate::cli::tasks::{load_tasks, save_tasks};
use crate::provider::{get_provider, Context};
use crate::workspace;

/// Run the implement command
pub async fn run_implement(
    provider: &str,
    task_id: Option<&str>,
    all: bool,
    dry_run: bool,
) -> Result<()> {
    use crate::cli::style;

    style::header("Implement Tasks");

    // Load tasks
    let mut task_list = load_tasks()
        .await
        .map_err(|_| anyhow::anyhow!("No tasks found. Run 'vibeanvil tasks' first."))?;

    if task_list.tasks.is_empty() {
        style::warn("No tasks to implement.");
        return Ok(());
    }

    // Determine which tasks to run
    let tasks_to_run: Vec<_> = if let Some(id) = task_id {
        // Run specific task
        task_list
            .tasks
            .iter()
            .filter(|t| t.id == id && !t.completed)
            .cloned()
            .collect()
    } else if all {
        // Run all uncompleted tasks
        task_list
            .tasks
            .iter()
            .filter(|t| !t.completed)
            .cloned()
            .collect()
    } else {
        // Run next uncompleted task
        task_list
            .next_task()
            .map(|t| vec![t.clone()])
            .unwrap_or_default()
    };

    if tasks_to_run.is_empty() {
        style::success("All tasks completed! 🎉");
        return Ok(());
    }

    println!(
        "\n{} tasks to implement:\n",
        tasks_to_run.len().to_string().cyan()
    );

    for task in &tasks_to_run {
        println!(
            "  {} [{}] {} [{}]",
            "○".white(),
            task.id.cyan(),
            task.title,
            task.effort.yellow()
        );
    }

    if dry_run {
        style::info("Dry run - no changes will be made.");
        return Ok(());
    }

    // Load context
    let state = workspace::load_state().await?;
    let anvil_dir = workspace::get_anvil_dir()?;

    // Load constitution for guidance
    let constitution = {
        let path = anvil_dir.join("constitution.md");
        if path.exists() {
            Some(tokio::fs::read_to_string(&path).await?)
        } else {
            None
        }
    };

    // Load contract for requirements
    let contract = {
        let path = anvil_dir.join("contract.md");
        if path.exists() {
            Some(tokio::fs::read_to_string(&path).await?)
        } else {
            None
        }
    };

    // Get provider
    let provider_instance = get_provider(provider)?;
    let context = Context {
        working_dir: std::env::current_dir()?,
        session_id: state.current_session_id.clone().unwrap_or_default(),
        contract_hash: state.spec_hash.clone(),
    };

    // Execute each task
    for task in tasks_to_run {
        println!("\n{}", "─".repeat(60).dimmed());
        style::step(&format!("Task [{}]: {}", task.id, task.title));

        let prompt = build_implement_prompt(&task, constitution.as_deref(), contract.as_deref());

        let pb = style::spinner("Implementing...");
        let response = provider_instance.execute(&prompt, &context).await?;
        pb.finish_and_clear();

        if response.success {
            println!("\n{}", response.output);

            // Mark task as complete
            task_list.complete_task(&task.id);
            save_tasks(&task_list).await?;

            style::success(&format!("Task {} completed", task.id));

            // Show progress
            println!(
                "\n{} Progress: {}/{} ({:.0}%)",
                "→".cyan(),
                task_list.completed_count,
                task_list.total_count,
                task_list.progress_percent()
            );
        } else {
            style::error(&format!("Task {} failed", task.id));
            for error in &response.errors {
                style::error(error);
            }

            // Stop on failure unless running all
            if !all {
                return Ok(());
            }
        }
    }

    // Final status
    println!("\n{}", "═".repeat(60).cyan());
    if task_list.completed_count == task_list.total_count {
        style::success("All tasks completed! 🎉");
        println!(
            "\n{}",
            "Run 'vibeanvil review start' to review the implementation.".yellow()
        );
    } else {
        println!(
            "\n{} {}/{} tasks completed",
            "→".cyan(),
            task_list.completed_count,
            task_list.total_count
        );
        if let Some(next) = task_list.next_task() {
            println!("\n{} Next: [{}] {}", "→".cyan(), next.id, next.title);
        }
    }

    Ok(())
}

fn build_implement_prompt(
    task: &crate::cli::tasks::Task,
    constitution: Option<&str>,
    contract: Option<&str>,
) -> String {
    let mut prompt = String::new();

    // Add constitution guidance
    if let Some(c) = constitution {
        prompt.push_str("## Project Constitution (follow these guidelines):\n");
        prompt.push_str(c);
        prompt.push_str("\n\n");
    }

    // Add contract requirements
    if let Some(c) = contract {
        prompt.push_str("## Contract Requirements:\n");
        prompt.push_str(c);
        prompt.push_str("\n\n");
    }

    // Task details
    prompt.push_str(&format!(
        r#"## Task to Implement

**ID**: {}
**Title**: {}
**Effort**: {}
**Description**: {}
**Files**: {}

## Instructions

Implement this task following the project constitution and contract requirements.

1. Create or modify the specified files
2. Write clean, idiomatic code
3. Add appropriate tests
4. Handle edge cases
5. Add documentation/comments as needed

Output the specific file changes with clear diff blocks or full file contents.
"#,
        task.id,
        task.title,
        task.effort,
        task.description,
        task.files.join(", ")
    ));

    prompt
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::tasks::Task;

    #[test]
    fn test_build_implement_prompt() {
        let task = Task {
            id: "1".to_string(),
            title: "Setup project".to_string(),
            description: "Initialize the project".to_string(),
            effort: "small".to_string(),
            files: vec!["package.json".to_string()],
            ..Default::default()
        };

        let prompt = build_implement_prompt(&task, None, None);
        assert!(prompt.contains("Setup project"));
        assert!(prompt.contains("package.json"));
    }

    #[test]
    fn test_build_implement_prompt_with_context() {
        let task = Task {
            id: "1".to_string(),
            title: "Setup project".to_string(),
            ..Default::default()
        };

        let prompt = build_implement_prompt(
            &task,
            Some("Always write tests"),
            Some("Must support feature X"),
        );
        assert!(prompt.contains("Always write tests"));
        assert!(prompt.contains("Must support feature X"));
    }
}
