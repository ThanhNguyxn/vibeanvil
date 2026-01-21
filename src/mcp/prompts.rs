//! MCP Prompts Implementation
//!
//! Exposes VibeAnvil workflow prompt templates via MCP.
//! Prompts allow AI assistants to use predefined templates for common tasks.

use super::protocol::*;

/// Prompt type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromptType {
    Plan,
    Review,
    Architect,
    Developer,
    QA,
    Commit,
    Intake,
    Clarify,
    Implement,
    Debug,
}

impl PromptType {
    /// Get the name of this prompt type
    pub fn name(&self) -> &'static str {
        match self {
            PromptType::Plan => "plan",
            PromptType::Review => "review",
            PromptType::Architect => "architect",
            PromptType::Developer => "developer",
            PromptType::QA => "qa",
            PromptType::Commit => "commit",
            PromptType::Intake => "intake",
            PromptType::Clarify => "clarify",
            PromptType::Implement => "implement",
            PromptType::Debug => "debug",
        }
    }

    /// Get the title of this prompt type
    pub fn title(&self) -> &'static str {
        match self {
            PromptType::Plan => "Generate Implementation Plan",
            PromptType::Review => "Code Review",
            PromptType::Architect => "Architecture Design",
            PromptType::Developer => "Developer Implementation",
            PromptType::QA => "QA Testing Plan",
            PromptType::Commit => "Git Commit Message",
            PromptType::Intake => "Capture Requirements",
            PromptType::Clarify => "Clarify Requirements",
            PromptType::Implement => "Implement Feature",
            PromptType::Debug => "Debug Issue",
        }
    }

    /// Get the description of this prompt type
    pub fn description(&self) -> &'static str {
        match self {
            PromptType::Plan => "Generate a detailed implementation plan for a feature",
            PromptType::Review => "Review code changes and provide feedback",
            PromptType::Architect => "Design system architecture for requirements",
            PromptType::Developer => "Implement code based on plan and contract",
            PromptType::QA => "Create QA testing plan and test cases",
            PromptType::Commit => "Generate a conventional commit message",
            PromptType::Intake => "Capture and structure user requirements",
            PromptType::Clarify => "Ask clarifying questions about requirements",
            PromptType::Implement => "Auto-implement a feature with AI assistance",
            PromptType::Debug => "Debug and fix an issue in the codebase",
        }
    }

    /// Get the arguments for this prompt type
    pub fn arguments(&self) -> Vec<PromptArgument> {
        match self {
            PromptType::Plan => vec![
                PromptArgument {
                    name: "feature".to_string(),
                    description: Some("The feature to plan implementation for".to_string()),
                    required: true,
                },
                PromptArgument {
                    name: "context".to_string(),
                    description: Some(
                        "Additional context from contract or requirements".to_string(),
                    ),
                    required: false,
                },
            ],
            PromptType::Review => vec![
                PromptArgument {
                    name: "files".to_string(),
                    description: Some("Files to review (comma-separated paths)".to_string()),
                    required: false,
                },
                PromptArgument {
                    name: "diff".to_string(),
                    description: Some("Git diff to review".to_string()),
                    required: false,
                },
            ],
            PromptType::Architect => vec![
                PromptArgument {
                    name: "requirements".to_string(),
                    description: Some("System requirements to design for".to_string()),
                    required: true,
                },
                PromptArgument {
                    name: "constraints".to_string(),
                    description: Some("Technical constraints and limitations".to_string()),
                    required: false,
                },
            ],
            PromptType::Developer => vec![
                PromptArgument {
                    name: "task".to_string(),
                    description: Some("The task or feature to implement".to_string()),
                    required: true,
                },
                PromptArgument {
                    name: "files".to_string(),
                    description: Some("Target files to modify".to_string()),
                    required: false,
                },
            ],
            PromptType::QA => vec![
                PromptArgument {
                    name: "feature".to_string(),
                    description: Some("The feature to create tests for".to_string()),
                    required: true,
                },
                PromptArgument {
                    name: "type".to_string(),
                    description: Some("Test type: unit, integration, e2e".to_string()),
                    required: false,
                },
            ],
            PromptType::Commit => vec![
                PromptArgument {
                    name: "changes".to_string(),
                    description: Some("Description of changes or git diff".to_string()),
                    required: true,
                },
                PromptArgument {
                    name: "type".to_string(),
                    description: Some("Commit type: feat, fix, docs, refactor, etc.".to_string()),
                    required: false,
                },
            ],
            PromptType::Intake => vec![PromptArgument {
                name: "request".to_string(),
                description: Some("User's initial request or idea".to_string()),
                required: true,
            }],
            PromptType::Clarify => vec![
                PromptArgument {
                    name: "requirements".to_string(),
                    description: Some("Initial requirements to clarify".to_string()),
                    required: true,
                },
                PromptArgument {
                    name: "questions".to_string(),
                    description: Some("Specific areas needing clarification".to_string()),
                    required: false,
                },
            ],
            PromptType::Implement => vec![
                PromptArgument {
                    name: "task".to_string(),
                    description: Some("The task to implement".to_string()),
                    required: true,
                },
                PromptArgument {
                    name: "plan".to_string(),
                    description: Some("Implementation plan context".to_string()),
                    required: false,
                },
            ],
            PromptType::Debug => vec![
                PromptArgument {
                    name: "issue".to_string(),
                    description: Some("Description of the bug or issue".to_string()),
                    required: true,
                },
                PromptArgument {
                    name: "error".to_string(),
                    description: Some("Error message or stack trace".to_string()),
                    required: false,
                },
            ],
        }
    }

    /// Parse prompt type from name
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "plan" => Some(PromptType::Plan),
            "review" => Some(PromptType::Review),
            "architect" => Some(PromptType::Architect),
            "developer" => Some(PromptType::Developer),
            "qa" => Some(PromptType::QA),
            "commit" => Some(PromptType::Commit),
            "intake" => Some(PromptType::Intake),
            "clarify" => Some(PromptType::Clarify),
            "implement" => Some(PromptType::Implement),
            "debug" => Some(PromptType::Debug),
            _ => None,
        }
    }

    /// Get all prompt types
    pub fn all() -> &'static [PromptType] {
        &[
            PromptType::Plan,
            PromptType::Review,
            PromptType::Architect,
            PromptType::Developer,
            PromptType::QA,
            PromptType::Commit,
            PromptType::Intake,
            PromptType::Clarify,
            PromptType::Implement,
            PromptType::Debug,
        ]
    }
}

