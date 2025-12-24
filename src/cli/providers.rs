//! Providers command - list available AI providers

use anyhow::Result;
use colored::Colorize;

use crate::provider::{get_provider, list_providers};

pub async fn run() -> Result<()> {
    println!();
    println!(
        "{}",
        "╔═══════════════════════════════════════════════════════════════╗".cyan()
    );
    println!(
        "{}",
        "║               🔌 Available Providers                          ║".cyan()
    );
    println!(
        "{}",
        "╚═══════════════════════════════════════════════════════════════╝".cyan()
    );
    println!();

    let providers = list_providers();

    for name in providers {
        let provider = get_provider(name)?;
        let available = provider.is_available();
        let status = if available {
            "✅ Available".green()
        } else {
            "❌ Not available".red()
        };

        println!("  {} {}", status, name.white().bold());

        // Show configuration info
        match name {
            "claude-code" => {
                println!("    {}", "Claude Code CLI for AI-assisted coding".dimmed());
                if !available {
                    println!(
                        "    {} {}",
                        "Install:".yellow(),
                        "npm install -g @anthropic-ai/claude-code".white()
                    );
                }
            }
            "human" => {
                println!(
                    "    {}",
                    "Generate prompts for IDE assistants (Copilot/Cursor/VS Code Chat)".dimmed()
                );
                println!(
                    "    {}",
                    "Always available - you apply changes manually".dimmed()
                );
            }
            "command" => {
                println!(
                    "    {}",
                    "Execute external CLI agents (Aider, etc.)".dimmed()
                );
                if !available {
                    println!(
                        "    {} Set these environment variables:",
                        "Config:".yellow()
                    );
                    println!(
                        "      {} {}",
                        "VIBEANVIL_PROVIDER_COMMAND".cyan(),
                        "= <command name>".dimmed()
                    );
                    println!(
                        "      {} {}",
                        "VIBEANVIL_PROVIDER_ARGS".cyan(),
                        "= <extra args> (optional)".dimmed()
                    );
                    println!(
                        "      {} {}",
                        "VIBEANVIL_PROVIDER_MODE".cyan(),
                        "= stdin|arg|file (optional)".dimmed()
                    );
                }
            }
            "patch" => {
                println!(
                    "    {}",
                    "Apply unified diffs from AI or manual editing".dimmed()
                );
                if available {
                    println!(
                        "    {} {}",
                        "Usage:".yellow(),
                        "export VIBEANVIL_PATCH_FILE=changes.patch".white()
                    );
                } else {
                    println!("    {} {}", "Requires:".yellow(), "git (not found)".white());
                }
            }
            "mock" => {
                println!("    {}", "For testing only - no actual changes".dimmed());
            }
            _ => {}
        }
        println!();
    }

    println!("{}", "─".repeat(50).dimmed());
    println!();
    println!("{}", "💡 Usage Examples:".white().bold());
    println!();
    println!(
        "  {} {}",
        "Copilot/Cursor:".cyan(),
        "vibeanvil build iterate --provider human".white()
    );
    println!(
        "  {} {}",
        "Claude Code:".cyan(),
        "vibeanvil build iterate --provider claude-code".white()
    );
    println!(
        "  {} {}",
        "CLI Agent:".cyan(),
        "vibeanvil build iterate --provider command".white()
    );
    println!(
        "  {} {}",
        "Unified Diff:".cyan(),
        "vibeanvil build iterate --provider patch".white()
    );
    println!();

    Ok(())
}
