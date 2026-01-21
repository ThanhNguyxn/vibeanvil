//! JetBrains AI provider - generates prompts for JetBrains AI Assistant
//!
//! JetBrains AI Assistant is built into IntelliJ IDEA, PyCharm, WebStorm,
//! and other JetBrains IDEs. This provider generates optimized prompts.
//!
//! Usage:
//! 1. Open project in JetBrains IDE
//! 2. Use AI Assistant (Alt+Enter or dedicated panel)
//! 3. Paste the prompt
//! 4. Apply suggested changes

use anyhow::Result;
use async_trait::async_trait;
use colored::Colorize;
use std::path::PathBuf;
use tokio::fs;

use super::{Context, Provider, ProviderResponse};

/// JetBrains AI Assistant provider
pub struct JetbrainsProvider;

impl JetbrainsProvider {
    pub fn new() -> Self {
        Self
    }

    fn prompt_file_path(context: &Context) -> PathBuf {
        context
            .working_dir
            .join(".vibeanvil")
            .join("sessions")
            .join(&context.session_id)
            .join("jetbrains_prompt.md")
    }

    fn generate_prompt_content(prompt: &str, context: &Context) -> String {
        let mut content = String::new();

        content.push_str("# JetBrains AI Assistant Task\n\n");

        // JetBrains-specific instructions
        content.push_str("## Instructions\n\n");
        content.push_str("Use JetBrains AI Assistant:\n");
        content.push_str("- Press `Alt+Enter` on code for AI suggestions\n");
        content.push_str("- Use AI Chat panel for conversations\n");
        content.push_str("- Use `/explain`, `/refactor`, `/tests` commands\n");
        content.push_str("- AI can generate code, write tests, explain logic\n\n");

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
        content.push_str("Run in terminal:\n");
        content.push_str("```bash\n");
        content.push_str("vibeanvil build manual evidence\n");
        content.push_str("vibeanvil build manual complete\n");
        content.push_str("```\n");

        content
    }
}

impl Default for JetbrainsProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Provider for JetbrainsProvider {
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
             2. Open AI Chat or press {} on code\n\
             3. Paste the prompt from: {}\n\
             4. Apply the suggested changes\n\
             5. Run: {}\n",
            "🧠 JetBrains AI Prompt Generated".green().bold(),
            "━".repeat(50).dimmed(),
            format!("Prompt saved to: {}", prompt_path.display()).cyan(),
            "Next steps:".yellow().bold(),
            "JetBrains IDE".cyan(),
            "Alt+Enter".cyan(),
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
        "jetbrains"
    }

    fn is_available(&self) -> bool {
        true // Always available - just generates a file
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jetbrains_provider_name() {
        let provider = JetbrainsProvider::new();
        assert_eq!(provider.name(), "jetbrains");
    }

    #[test]
    fn test_jetbrains_prompt_content() {
        let context = Context {
            working_dir: PathBuf::from("/test"),
            session_id: "test-session".to_string(),
            contract_hash: None,
        };

        let content = JetbrainsProvider::generate_prompt_content("Refactor code", &context);
        assert!(content.contains("JetBrains"));
        assert!(content.contains("Alt+Enter"));
        assert!(content.contains("Refactor code"));
    }
}
