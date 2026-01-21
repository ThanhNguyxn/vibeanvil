//! Trae provider - generates prompts for ByteDance's Trae AI IDE
//!
//! Trae is ByteDance's AI-powered IDE with SOLO mode (autonomous coding),
//! multi-agent collaboration, and MCP support.
//!
//! Usage:
//! 1. Open project in Trae
//! 2. Use SOLO mode for autonomous coding
//! 3. Paste the prompt
//! 4. Let Trae make changes

use anyhow::Result;
use async_trait::async_trait;
use colored::Colorize;
use std::path::PathBuf;
use tokio::fs;

use super::{Context, Provider, ProviderResponse};

/// Trae provider for ByteDance's AI IDE
pub struct TraeProvider;

impl TraeProvider {
    pub fn new() -> Self {
        Self
    }

    fn prompt_file_path(context: &Context) -> PathBuf {
        context
            .working_dir
            .join(".vibeanvil")
            .join("sessions")
            .join(&context.session_id)
            .join("trae_prompt.md")
    }

    fn generate_prompt_content(prompt: &str, context: &Context) -> String {
        let mut content = String::new();

        content.push_str("# Trae AI Task\n\n");

        // Trae-specific instructions
        content.push_str("## Instructions for Trae\n\n");
        content.push_str("Use SOLO mode for autonomous coding:\n");
        content.push_str("- SOLO can plan, code, debug, and test\n");
        content.push_str("- Use Builder mode for step-by-step guidance\n");
        content.push_str("- Use Chat mode for quick questions\n\n");

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

impl Default for TraeProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Provider for TraeProvider {
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
             2. Switch to {} for autonomous coding\n\
             3. Paste the prompt from: {}\n\
             4. Review and accept changes\n\
             5. Run: {}\n",
            "🎯 Trae AI Prompt Generated".green().bold(),
            "━".repeat(50).dimmed(),
            format!("Prompt saved to: {}", prompt_path.display()).cyan(),
            "Next steps:".yellow().bold(),
            "Trae".cyan(),
            "SOLO mode".cyan(),
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
        "trae"
    }

    fn is_available(&self) -> bool {
        true // Always available - just generates a file
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trae_provider_name() {
        let provider = TraeProvider::new();
        assert_eq!(provider.name(), "trae");
    }

    #[test]
    fn test_trae_prompt_content() {
        let context = Context {
            working_dir: PathBuf::from("/test"),
            session_id: "test-session".to_string(),
            contract_hash: None,
        };

        let content = TraeProvider::generate_prompt_content("Create API", &context);
        assert!(content.contains("Trae"));
        assert!(content.contains("SOLO"));
        assert!(content.contains("Create API"));
    }
}