/// Prompt registry
pub struct PromptRegistry;

impl PromptRegistry {
    /// List all available prompts
    pub fn list_prompts() -> Vec<PromptDefinition> {
        PromptType::all()
            .iter()
            .map(|pt| PromptDefinition {
                name: pt.name().to_string(),
                description: Some(pt.description().to_string()),
                arguments: pt.arguments(),
            })
            .collect()
    }

    /// Get a prompt by name with arguments
    pub fn get_prompt(
        name: &str,
        arguments: Option<serde_json::Value>,
    ) -> Result<GetPromptResult, String> {
        let prompt_type =
            PromptType::from_name(name).ok_or_else(|| format!("Unknown prompt: {}", name))?;

        let args: std::collections::HashMap<String, String> = arguments
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or_default();

        // Build the prompt message based on type
        let content = Self::build_prompt_content(prompt_type, &args)?;

        Ok(GetPromptResult {
            description: Some(prompt_type.description().to_string()),
            messages: vec![PromptMessage {
                role: MessageRole::User,
                content: Content::Text { text: content },
            }],
        })
    }

    /// Build prompt content from template
    fn build_prompt_content(
        prompt_type: PromptType,
        args: &std::collections::HashMap<String, String>,
    ) -> Result<String, String> {
        // Try to load embedded template first
        let template = crate::prompt::load_template(prompt_type.name()).ok();

        match prompt_type {
            PromptType::Plan => {
                let feature = args
                    .get("feature")
                    .ok_or("Missing required argument: feature")?;
                let context = args.get("context").map(|s| s.as_str()).unwrap_or("");

                if let Some(tpl) = template {
                    Ok(Self::interpolate_template(
                        &tpl,
                        &[("feature", feature), ("context", context)],
                    ))
                } else {
                    let context_str = if context.is_empty() {
                        String::new()
                    } else {
                        format!("Context:\n{}", context)
                    };
                    Ok(format!(
                        "Create a detailed implementation plan for: {}\n\n{}",
                        feature, context_str
                    ))
                }
            }
            PromptType::Review => {
                let files = args.get("files").map(|s| s.as_str()).unwrap_or("");
                let diff = args.get("diff").map(|s| s.as_str()).unwrap_or("");

                if let Some(tpl) = template {
                    Ok(Self::interpolate_template(
                        &tpl,
                        &[("files", files), ("diff", diff)],
                    ))
                } else {
                    Ok(format!(
                        "Review the following code changes:\n\nFiles: {}\n\nDiff:\n{}",
                        files, diff
                    ))
                }
            }
            PromptType::Architect => {
                let requirements = args
                    .get("requirements")
                    .ok_or("Missing required argument: requirements")?;
                let constraints = args.get("constraints").map(|s| s.as_str()).unwrap_or("");

                if let Some(tpl) = template {
                    Ok(Self::interpolate_template(
                        &tpl,
                        &[("requirements", requirements), ("constraints", constraints)],
                    ))
                } else {
                    Ok(format!(
                        "Design the architecture for:\n{}\n\nConstraints:\n{}",
                        requirements, constraints
                    ))
                }
            }
            PromptType::Developer => {
                let task = args.get("task").ok_or("Missing required argument: task")?;
                let files = args.get("files").map(|s| s.as_str()).unwrap_or("");

                if let Some(tpl) = template {
                    Ok(Self::interpolate_template(
                        &tpl,
                        &[("task", task), ("files", files)],
                    ))
                } else {
                    Ok(format!(
                        "Implement the following task:\n{}\n\nTarget files: {}",
                        task, files
                    ))
                }
            }
            PromptType::QA => {
                let feature = args
                    .get("feature")
                    .ok_or("Missing required argument: feature")?;
                let test_type = args.get("type").map(|s| s.as_str()).unwrap_or("unit");

                if let Some(tpl) = template {
                    Ok(Self::interpolate_template(
                        &tpl,
                        &[("feature", feature), ("type", test_type)],
                    ))
                } else {
                    Ok(format!("Create {} tests for: {}", test_type, feature))
                }
            }
            PromptType::Commit => {
                let changes = args
                    .get("changes")
                    .ok_or("Missing required argument: changes")?;
                let commit_type = args.get("type").map(|s| s.as_str()).unwrap_or("feat");

                if let Some(tpl) = template {
                    Ok(Self::interpolate_template(
                        &tpl,
                        &[("changes", changes), ("type", commit_type)],
                    ))
                } else {
                    Ok(format!(
                        "Generate a conventional commit message (type: {}) for:\n{}",
                        commit_type, changes
                    ))
                }
            }
            PromptType::Intake => {
                let request = args
                    .get("request")
                    .ok_or("Missing required argument: request")?;

                Ok(format!(
                    "Capture and structure the following user request into formal requirements:\n\n{}",
                    request
                ))
            }
            PromptType::Clarify => {
                let requirements = args
                    .get("requirements")
                    .ok_or("Missing required argument: requirements")?;
                let questions = args.get("questions").map(|s| s.as_str()).unwrap_or("");

                let questions_str = if questions.is_empty() {
                    String::new()
                } else {
                    format!("Focus on: {}", questions)
                };
                Ok(format!(
                    "Review these requirements and ask clarifying questions:\n\n{}\n\n{}",
                    requirements, questions_str
                ))
            }
            PromptType::Implement => {
                let task = args.get("task").ok_or("Missing required argument: task")?;
                let plan = args.get("plan").map(|s| s.as_str()).unwrap_or("");

                let plan_str = if plan.is_empty() {
                    String::new()
                } else {
                    format!("Based on plan:\n{}", plan)
                };
                Ok(format!(
                    "Implement the following task:\n\n{}\n\n{}",
                    task, plan_str
                ))
            }
            PromptType::Debug => {
                let issue = args
                    .get("issue")
                    .ok_or("Missing required argument: issue")?;
                let error = args.get("error").map(|s| s.as_str()).unwrap_or("");

                let error_str = if error.is_empty() {
                    String::new()
                } else {
                    format!("Error:\n{}", error)
                };
                Ok(format!(
                    "Debug and fix the following issue:\n\n{}\n\n{}",
                    issue, error_str
                ))
            }
        }
    }

