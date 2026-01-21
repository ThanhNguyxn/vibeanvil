//! Supermaven provider - generates prompts for Supermaven AI completion
//!
//! Supermaven is a fast AI code completion tool with 1M token context,
//! available as extensions for VS Code, JetBrains, and Neovim.
//!
//! Usage:
//! 1. Install Supermaven extension
//! 2. Use the generated prompt as context
//! 3. Let Supermaven complete your code

use anyhow::Result;
use async_trait::async_trait;
use colored::Colorize;
use std::path::PathBuf;
use tokio::fs;

use super::{Context, Provider, ProviderResponse};

/// Supermaven AI completion provider
pub struct SupermavenProvider;

impl SupermavenProvider {
    pub fn new() -> Self {
        Self
    }

    fn prompt_file_path(context: &Context) -> PathBuf {
        context
            .working_dir
            .join(".vibeanvil")
            .join("sessions")
            .join(&context.session_id)
            .join("supermaven_prompt.md")
    }

    fn generate_prompt_content(prompt: &str, context: &Context) -> String {
        let mut content = String::new();

        content.push_str("# Supermaven Task\n\n");

        // Supermaven-specific instructions
        content.push_str("## Instructions\n\n");
        content.push_str("Supermaven provides fast AI completions:\n");
        content.push_str("- 1M token context window\n");
        content.push_str("- Fast inline completions\n");
        content.push_str("- Use Supermaven Chat for longer tasks\n");
        content.push_str("- Works with VS Code, JetBrains, Neovim\n\n");

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
        content.push_str("```bash\n");
        content.push_str("vibeanvil build manual evidence\n");
        content.push_str("vibeanvil build manual complete\n");
        content.push_str("```\n");

        content
    }
}

impl Default for SupermavenProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Provider for SupermavenProvider {
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
             1. Open the project in your IDE with {}\n\
             2. Use Supermaven Chat or inline completions\n\
             3. Reference prompt: {}\n\
             4. Run: {}\n",
            "⚡ Supermaven Prompt Generated".green().bold(),
            "━".repeat(50).dimmed(),
            format!("Prompt saved to: {}", prompt_path.display()).cyan(),
            "Next steps:".yellow().bold(),
            "Supermaven extension".cyan(),
            prompt_path.display().to_string().cyan(),
            "vibeanvil build manual evidence".cyan(),
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
        "supermaven"
    }

    fn is_available(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supermaven_provider_name() {
        let provider = SupermavenProvider::new();
        assert_eq!(provider.name(), "supermaven");
    }
}
