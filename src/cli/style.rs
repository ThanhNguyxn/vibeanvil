//! Centralized styling for VibeAnvil CLI
//! Provides consistent spinners, messages, and prompts.

use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

/// Create a new spinner with a message
pub fn spinner(msg: &str) -> ProgressBar {
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
