//! Tasks command - Break implementation plan into actionable tasks (from Spec-Kit)
//!
//! Creates a structured task list from the implementation plan for systematic execution.

use anyhow::Result;
use colored::*;
use serde::{Deserialize, Serialize};

use crate::provider::{get_provider, Context};
use crate::workspace;

/// A single task in the task list
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// Task ID (e.g., "1", "1.1", "2")
    pub id: String,
    /// Task title
    pub title: String,
    /// Task description
    pub description: String,
    /// Estimated effort (e.g., "small", "medium", "large")
    pub effort: String,
    /// Dependencies (list of task IDs this depends on)
    pub dependencies: Vec<String>,
    /// Files that will be affected
    pub files: Vec<String>,
    /// Whether task is completed
    pub completed: bool,
}

/// Task list container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskList {
    /// Generated timestamp
    pub generated_at: String,
    /// List of tasks
    pub tasks: Vec<Task>,
    /// Total count
    pub total_count: usize,
    /// Completed count
    pub completed_count: usize,
}

impl Default for TaskList {
    fn default() -> Self {
        Self {
            generated_at: chrono::Utc::now().to_rfc3339(),
            tasks: vec![],
            total_count: 0,
            completed_count: 0,
        }
    }
}

impl TaskList {
    /// Calculate progress percentage
    pub fn progress_percent(&self) -> f32 {
        if self.total_count == 0 {
            0.0
        } else {
            (self.completed_count as f32 / self.total_count as f32) * 100.0
        }
    }

    /// Get next uncompleted task
    pub fn next_task(&self) -> Option<&Task> {
        self.tasks.iter().find(|t| !t.completed)
    }

    /// Mark a task as completed
    pub fn complete_task(&mut self, task_id: &str) -> bool {
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == task_id) {
            task.completed = true;
            self.completed_count = self.tasks.iter().filter(|t| t.completed).count();
            true
        } else {
            false
        }
    }
}

/// Generate tasks from the implementation plan
pub async fn run_tasks(provider: &str, regenerate: bool) -> Result<()> {
    use crate::cli::style;

    style::header("Generate Tasks");

    // Load current state
    let state = workspace::load_state().await?;

    // Check if we have a plan
    if !state
        .current_state
        .is_at_least(crate::state::State::PlanCreated)
    {
        anyhow::bail!("No implementation plan found. Run 'vibeanvil plan' first.");
    }

    let tasks_path = workspace::get_anvil_dir()?.join("tasks.json");

    // Check if tasks already exist
    if tasks_path.exists() && !regenerate {
        style::info("Tasks already generated. Use --regenerate to recreate.");
        let task_list = load_tasks().await?;
        display_tasks(&task_list);
        return Ok(());
    }

    // Load plan content
    let plan_path = workspace::get_anvil_dir()?.join("plan.md");
    let plan_content = if plan_path.exists() {
        tokio::fs::read_to_string(&plan_path).await?
    } else {
        anyhow::bail!("Plan file not found at .anvil/plan.md");
    };

    // Load contract for context
    let contract_path = workspace::get_anvil_dir()?.join("contract.md");
    let contract_content = if contract_path.exists() {
        Some(tokio::fs::read_to_string(&contract_path).await?)
    } else {
        None
    };

    // Build tasks prompt
    let prompt = build_tasks_prompt(&plan_content, contract_content.as_deref());

    style::step("Breaking down plan into tasks...");

    let provider_instance = get_provider(provider)?;
    let context = Context {
        working_dir: std::env::current_dir()?,
        session_id: state.current_session_id.clone().unwrap_or_default(),
        contract_hash: state.spec_hash.clone(),
    };

    let response = provider_instance.execute(&prompt, &context).await?;

    if response.success {
        // Parse the response into tasks
        let tasks = parse_tasks_from_response(&response.output);

        let task_list = TaskList {
            generated_at: chrono::Utc::now().to_rfc3339(),
            total_count: tasks.len(),
            completed_count: 0,
            tasks,
        };

        // Save tasks
        let tasks_json = serde_json::to_string_pretty(&task_list)?;
        tokio::fs::write(&tasks_path, &tasks_json).await?;

        // Also save human-readable version
        let tasks_md = generate_tasks_markdown(&task_list);
        let tasks_md_path = workspace::get_anvil_dir()?.join("tasks.md");
        tokio::fs::write(&tasks_md_path, &tasks_md).await?;

        style::success(&format!(
            "Generated {} tasks. Saved to .anvil/tasks.json",
            task_list.total_count
        ));

        display_tasks(&task_list);
    } else {
        for error in &response.errors {
            style::error(error);
        }
    }

    Ok(())
}

