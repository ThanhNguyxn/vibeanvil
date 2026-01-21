//! Windsurf provider - generates prompts for Codeium's Windsurf IDE
//!
//! Windsurf is Codeium's AI-first IDE with Cascade (agentic mode) and
//! Flow (inline editing). This provider generates optimized prompts.
//!
//! Usage:
//! 1. Open project in Windsurf
//! 2. Press Cmd+I to open Cascade
//! 3. Paste the prompt
//! 4. Let Windsurf make changes

use anyhow::Result;
use async_trait::async_trait;
use colored::Colorize;
use std::path::PathBuf;
use tokio::fs;

use super::{Context, Provider, ProviderResponse};

/// Windsurf provider for Codeium's AI IDE
pub struct WindsurfProvider;

impl WindsurfProvider {
    pub fn new() -> Self {
        Self
    }

    fn prompt_file_path(context: &Context) -> PathBuf {
        context
            .working_dir
            .join(".vibeanvil")
            .join("sessions")
            .join(&context.session_id)
            .join("windsurf_prompt.md")
    }

    fn generate_prompt_content(prompt: &str, context: &Context) -> String {
        let mut content = String::new();

        content.push_str("# Windsurf Cascade Task\n\n");

        // Windsurf-specific instructions
        content.push_str("## Instructions for Windsurf\n\n");
        content.push_str("Use Cascade (Cmd+I) for autonomous coding:\n");
        content.push_str("- Cascade can read, write, and execute code\n");
        content.push_str("- Use Flow (Cmd+L) for inline edits\n");
        content.push_str("- Use @-mentions to add context (@file, @folder, @docs)\n\n");

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

impl Default for WindsurfProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Provider for WindsurfProvider {
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
             2. Press {} to open Cascade\n\
             3. Paste the prompt from: {}\n\
             4. Review and accept changes\n\
             5. Run: {}\n",
            "🏄 Windsurf Cascade Prompt Generated".green().bold(),
            "━".repeat(50).dimmed(),
            format!("Prompt saved to: {}", prompt_path.display()).cyan(),
            "Next steps:".yellow().bold(),
            "Windsurf".cyan(),
            "Cmd+I".cyan(),
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
        "windsurf"
    }

    fn is_available(&self) -> bool {
        true // Always available - just generates a file
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_windsurf_provider_name() {
        let provider = WindsurfProvider::new();
        assert_eq!(provider.name(), "windsurf");
    }

    #[test]
    fn test_windsurf_prompt_content() {
        let context = Context {
            working_dir: PathBuf::from("/test"),
            session_id: "test-session".to_string(),
            contract_hash: None,
        };

        let content = WindsurfProvider::generate_prompt_content("Build feature", &context);
        assert!(content.contains("Windsurf"));
        assert!(content.contains("Cascade"));
        assert!(content.contains("Build feature"));
    }
}
