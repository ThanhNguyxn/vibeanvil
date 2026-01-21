//! GitHub Copilot provider - generates prompts for GitHub Copilot Chat
//!
//! GitHub Copilot is an AI coding assistant integrated into VS Code, JetBrains,
//! Neovim, and other editors. This provider generates optimized prompts for Copilot.
//!
//! Usage:
//! 1. Open GitHub Copilot Chat in your IDE
//! 2. Paste the generated prompt
//! 3. Apply suggested changes
//! 4. Run `vibeanvil build manual evidence` to capture changes

use anyhow::Result;
use async_trait::async_trait;
use colored::Colorize;
use std::path::PathBuf;
use tokio::fs;

use super::{Context, Provider, ProviderResponse};

/// GitHub Copilot provider for IDE-based AI assistance
pub struct CopilotProvider;

impl CopilotProvider {
    pub fn new() -> Self {
        Self
    }

    fn prompt_file_path(context: &Context) -> PathBuf {
        context
            .working_dir
            .join(".vibeanvil")
            .join("sessions")
            .join(&context.session_id)
            .join("copilot_prompt.md")
    }

    fn generate_prompt_content(prompt: &str, context: &Context) -> String {
        let mut content = String::new();

        content.push_str("# GitHub Copilot Task\n\n");

        // Copilot-specific instructions
        content.push_str("## Instructions for Copilot\n\n");
        content.push_str("Please help me with the following coding task. ");
        content.push_str("When making changes:\n");
        content.push_str("- Follow the existing code style and patterns\n");
        content.push_str("- Ensure all tests pass\n");
        content.push_str("- Add comments where helpful\n\n");

        content.push_str("---\n\n");

        content.push_str("## Context\n\n");
        content.push_str(&format!("- **Session**: `{}`\n", context.session_id));
        if let Some(hash) = &context.contract_hash {
            content.push_str(&format!("- **Contract**: `{}`\n", hash));
        }
        content.push('\n');

        content.push_str("---\n\n");

        content.push_str("## Task\n\n");
        content.push_str(prompt);
        content.push_str("\n\n");

        content.push_str("---\n\n");

        content.push_str("## After Changes\n\n");
        content.push_str("Run these commands to complete:\n");
        content.push_str("```bash\n");
        content.push_str("vibeanvil build manual evidence\n");
        content.push_str("vibeanvil build manual complete\n");
        content.push_str("```\n");

        content
    }
}

impl Default for CopilotProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Provider for CopilotProvider {
    async fn execute(&self, prompt: &str, context: &Context) -> Result<ProviderResponse> {
        let prompt_path = Self::prompt_file_path(context);

        // Ensure directory exists
        if let Some(parent) = prompt_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        // Generate and save prompt
        let content = Self::generate_prompt_content(prompt, context);
        fs::write(&prompt_path, &content).await?;

        // Print instructions to user
        let output = format!(
            "\n{}\n\n\
             {}\n\
             {}\n\n\
             {}\n\
             1. Open {} in VS Code/JetBrains\n\
             2. Open prompt file: {}\n\
             3. Copy the content and paste into Copilot Chat\n\
             4. Apply the suggested changes\n\
             5. Run: {}\n\
             6. Run: {}\n",
            "📋 GitHub Copilot Prompt Generated".green().bold(),
            "━".repeat(50).dimmed(),
            format!("Prompt saved to: {}", prompt_path.display()).cyan(),
            "Next steps:".yellow().bold(),
            "GitHub Copilot Chat".cyan(),
            prompt_path.display().to_string().cyan(),
            "vibeanvil build manual evidence".cyan(),
            "vibeanvil build manual complete".cyan(),
        );

        Ok(ProviderResponse {
            success: true,
            output,
            errors: vec![],
            warnings: vec![],
            files_modified: vec![prompt_path.to_string_lossy().to_string()],
        })
    }

    fn name(&self) -> &str {
        "copilot"
    }

    fn is_available(&self) -> bool {
        true // Always available - just generates a file
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_copilot_provider_name() {
        let provider = CopilotProvider::new();
        assert_eq!(provider.name(), "copilot");
    }

    #[test]
    fn test_copilot_is_available() {
        let provider = CopilotProvider::new();
        assert!(provider.is_available());
    }

    #[test]
    fn test_copilot_prompt_content() {
        let context = Context {
            working_dir: PathBuf::from("/test"),
            session_id: "test-session".to_string(),
            contract_hash: Some("abc123".to_string()),
        };

        let content = CopilotProvider::generate_prompt_content("Fix the bug", &context);
        assert!(content.contains("GitHub Copilot"));
        assert!(content.contains("Fix the bug"));
        assert!(content.contains("test-session"));
    }
}
