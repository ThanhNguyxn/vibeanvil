//! Providers command - list available AI providers

use anyhow::Result;
use colored::Colorize;

use crate::provider::{get_provider, list_providers, CapabilityMatrix, ProviderSelector, TaskType};

/// Provider subcommand
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum ProviderSubcommand {
    /// List available providers
    #[default]
    List,
    /// Show capability matrix
    Matrix,
    /// Recommend provider for task
    Recommend(String),
    /// Compare providers
    Compare(Vec<String>),
}

pub async fn run() -> Result<()> {
    run_subcommand(ProviderSubcommand::List).await
}

pub async fn run_subcommand(cmd: ProviderSubcommand) -> Result<()> {
    match cmd {
        ProviderSubcommand::List => run_list().await,
        ProviderSubcommand::Matrix => run_matrix().await,
        ProviderSubcommand::Recommend(task) => run_recommend(&task).await,
        ProviderSubcommand::Compare(providers) => run_compare(&providers).await,
    }
}

async fn run_list() -> Result<()> {
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
    println!("{}", "💡 More Commands:".white().bold());
    println!();
    println!(
        "  {} {}",
        "Matrix:".cyan(),
        "vibeanvil providers matrix".white()
    );
    println!(
        "  {} {}",
        "Recommend:".cyan(),
        "vibeanvil providers recommend \"fix the login bug\"".white()
    );
    println!();

    Ok(())
}

/// Show provider capability matrix
async fn run_matrix() -> Result<()> {
    println!();
    println!(
        "{}",
        "╔═══════════════════════════════════════════════════════════════╗".cyan()
    );
    println!(
        "{}",
        "║               📊 Provider Capability Matrix                   ║".cyan()
    );
    println!(
        "{}",
        "╚═══════════════════════════════════════════════════════════════╝".cyan()
    );
    println!();

    let matrix = CapabilityMatrix::build_default();

    // Show by tier
    for tier in 1..=4 {
        let tier_name = match tier {
            1 => "🌟 Tier 1: Premium Agentic AI",
            2 => "⭐ Tier 2: Standard AI Tools",
            3 => "✨ Tier 3: Specialized Tools",
            4 => "💾 Tier 4: Local/Offline",
            _ => "Unknown",
        };
        println!("{}", tier_name.white().bold());
        println!();

        let providers = matrix.by_tier(tier);
        for p in providers {
            let mut caps: Vec<String> = Vec::new();

            // Show key capabilities
            use crate::provider::Capability;
            for cap in [
                Capability::CodeGeneration,
                Capability::Agentic,
                Capability::MultiFile,
                Capability::CodeReview,
            ] {
                let score = p.capability_score(cap);
                if score >= 7 {
                    caps.push(format!("{}:{}", cap.short_code(), score));
                }
            }

            let caps_str = if caps.is_empty() {
                "".to_string()
            } else {
                format!("[{}]", caps.join(", "))
            };

            let tags = p.tags.join(", ");
            println!(
                "  {} {} {}",
                p.name.cyan(),
                caps_str.dimmed(),
                format!("({})", tags).dimmed()
            );
        }
        println!();
    }

    println!("{}", "─".repeat(60).dimmed());
    println!();
    println!("{}", "Capability Codes:".white().bold());
    println!("  GEN=Code Generation, AGT=Agentic, MUL=Multi-File, REV=Code Review");
    println!("  FIX=Bug Fixing, TST=Test Gen, DOC=Documentation, ARC=Architecture");
    println!();

    Ok(())
}

/// Recommend provider for a task
async fn run_recommend(task: &str) -> Result<()> {
    println!();
    println!(
        "{}",
        "╔═══════════════════════════════════════════════════════════════╗".cyan()
    );
    println!(
        "{}",
        "║               🎯 Provider Recommendations                     ║".cyan()
    );
    println!(
        "{}",
        "╚═══════════════════════════════════════════════════════════════╝".cyan()
    );
    println!();

    let selector = ProviderSelector::new();
    let task_type = TaskType::infer(task);

    println!("{}: {}", "Task".white().bold(), task.cyan());
    println!("{}: {:?}", "Detected Type".white().bold(), task_type);
    println!();

    let recommendations = selector.recommend(task, 5);

    println!("{}", "Top Recommendations:".white().bold());
    println!();

    for (i, rec) in recommendations.iter().enumerate() {
        let rank = match i {
            0 => "🥇",
            1 => "🥈",
            2 => "🥉",
            _ => "  ",
        };

        println!("{} {} (Tier {})", rank, rec.name.cyan().bold(), rec.tier);
        println!("   {}", rec.reason.dimmed());

        let caps: Vec<String> = rec
            .capabilities
            .iter()
            .map(|(name, score)| format!("{}: {}/10", name, score))
            .collect();
        println!("   {}", caps.join(", ").dimmed());
        println!();
    }

    println!("{}", "─".repeat(50).dimmed());
    println!();
    println!(
        "{} {}",
        "Usage:".yellow(),
        format!(
            "vibeanvil build iterate --provider {}",
            recommendations[0].name
        )
        .white()
    );
    println!();

    Ok(())
}

/// Compare specific providers
async fn run_compare(providers: &[String]) -> Result<()> {
    println!();
    println!(
        "{}",
        "╔═══════════════════════════════════════════════════════════════╗".cyan()
    );
    println!(
        "{}",
        "║               ⚖️  Provider Comparison                         ║".cyan()
    );
    println!(
        "{}",
        "╚═══════════════════════════════════════════════════════════════╝".cyan()
    );
    println!();

    let matrix = CapabilityMatrix::build_default();

    use crate::provider::Capability;
    let key_caps = [
        Capability::CodeGeneration,
        Capability::CodeReview,
        Capability::Agentic,
        Capability::MultiFile,
        Capability::BugFixing,
        Capability::TestGeneration,
        Capability::Streaming,
    ];

    // Header
    print!("{:20}", "Capability".white().bold());
    for name in providers {
        print!("{:15}", name.cyan());
    }
    println!();
    println!("{}", "─".repeat(20 + providers.len() * 15).dimmed());

    // Rows
    for cap in key_caps {
        print!("{:20}", cap.display_name());
        for name in providers {
            if let Some(profile) = matrix.get(name) {
                let score = profile.capability_score(cap);
                let display = format!("{}/10", score);
                let colored = if score >= 8 {
                    display.green()
                } else if score >= 5 {
                    display.yellow()
                } else if score > 0 {
                    display.red()
                } else {
                    "-".dimmed()
                };
                print!("{:15}", colored);
            } else {
                print!("{:15}", "N/A".dimmed());
            }
        }
        println!();
    }

    println!();
    println!("{}", "─".repeat(20 + providers.len() * 15).dimmed());

    // Summary row
    print!("{:20}", "Tier".white().bold());
    for name in providers {
        if let Some(profile) = matrix.get(name) {
            print!("{:15}", format!("Tier {}", profile.tier).cyan());
        } else {
            print!("{:15}", "N/A".dimmed());
        }
    }
    println!();

    print!("{:20}", "Cost/1K".white().bold());
    for name in providers {
        if let Some(profile) = matrix.get(name) {
            let cost = if profile.cost_per_1k == 0.0 {
                "Free".to_string()
            } else {
                format!("${:.3}", profile.cost_per_1k)
            };
            print!("{:15}", cost.green());
        } else {
            print!("{:15}", "N/A".dimmed());
        }
    }
    println!();
    println!();

    Ok(())
}
