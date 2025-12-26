//! Wizard command - Interactive menu when no args provided

use anyhow::Result;
use colored::Colorize;
use inquire::Select;

/// Wizard action choices
#[derive(Debug, Clone)]
enum WizardAction {
    Init,
    BrainEnsure,
    BrainSearch,
    Intake,
    Status,
    Doctor,
    Harvest,
    Help,
    Exit,
}

impl std::fmt::Display for WizardAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WizardAction::Init => write!(f, "📁 Initialize new workspace"),
            WizardAction::BrainEnsure => write!(f, "🧠 Install Core BrainPack"),
            WizardAction::BrainSearch => write!(f, "🔍 Search BrainPack"),
            WizardAction::Intake => write!(f, "📝 Capture requirements (intake)"),
            WizardAction::Status => write!(f, "📊 View current status"),
            WizardAction::Doctor => write!(f, "🩺 Check system health"),
            WizardAction::Harvest => write!(f, "🌾 Harvest from GitHub"),
            WizardAction::Help => write!(f, "❓ Show help"),
            WizardAction::Exit => write!(f, "👋 Exit"),
        }
    }
}

pub async fn run() -> Result<()> {
    // Print welcome banner
    println!();
    println!(
        "{}",
        "╔═══════════════════════════════════════════════════════════════╗".cyan()
    );
    println!(
        "{}",
        format!(
            "║   🔨 VibeAnvil v{}                                         ║",
            env!("CARGO_PKG_VERSION")
        )
        .cyan()
    );
    println!(
        "{}",
        "║   Contract-first vibe coding with evidence & audit           ║".white()
    );
    println!(
        "{}",
        "╚═══════════════════════════════════════════════════════════════╝".cyan()
    );
    println!();

    let options = vec![
        WizardAction::Init,
        WizardAction::BrainEnsure,
        WizardAction::BrainSearch,
        WizardAction::Intake,
        WizardAction::Status,
        WizardAction::Doctor,
        WizardAction::Harvest,
        WizardAction::Help,
        WizardAction::Exit,
    ];

    let answer = Select::new("What would you like to do?", options)
        .with_help_message("Use ↑↓ to navigate, Enter to select")
        .prompt();

    match answer {
        Ok(action) => handle_action(action).await,
        Err(_) => {
            println!();
            println!("{}", "👋 Goodbye!".cyan());
            println!();
            Ok(())
        }
    }
}

async fn handle_action(action: WizardAction) -> Result<()> {
    println!();

    match action {
        WizardAction::Init => {
            crate::cli::init::run(false).await?;
        }
        WizardAction::BrainEnsure => {
            crate::cli::brain::run(crate::cli::BrainArgs {
                command: crate::cli::BrainCommands::Ensure {
                    refresh_core: false,
                    verbose: false,
                },
            })
            .await?;
        }
        WizardAction::BrainSearch => {
            let query = inquire::Text::new("Search query:")
                .with_help_message("Enter keywords to search in BrainPack")
                .prompt();

            match query {
                Ok(q) if !q.is_empty() => {
                    crate::cli::brain::run(crate::cli::BrainArgs {
                        command: crate::cli::BrainCommands::Search {
                            query: q,
                            limit: 10,
                        },
                    })
                    .await?;
                }
                _ => {
                    println!("{}", "Search cancelled.".dimmed());
                }
            }
        }
        WizardAction::Intake => {
            let message = inquire::Text::new("Describe your project requirements:")
                .with_help_message("Be specific about what you want to build")
                .prompt();

            match message {
                Ok(m) if !m.is_empty() => {
                    crate::cli::intake::run(Some(m)).await?;
                }
                _ => {
                    println!("{}", "Intake cancelled.".dimmed());
                }
            }
        }
        WizardAction::Status => {
            crate::cli::status::run(false).await?;
        }
        WizardAction::Doctor => {
            crate::cli::doctor::run().await?;
        }
        WizardAction::Harvest => {
            println!("{}", "💡 Harvest repos from GitHub:".white().bold());
            println!();
            println!(
                "  {} {}",
                "•".cyan(),
                "vibeanvil harvest --query 'state machine workflow cli'".white()
            );
            println!(
                "  {} {}",
                "•".cyan(),
                "vibeanvil harvest --query 'rust error handling' --language Rust".white()
            );
            println!();
            println!(
                "{}",
                "See brainpacks/presets.yaml for curated search queries.".dimmed()
            );
        }
        WizardAction::Help => {
            println!("{}", "📚 VibeAnvil Commands:".white().bold());
            println!();
            println!("  {} {}", "init".cyan(), "Initialize workspace".dimmed());
            println!("  {} {}", "intake".cyan(), "Capture requirements".dimmed());
            println!("  {} {}", "blueprint".cyan(), "Generate blueprint".dimmed());
            println!("  {} {}", "contract".cyan(), "Manage contract".dimmed());
            println!("  {} {}", "build".cyan(), "Execute build".dimmed());
            println!("  {} {}", "review".cyan(), "Review changes".dimmed());
            println!("  {} {}", "ship".cyan(), "Ship release".dimmed());
            println!();
            println!("  {} {}", "brain".cyan(), "Manage BrainPack".dimmed());
            println!("  {} {}", "harvest".cyan(), "Harvest from GitHub".dimmed());
            println!("  {} {}", "doctor".cyan(), "Check system health".dimmed());
            println!();
            println!(
                "{}",
                "Run 'vibeanvil <command> --help' for more info.".dimmed()
            );
        }
        WizardAction::Exit => {
            println!("{}", "👋 Goodbye!".cyan());
        }
    }

    println!();
    Ok(())
}