    /// Simple template interpolation
    fn interpolate_template(template: &str, vars: &[(&str, &str)]) -> String {
        let mut result = template.to_string();
        for (key, value) in vars {
            result = result.replace(&format!("{{{{{}}}}}", key), value);
            result = result.replace(&format!("{{{{ {} }}}}", key), value);
        }
        result
    }
}

/// Prompt get params
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PromptGetParams {
    pub name: String,
    #[serde(default)]
    pub arguments: Option<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_type_from_name() {
        assert_eq!(PromptType::from_name("plan"), Some(PromptType::Plan));
        assert_eq!(PromptType::from_name("review"), Some(PromptType::Review));
        assert_eq!(PromptType::from_name("invalid"), None);
    }

    #[test]
    fn test_list_prompts() {
        let prompts = PromptRegistry::list_prompts();
        assert!(prompts.len() >= 10);
        assert!(prompts.iter().any(|p| p.name == "plan"));
        assert!(prompts.iter().any(|p| p.name == "review"));
    }

    #[test]
    fn test_get_prompt_plan() {
        let mut args = std::collections::HashMap::new();
        args.insert("feature".to_string(), "user authentication".to_string());

        let result = PromptRegistry::get_prompt("plan", Some(serde_json::to_value(args).unwrap()));

        assert!(result.is_ok());
        let prompt = result.unwrap();
        assert!(prompt.messages.len() == 1);
    }

    #[test]
    fn test_get_prompt_missing_required() {
        let result = PromptRegistry::get_prompt("plan", None);
        assert!(result.is_err());
    }

    #[test]
    fn test_interpolate_template() {
        let template = "Hello {{name}}, welcome to {{ project }}!";
        let result = PromptRegistry::interpolate_template(
            template,
            &[("name", "Alice"), ("project", "VibeAnvil")],
        );
        assert_eq!(result, "Hello Alice, welcome to VibeAnvil!");
    }
}
