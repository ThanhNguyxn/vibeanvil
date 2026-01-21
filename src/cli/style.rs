//! Centralized styling for VibeAnvil CLI
//! Provides consistent spinners, messages, and prompts.

use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

/// Create a new spinner with a message
pub fn spinner(msg: &str) -> ProgressBar {
    // In CI or non-interactive mode, return a hidden spinner to avoid log clutter/hangs
    if std::env::var("CI").is_ok() || !console::user_attended() {
        return ProgressBar::hidden();
    }

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏ ")
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    pb.set_message(msg.to_string());
    pb.enable_steady_tick(Duration::from_millis(80));
    pb
}

/// Print a success message
pub fn success(msg: &str) {
    println!("{} {}", "✔".green(), msg);
}

/// Print an error message
pub fn error(msg: &str) {
    println!("{} {}", "✖".red(), msg);
}

/// Print a warning message
pub fn warn(msg: &str) {
    println!("{} {}", "⚠️".yellow(), msg);
}

/// Print an info message
pub fn info(msg: &str) {
    println!("{} {}", "ℹ".blue(), msg);
}

/// Print a step header
pub fn step(msg: &str) {
    println!("\n{} {}", "➤".bold().cyan(), msg.bold());
}

/// Print a section header with a decorative line
pub fn header(title: &str) {
    let line = "═".repeat(50);
    println!("\n{}", line.cyan());
    println!("{}", title.cyan().bold());
    println!("{}\n", line.cyan());
}
