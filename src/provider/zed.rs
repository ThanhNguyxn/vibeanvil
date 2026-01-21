//! Zed AI provider - generates prompts for Zed's built-in AI assistant
//!
//! Zed is a high-performance code editor with built-in AI features.
//! This provider generates optimized prompts for Zed's AI assistant.
//!
//! Usage:
//! 1. Open Zed editor
//! 2. Use Cmd+Enter (Mac) or Ctrl+Enter to open AI assistant
//! 3. Paste the generated prompt
//! 4. Apply suggested changes

use anyhow::Result;
use async_trait::async_trait;
use colored::Colorize;
use std::path::PathBuf;
use tokio::fs;

use super::{Context, Provider, ProviderResponse};

/// Zed AI provider for Zed editor's built-in assistant
pub struct ZedProvider;

impl ZedProvider {
    pub fn new() -> Self {
        Self
    }

    fn prompt_file_path(context: &Context) -> PathBuf {
        context
            .working_dir
            .join(".vibeanvil")
            .join("sessions")
            .join(&context.session_id)
            .join("zed_prompt.md")
    }

    fn generate_prompt_content(prompt: &str, context: &Context) -> String {
        let mut content = String::new();

        content.push_str("# Zed AI Task\n\n");

        // Zed-specific instructions
        content.push_str("## Instructions\n\n");
        content.push_str("Use Zed's AI assistant to help with this task:\n");
        content.push_str("- Press `Cmd+Enter` (Mac) or `Ctrl+Enter` (Linux) to open AI\n");
        content.push_str("- Use `/file` to include relevant files as context\n");
        content.push_str("- Use `/tab` to include open tabs\n\n");

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

impl Default for ZedProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Provider for ZedProvider {
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
             1. Open the project in {}\n\
             2. Open prompt file: {}\n\
             3. Press {} to open AI assistant\n\
             4. Paste the prompt content\n\
             5. Apply the suggested changes\n\
             6. Run: {}\n",
            "⚡ Zed AI Prompt Generated".green().bold(),
            "━".repeat(50).dimmed(),
            format!("Prompt saved to: {}", prompt_path.display()).cyan(),
            "Next steps:".yellow().bold(),
            "Zed".cyan(),
            prompt_path.display().to_string().cyan(),
            "Cmd+Enter / Ctrl+Enter".cyan(),
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
        "zed"
    }

    fn is_available(&self) -> bool {
        true // Always available - just generates a file
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zed_provider_name() {
        let provider = ZedProvider::new();
        assert_eq!(provider.name(), "zed");
    }

    #[test]
    fn test_zed_prompt_content() {
        let context = Context {
            working_dir: PathBuf::from("/test"),
            session_id: "test-session".to_string(),
            contract_hash: None,
        };

        let content = ZedProvider::generate_prompt_content("Add feature", &context);
        assert!(content.contains("Zed AI"));
        assert!(content.contains("Cmd+Enter"));
        assert!(content.contains("Add feature"));
    }
}
