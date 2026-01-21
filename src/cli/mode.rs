//! Chat modes for different interaction styles (inspired by Aider)
//!
//! Modes control how the AI interacts with your codebase:
//! - Ask: Discuss code without making changes
//! - Code: Make changes to satisfy requests
//! - Architect: Propose changes at high level, then apply with editor
//! - Help: Answer questions about VibeAnvil itself

use anyhow::Result;
use colored::*;
use serde::{Deserialize, Serialize};

use crate::provider::{get_provider, Context, ProviderResponse};

/// Available chat modes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ChatMode {
    /// Ask mode - discuss code without making changes
    Ask,
    /// Code mode - make changes to satisfy requests (default)
    #[default]
    Code,
    /// Architect mode - high-level proposals then detailed edits
    Architect,
    /// Help mode - answer questions about VibeAnvil
    Help,
}

impl std::fmt::Display for ChatMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChatMode::Ask => write!(f, "ask"),
            ChatMode::Code => write!(f, "code"),
            ChatMode::Architect => write!(f, "architect"),
            ChatMode::Help => write!(f, "help"),
        }
    }
}

impl std::str::FromStr for ChatMode {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "ask" => Ok(ChatMode::Ask),
            "code" => Ok(ChatMode::Code),
            "architect" => Ok(ChatMode::Architect),
            "help" => Ok(ChatMode::Help),
            _ => anyhow::bail!("Unknown chat mode: {}. Use: ask, code, architect, help", s),
        }
    }
}

/// Chat mode configuration
#[derive(Debug, Clone)]
pub struct ModeConfig {
    /// Current active mode
    pub mode: ChatMode,
    /// Provider to use for AI interactions
    pub provider: String,
    /// Editor model for architect mode
    pub editor_model: Option<String>,
    /// Working directory
    pub working_dir: std::path::PathBuf,
    /// Session ID
    pub session_id: String,
}

impl Default for ModeConfig {
    fn default() -> Self {
        Self {
            mode: ChatMode::Code,
            provider: "claude-code".to_string(),
            editor_model: None,
            working_dir: std::env::current_dir().unwrap_or_default(),
            session_id: uuid::Uuid::new_v4().to_string(),
        }
    }
}

/// Mode-aware chat handler
pub struct ModeHandler {
    config: ModeConfig,
}

impl ModeHandler {
    pub fn new(config: ModeConfig) -> Self {
        Self { config }
    }

    /// Get the prompt prefix/template for the current mode
    fn get_mode_prompt(&self, user_message: &str) -> String {
        match self.config.mode {
            ChatMode::Ask => {
                format!(
                    r#"You are in ASK mode. The user wants to discuss and understand code WITHOUT making any changes.

Rules for ASK mode:
- Explain, analyze, and answer questions about the code
- DO NOT suggest file edits or modifications
- DO NOT output diff blocks or code changes
- Focus on understanding, architecture, patterns, and best practices

User's question: {}"#,
                    user_message
                )
            }
            ChatMode::Code => {
                format!(
                    r#"You are in CODE mode. The user wants you to make changes to the codebase.

Rules for CODE mode:
- Analyze the request and implement the changes
- Output specific file edits with clear diff blocks
- Ensure code is correct, idiomatic, and follows project conventions
- Run tests and lint if available

User's request: {}"#,
                    user_message
                )
            }
            ChatMode::Architect => {
                format!(
                    r#"You are in ARCHITECT mode. This is a two-phase interaction:

Phase 1 (You are here): Propose a high-level solution
- Analyze the problem thoroughly
- Propose an architectural approach
- List the files that need to change
- Describe the changes at a conceptual level
- DO NOT output detailed code edits yet

After this, an editor model will translate your proposal into specific file edits.

User's request: {}"#,
                    user_message
                )
            }
            ChatMode::Help => {
                format!(
                    r#"You are in HELP mode. Answer questions about VibeAnvil.

VibeAnvil is a contract-first vibe coding tool with these commands:
- init: Initialize workspace
- intake: Capture requirements
- blueprint: Generate architecture blueprint
- contract: Create/validate/lock the contract
- plan: Create implementation plan
- build: Execute build (manual/auto/iterate modes)
- review: Review changes
- ship: Mark as shipped
- brain: Manage knowledge base
- harvest: Harvest repos for knowledge
- status: Show workflow status
- undo: Revert last AI change

User's question: {}"#,
                    user_message
                )
            }
        }
    }

