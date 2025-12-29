//! Undo command - revert the last AI-made change

use anyhow::{Context, Result};
use colored::*;
use std::process::Command;

/// Run the undo command
pub async fn run(dry_run: bool) -> Result<()> {
    // Check if we're in a git repository
    let status = Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .output()
        .context("Failed to check git repository")?;

    if !status.status.success() {
        anyhow::bail!("Not inside a git repository");
    }

    // Get the last commit info
    let output = Command::new("git")
        .args(["log", "-1", "--pretty=format:%h %s", "--no-walk"])
        .output()
        .context("Failed to get last commit")?;

    let last_commit = String::from_utf8_lossy(&output.stdout);

    if last_commit.is_empty() {
        anyhow::bail!("No commits found to undo");
    }

    // Check if it's likely an AI-made commit (contains common AI patterns)
    let is_ai_commit = last_commit.contains("feat:")
        || last_commit.contains("fix:")
        || last_commit.contains("refactor:")
        || last_commit.contains("docs:")
        || last_commit.contains("chore:")
        || last_commit.contains("AI")
        || last_commit.contains("auto");

    if dry_run {
        println!("{}", "🔍 Dry run - showing what would be undone:".cyan());
        println!();
        println!("   {} {}", "Last commit:".yellow(), last_commit);
        println!();

        if is_ai_commit {
            println!("   {} This looks like an AI-made commit.", "✓".green());
        } else {
            println!("   {} This may not be an AI-made commit.", "⚠".yellow());
        }

        println!();
        println!("   Run {} to actually undo.", "vibeanvil undo".cyan());
        return Ok(());
    }

    // Confirm before undoing
    println!("{}", "🔄 Undoing last commit...".cyan());
    println!();
    println!("   {} {}", "Reverting:".yellow(), last_commit);
    println!();

    // Git reset --soft HEAD~1 (keeps changes in staging)
    let reset = Command::new("git")
        .args(["reset", "--soft", "HEAD~1"])
        .output()
        .context("Failed to reset last commit")?;

    if !reset.status.success() {
        let err = String::from_utf8_lossy(&reset.stderr);
        anyhow::bail!("Failed to undo: {}", err);
    }

    println!("   {} Last commit undone!", "✓".green());
    println!(
        "   {} Changes are still staged - you can review and recommit.",
        "ℹ".blue()
    );
    println!();
    println!("   To see staged changes: {}", "git diff --cached".cyan());
    println!("   To unstage all: {}", "git reset HEAD".cyan());

    Ok(())
}
