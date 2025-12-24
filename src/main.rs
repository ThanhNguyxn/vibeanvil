//! # VibeAnvil CLI
//!
//! Contract-first vibe coding with evidence, audit, and repo-brain harvesting.
//!
//! A next-gen "guardrails" CLI implementing:
//! - Enforced workflow state machine
//! - Evidence & audit by default
//! - Build modes: manual, auto, iterate
//! - Provider plugins (Claude Code CLI adapter)
//! - Repo-brain harvesting with searchable BrainPack

// Allow dead code for public API functions that may be used in the future
#![allow(dead_code)]

use anyhow::Result;
use clap::Parser;

mod audit;
mod brain;
mod build;
mod cli;
mod contract;
mod evidence;
mod provider;
mod state;
mod workspace;

use cli::{Cli, Commands};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with_target(false)
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Init { force } => cli::init::run(force).await,
        Commands::Intake { message } => cli::intake::run(message).await,
        Commands::Blueprint { auto } => cli::blueprint::run(auto).await,
        Commands::Contract { action } => cli::contract::run(action).await,
        Commands::Plan { provider } => cli::plan::run(provider).await,
        Commands::Build(args) => cli::build::run(args).await,
        Commands::Review { action } => cli::review::run(action).await,
        Commands::Snapshot { message } => cli::snapshot::run(message).await,
        Commands::Ship { tag, message } => cli::ship::run(tag, message).await,
        Commands::Harvest(args) => cli::harvest::run(args).await,
        Commands::Brain(args) => cli::brain::run(args).await,
        Commands::Status { verbose } => cli::status::run(verbose).await,
        Commands::Log { lines, json } => cli::log::run(lines, json).await,
        Commands::Update => cli::update::check_update().await,
        Commands::Upgrade => cli::update::upgrade().await,
    }
}
