//! Init command handler with beautiful output

use anyhow::Result;
use colored::Colorize;

use crate::audit::{generate_session_id, AuditLogger};
use crate::cli::ui;
use crate::workspace;

pub async fn run(force: bool) -> Result<()> {
    // Print beautiful banner
    ui::print_banner();

    if workspace::workspace_exists().await && !force {
        println!();
        println!(
            "{}",
            "┌─────────────────────────────────────────────────┐".yellow()
        );
        println!(
            "{}",
            "│  ⚠️  Workspace already exists!                   │".yellow()
        );
        println!(
            "{}",
            "│  Use --force to reinitialize                    │".white()
        );
        println!(
            "{}",
            "└─────────────────────────────────────────────────┘".yellow()
        );
        println!();
        return Ok(());
    }

    workspace::init_workspace(force).await?;

    let session_id = generate_session_id();
    let logger = AuditLogger::new(&session_id);
    logger
        .log_command("init", vec![format!("force={}", force)])
        .await?;

    println!();
    println!(
        "{}",
        "┌─────────────────────────────────────────────────┐".green()
    );
    println!(
        "{}",
        "│  ✅ Workspace initialized successfully!         │".green()
    );
    println!(
        "{}",
        "└─────────────────────────────────────────────────┘".green()
    );
    println!();

    // Core BrainPack info
    println!(
        "{}",
        "🧠 Core BrainPack: Run 'vibeanvil brain ensure' to install".cyan()
    );
    println!(
        "{}",
        "   Curated templates for contracts, plans, and best practices".dimmed()
    );
    println!();

    println!("{}", "📋 Next Steps:".white().bold());
    println!();
    println!("  {} {}", "1.".cyan(), "vibeanvil brain ensure".white());
    println!(
        "  {} {}",
        "2.".cyan(),
        "vibeanvil intake -m \"Your project requirements\"".white()
    );
    println!("  {} {}", "3.".cyan(), "vibeanvil blueprint --auto".white());
    println!("  {} {}", "4.".cyan(), "vibeanvil contract create".white());
    println!("  {} {}", "5.".cyan(), "vibeanvil contract lock".white());
    println!();

    println!("{}", "💡 Tips:".yellow().bold());
    println!(
        "  {} {}",
        "•".dimmed(),
        "Use 'vibeanvil brain search \"acceptance criteria\"' to find guidance".dimmed()
    );
    println!(
        "  {} {}",
        "•".dimmed(),
        "Use 'vibeanvil status' to check current state".dimmed()
    );
    println!(
        "  {} {}",
        "•".dimmed(),
        "Use 'vibeanvil update' to check for updates".dimmed()
    );
    println!();

    Ok(())
}