/// Load existing tasks
pub async fn load_tasks() -> Result<TaskList> {
    let tasks_path = workspace::get_anvil_dir()?.join("tasks.json");
    let content = tokio::fs::read_to_string(&tasks_path).await?;
    let task_list: TaskList = serde_json::from_str(&content)?;
    Ok(task_list)
}

/// Save tasks
pub async fn save_tasks(task_list: &TaskList) -> Result<()> {
    let tasks_path = workspace::get_anvil_dir()?.join("tasks.json");
    let tasks_json = serde_json::to_string_pretty(task_list)?;
    tokio::fs::write(&tasks_path, &tasks_json).await?;
    Ok(())
}

/// Mark a task as done
pub async fn complete_task(task_id: &str) -> Result<()> {
    use crate::cli::style;

    let mut task_list = load_tasks().await?;

    if task_list.complete_task(task_id) {
        save_tasks(&task_list).await?;
        style::success(&format!("Task {} marked as complete", task_id));

        // Show progress
        println!(
            "\n{} Progress: {}/{} ({:.0}%)",
            "→".cyan(),
            task_list.completed_count,
            task_list.total_count,
            task_list.progress_percent()
        );

        // Show next task
        if let Some(next) = task_list.next_task() {
            println!("\n{} Next task: {} - {}", "→".cyan(), next.id, next.title);
        } else {
            style::success("All tasks completed! 🎉");
        }
    } else {
        style::error(&format!("Task {} not found", task_id));
    }

    Ok(())
}

/// Display the task list
fn display_tasks(task_list: &TaskList) {
    println!("\n{}", "═".repeat(60).cyan());
    println!(
        "{} ({}/{})",
        "Task List".cyan().bold(),
        task_list.completed_count,
        task_list.total_count
    );
    println!("{}\n", "═".repeat(60).cyan());

    for task in &task_list.tasks {
        let status = if task.completed {
            "✓".green()
        } else {
            "○".white()
        };

        let title = if task.completed {
            task.title.strikethrough().to_string()
        } else {
            task.title.clone()
        };

        let effort_color = match task.effort.as_str() {
            "small" => task.effort.green(),
            "medium" => task.effort.yellow(),
            "large" => task.effort.red(),
            _ => task.effort.white(),
        };

        println!(
            "{} [{}] {} [{}]",
            status,
            task.id.cyan(),
            title,
            effort_color
        );

        if !task.dependencies.is_empty() {
            println!(
                "    └─ depends on: {}",
                task.dependencies.join(", ").dimmed()
            );
        }
    }

    // Progress bar
    let progress = task_list.progress_percent();
    let filled = (progress / 5.0) as usize;
    let empty = 20 - filled;
    println!(
        "\n[{}{}] {:.0}%",
        "█".repeat(filled).green(),
        "░".repeat(empty).dimmed(),
        progress
    );
}

fn build_tasks_prompt(plan: &str, contract: Option<&str>) -> String {
    let contract_section = contract
        .map(|c| format!("\n## Contract:\n{}", c))
        .unwrap_or_default();

    format!(
        r#"You are a project manager. Break down the following implementation plan into specific, actionable tasks.

## Implementation Plan:
{}
{}

## Your Task:
Generate a list of tasks that can be executed one by one to implement the plan.

For each task, provide:
1. **ID**: Hierarchical numbering (1, 1.1, 1.2, 2, 2.1, etc.)
2. **Title**: Brief task title (max 60 chars)
3. **Description**: What needs to be done
4. **Effort**: "small" (< 30 min), "medium" (30 min - 2 hours), "large" (> 2 hours)
5. **Dependencies**: List of task IDs that must be completed first
6. **Files**: List of files that will be created or modified

Format each task like this:
```
### Task [ID]: [Title]
**Effort**: [effort]
**Dependencies**: [comma-separated IDs or "none"]
**Files**: [comma-separated file paths]

[Description]
```

Guidelines:
- Tasks should be small and focused (aim for "small" or "medium" effort)
- Large tasks should be broken into sub-tasks
- Order tasks logically (setup → core → features → tests → polish)
- Include testing tasks
- Include documentation tasks if needed"#,
        plan, contract_section
    )
}