    /// Execute a chat in the current mode
    pub async fn chat(&self, message: &str) -> Result<ProviderResponse> {
        let provider = get_provider(&self.config.provider)?;
        let context = Context {
            working_dir: self.config.working_dir.clone(),
            session_id: self.config.session_id.clone(),
            contract_hash: None,
        };

        let prompt = self.get_mode_prompt(message);

        match self.config.mode {
            ChatMode::Architect => {
                // Phase 1: Get architectural proposal
                println!("{}", "⚙ Architect analyzing...".cyan());
                let proposal = provider.execute(&prompt, &context).await?;

                // Phase 2: If we have an editor model, use it to apply changes
                if let Some(editor_model) = &self.config.editor_model {
                    println!("{}", "⚙ Editor applying changes...".cyan());
                    let editor = get_provider(editor_model)?;
                    let edit_prompt = format!(
                        r#"You are an editor model. Translate this architectural proposal into specific file edits.

PROPOSAL:
{}

Output precise file edits with clear diff blocks showing exactly what to change."#,
                        proposal.output
                    );
                    editor.execute(&edit_prompt, &context).await
                } else {
                    Ok(proposal)
                }
            }
            _ => provider.execute(&prompt, &context).await,
        }
    }

    /// Get the prompt indicator for the current mode
    pub fn prompt_indicator(&self) -> String {
        match self.config.mode {
            ChatMode::Ask => "ask> ".yellow().to_string(),
            ChatMode::Code => "> ".green().to_string(),
            ChatMode::Architect => "architect> ".magenta().to_string(),
            ChatMode::Help => "help> ".blue().to_string(),
        }
    }
}

/// Run the chat mode command
pub async fn run_mode(mode: ChatMode, message: &str, provider: &str) -> Result<()> {
    use crate::cli::style;

    let config = ModeConfig {
        mode,
        provider: provider.to_string(),
        ..Default::default()
    };

    let handler = ModeHandler::new(config);

    style::step(&format!("Mode: {}", mode));
    println!("{}", handler.prompt_indicator());

    let response = handler.chat(message).await?;

    if response.success {
        println!("\n{}", response.output);
        style::success(&format!("{} mode complete", mode));
    } else {
        for error in &response.errors {
            style::error(error);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_mode_from_str() {
        assert_eq!("ask".parse::<ChatMode>().unwrap(), ChatMode::Ask);
        assert_eq!("code".parse::<ChatMode>().unwrap(), ChatMode::Code);
        assert_eq!(
            "architect".parse::<ChatMode>().unwrap(),
            ChatMode::Architect
        );
        assert_eq!("help".parse::<ChatMode>().unwrap(), ChatMode::Help);
        assert!("invalid".parse::<ChatMode>().is_err());
    }

    #[test]
    fn test_chat_mode_display() {
        assert_eq!(ChatMode::Ask.to_string(), "ask");
        assert_eq!(ChatMode::Code.to_string(), "code");
        assert_eq!(ChatMode::Architect.to_string(), "architect");
        assert_eq!(ChatMode::Help.to_string(), "help");
    }

    #[test]
    fn test_mode_handler_prompt() {
        let config = ModeConfig::default();
        let handler = ModeHandler::new(config);
        let prompt = handler.get_mode_prompt("test");
        assert!(prompt.contains("CODE mode"));
    }
}
