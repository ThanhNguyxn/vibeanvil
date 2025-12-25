//! Human provider - generates prompt files for IDE assistants (Copilot/Cursor/VS Code Chat)

use anyhow::Result;
use async_trait::async_trait;
use colored::Colorize;
use std::path::PathBuf;
use tokio::fs;

use super::{Context, Provider, ProviderResponse};

/// Human provider for IDE assistant workflows
///
/// This provider generates a markdown prompt file that users can copy/paste
/// into their IDE assistant (GitHub Copilot, Cursor, VS Code Chat, etc.)
pub struct HumanProvider;

impl HumanProvider {
    pub fn new() -> Self {
        Self
    }

    /// Generate the prompt file path for a session
    fn prompt_file_path(context: &Context) -> PathBuf {
        context
            .working_dir
            .join(".vibeanvil")
            .join("sessions")
            .join(&context.session_id)
            .join("human_prompt.md")
    }

    /// Generate the prompt content with context and instructions
    fn generate_prompt_content(prompt: &str, context: &Context) -> String {
        let mut content = String::new();

        content.push_str("# VibeAnvil AI Prompt\n\n");

        // Safety instructions first
        content.push_str("## ⚠️ Safety Instructions\n\n");
        content.push_str("**IMPORTANT - Please follow these rules:**\n\n");
        content.push_str(
            "- ❌ **Do NOT paste secrets** (API keys, tokens, passwords) into responses\n",
        );
        content.push_str("- ❌ **Do NOT modify files outside the repository**\n");
        content.push_str("- ❌ **Do NOT modify .git/ directory or config files**\n");
        content.push_str("- ✅ **DO run tests after changes** (`cargo test`, `npm test`, etc.)\n");
        content.push_str("- ✅ **DO follow existing code style**\n\n");

        content.push_str("---\n\n");

        content.push_str("## Instructions\n\n");
        content.push_str("Copy this entire file content and paste it into your IDE assistant ");
        content.push_str("(GitHub Copilot Chat, Cursor, VS Code Chat, or similar).\n\n");
        content.push_str("After the AI makes changes, run:\n");
        content.push_str("```bash\n");
        content.push_str("vibeanvil build manual evidence\n");
        content.push_str("vibeanvil build manual complete\n");
        content.push_str("```\n\n");

        content.push_str("---\n\n");

        content.push_str("## Context\n\n");
        content.push_str(&format!("- **Session ID**: `{}`\n", context.session_id));
        if let Some(hash) = &context.contract_hash {
            content.push_str(&format!("- **Contract Hash**: `{}`\n", hash));
        }
        content.push_str(&format!(
            "- **Working Directory**: `{}`\n",
            context.working_dir.display()
        ));
        content.push('\n');

        content.push_str("---\n\n");

        content.push_str("## Task\n\n");
        content.push_str(prompt);
        content.push_str("\n\n");

        content.push_str("---\n\n");

        content.push_str("## Constraints\n\n");
        content.push_str("- Follow the project's existing code style\n");
        content.push_str("- Ensure all tests pass after changes\n");
        content.push_str("- Do not modify files outside the project directory\n");
        if context.contract_hash.is_some() {
            content.push_str("- Adhere to the locked contract requirements\n");
        }

        content
    }
}

impl Default for HumanProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Provider for HumanProvider {
    async fn execute(&self, prompt: &str, context: &Context) -> Result<ProviderResponse> {
        let prompt_path = Self::prompt_file_path(context);

        // Ensure session directory exists
        if let Some(parent) = prompt_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        // Generate and write prompt file
        let content = Self::generate_prompt_content(prompt, context);
        fs::write(&prompt_path, &content).await?;

        // Print instructions to console
        println!();
        println!("{}", "═".repeat(60).cyan());
        println!(
            "{}",
            " 🧑‍💻 Human Provider - IDE Assistant Workflow".cyan().bold()
        );
        println!("{}", "═".repeat(60).cyan());
        println!();
        println!(
            "{}",
            "A prompt file has been generated for your IDE assistant.".white()
        );
        println!();
        println!("{}", "📋 Steps:".yellow().bold());
        println!("  {} Open the prompt file:", "1.".cyan());
        println!("     {}", prompt_path.display().to_string().green());
        println!();
        println!(
            "  {} Copy the content and paste into your IDE assistant:",
            "2.".cyan()
        );
        println!("     • GitHub Copilot Chat");
        println!("     • Cursor");
        println!("     • VS Code Chat");
        println!("     • Or any other AI assistant");
        println!();
        println!("  {} Apply the suggested changes in your IDE", "3.".cyan());
        println!();
        println!("  {} Run evidence capture and complete:", "4.".cyan());
        println!("     {}", "vibeanvil build manual evidence".green());
        println!("     {}", "vibeanvil build manual complete".green());
        println!();
        println!("{}", "═".repeat(60).cyan());

        Ok(ProviderResponse {
            success: true,
            output: format!(
                "Prompt file generated at: {}\nThis is a human-driven workflow. Apply changes via your IDE assistant.",
                prompt_path.display()
            ),
            errors: vec![],
            warnings: vec![
                "Human provider: No automatic code changes. User must apply changes manually.".to_string()
            ],
            files_modified: vec![prompt_path.display().to_string()],
        })
    }

    fn name(&self) -> &str {
        "human"
    }

    fn is_available(&self) -> bool {
        // Human provider is always available
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_provider_name() {
        let provider = HumanProvider::new();
        assert_eq!(provider.name(), "human");
    }

    #[test]
    fn test_always_available() {
        let provider = HumanProvider::new();
        assert!(provider.is_available());
    }

    #[tokio::test]
    async fn test_generates_prompt_file() {
        let temp_dir = TempDir::new().unwrap();
        let session_dir = temp_dir.path().join(".vibeanvil/sessions/test-session");
        std::fs::create_dir_all(&session_dir).unwrap();

        let context = Context {
            working_dir: temp_dir.path().to_path_buf(),
            session_id: "test-session".to_string(),
            contract_hash: Some("abc123".to_string()),
        };

        let provider = HumanProvider::new();
        let response = provider.execute("Test prompt", &context).await.unwrap();

        assert!(response.success);
        assert!(!response.files_modified.is_empty());

        let prompt_path = HumanProvider::prompt_file_path(&context);
        assert!(prompt_path.exists());

        let content = std::fs::read_to_string(&prompt_path).unwrap();
        assert!(content.contains("Test prompt"));
        assert!(content.contains("test-session"));
        assert!(content.contains("abc123"));
    }
}