fn parse_tasks_from_response(response: &str) -> Vec<Task> {
    let mut tasks = Vec::new();
    let mut current_task: Option<Task> = None;

    for line in response.lines() {
        let line = line.trim();

        if line.starts_with("### Task ") {
            // Save previous task
            if let Some(task) = current_task.take() {
                tasks.push(task);
            }

            // Parse new task header: "### Task [ID]: [Title]"
            if let Some(rest) = line.strip_prefix("### Task ") {
                let parts: Vec<&str> = rest.splitn(2, ':').collect();
                if parts.len() == 2 {
                    current_task = Some(Task {
                        id: parts[0].trim().trim_matches(['[', ']']).to_string(),
                        title: parts[1].trim().to_string(),
                        description: String::new(),
                        effort: "medium".to_string(),
                        dependencies: vec![],
                        files: vec![],
                        completed: false,
                    });
                }
            }
        } else if let Some(ref mut task) = current_task {
            if line.starts_with("**Effort**:") {
                task.effort = line
                    .strip_prefix("**Effort**:")
                    .unwrap_or("")
                    .trim()
                    .to_lowercase();
            } else if line.starts_with("**Dependencies**:") {
                let deps = line.strip_prefix("**Dependencies**:").unwrap_or("").trim();
                if deps.to_lowercase() != "none" && !deps.is_empty() {
                    task.dependencies = deps
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                }
            } else if line.starts_with("**Files**:") {
                let files = line.strip_prefix("**Files**:").unwrap_or("").trim();
                if !files.is_empty() {
                    task.files = files
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                }
            } else if !line.starts_with("```") && !line.is_empty() && !line.starts_with("**") {
                if !task.description.is_empty() {
                    task.description.push('\n');
                }
                task.description.push_str(line);
            }
        }
    }

    // Don't forget the last task
    if let Some(task) = current_task {
        tasks.push(task);
    }

    tasks
}

fn generate_tasks_markdown(task_list: &TaskList) -> String {
    let mut md = String::from("# Implementation Tasks\n\n");
    md.push_str(&format!("Generated: {}\n\n", task_list.generated_at));
    md.push_str(&format!(
        "Progress: {}/{} ({:.0}%)\n\n",
        task_list.completed_count,
        task_list.total_count,
        task_list.progress_percent()
    ));

    for task in &task_list.tasks {
        let status = if task.completed { "- [x]" } else { "- [ ]" };
        md.push_str(&format!("{} **{}**: {}\n", status, task.id, task.title));
        md.push_str(&format!("  - Effort: {}\n", task.effort));

        if !task.dependencies.is_empty() {
            md.push_str(&format!(
                "  - Dependencies: {}\n",
                task.dependencies.join(", ")
            ));
        }

        if !task.files.is_empty() {
            md.push_str(&format!("  - Files: {}\n", task.files.join(", ")));
        }

        if !task.description.is_empty() {
            md.push_str(&format!(
                "  - {}\n",
                task.description.replace('\n', "\n    ")
            ));
        }

        md.push('\n');
    }

    md
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_list_progress() {
        let task_list = TaskList {
            total_count: 10,
            completed_count: 3,
            ..Default::default()
        };
        // Use approximate comparison for floating point
        let progress = task_list.progress_percent();
        assert!((progress - 30.0).abs() < 0.001);
    }

    #[test]
    fn test_parse_tasks() {
        let response = r#"
### Task [1]: Setup project
**Effort**: small
**Dependencies**: none
**Files**: package.json, tsconfig.json

Initialize the project structure.

### Task [2]: Implement core
**Effort**: large
**Dependencies**: 1
**Files**: src/core.ts

Implement the core functionality.
"#;

        let tasks = parse_tasks_from_response(response);
        assert_eq!(tasks.len(), 2);
        assert_eq!(tasks[0].id, "1");
        assert_eq!(tasks[0].title, "Setup project");
        assert_eq!(tasks[0].effort, "small");
        assert_eq!(tasks[1].dependencies, vec!["1"]);
    }

    #[test]
    fn test_complete_task() {
        let mut task_list = TaskList {
            tasks: vec![
                Task {
                    id: "1".to_string(),
                    completed: false,
                    ..Default::default()
                },
                Task {
                    id: "2".to_string(),
                    completed: false,
                    ..Default::default()
                },
            ],
            total_count: 2,
            completed_count: 0,
            ..Default::default()
        };

        assert!(task_list.complete_task("1"));
        assert_eq!(task_list.completed_count, 1);
        assert!(task_list.tasks[0].completed);
    }
}

impl Default for Task {
    fn default() -> Self {
        Self {
            id: String::new(),
            title: String::new(),
            description: String::new(),
            effort: "medium".to_string(),
            dependencies: vec![],
            files: vec![],
            completed: false,
        }
    }
}
