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
mod guardrails;
mod mcp;
mod prompt;
mod provider;
mod security;
mod state;
mod workspace;

use cli::{ChatModeArg, Cli, Commands};

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
        Commands::Status { verbose, json } => cli::status::run(verbose, json).await,
        Commands::Log { lines, json } => cli::log::run(lines, json).await,
        Commands::Update => cli::update::check_update().await,
        Commands::Upgrade => cli::update::upgrade().await,
        Commands::Doctor => cli::doctor::run().await,
        Commands::Wizard => cli::wizard::run().await,
        Commands::Providers { subcommand, args } => {
            let cmd = match subcommand.as_deref() {
                Some("matrix") => cli::providers::ProviderSubcommand::Matrix,
                Some("recommend") => {
                    let task = args.join(" ");
                    if task.is_empty() {
                        eprintln!("Usage: vibeanvil providers recommend \"<task description>\"");
                        std::process::exit(1);
                    }
                    cli::providers::ProviderSubcommand::Recommend(task)
                }
                Some("compare") => {
                    if args.is_empty() {
                        eprintln!("Usage: vibeanvil providers compare <provider1> <provider2> ...");
                        std::process::exit(1);
                    }
                    cli::providers::ProviderSubcommand::Compare(args)
                }
                Some("list") | None => cli::providers::ProviderSubcommand::List,
                Some(other) => {
                    eprintln!(
                        "Unknown subcommand: {}. Use list, matrix, recommend, or compare.",
                        other
                    );
                    std::process::exit(1);
                }
            };
            cli::providers::run_subcommand(cmd).await
        }
        Commands::Undo { dry_run } => cli::undo::run(dry_run).await,

        // New workflow commands
        Commands::Constitution {
            guidelines,
            view,
            provider,
        } => cli::constitution::run_constitution(&provider, guidelines.as_deref(), view).await,
        Commands::Clarify { provider } => cli::clarify::run_clarify(&provider).await,
        Commands::Tasks {
            provider,
            regenerate,
            done,
        } => {
            if let Some(task_id) = done {
                cli::tasks::complete_task(&task_id).await
            } else {
                cli::tasks::run_tasks(&provider, regenerate).await
            }
        }
        Commands::Analyze { provider } => cli::analyze::run_analyze(&provider).await,
        Commands::Implement {
            provider,
            task,
            all,
            dry_run,
        } => cli::implement::run_implement(&provider, task.as_deref(), all, dry_run).await,
        Commands::Run {
            command,
            capture,
            share,
        } => cli::run::run_command(&command, capture, share)
            .await
            .map(|_| ()),
        Commands::Test { cmd, fix } => cli::run::run_tests(cmd.as_deref(), fix).await.map(|_| ()),
        Commands::Lint { cmd, fix } => cli::run::run_lint(cmd.as_deref(), fix).await.map(|_| ()),
        Commands::Map { max_tokens } => cli::repomap::run_map(max_tokens).await,
        Commands::Chat {
            mode,
            message,
            provider,
        } => {
            let chat_mode = match mode {
                ChatModeArg::Ask => cli::mode::ChatMode::Ask,
                ChatModeArg::Code => cli::mode::ChatMode::Code,
                ChatModeArg::Architect => cli::mode::ChatMode::Architect,
                ChatModeArg::Help => cli::mode::ChatMode::Help,
            };
            cli::mode::run_mode(chat_mode, &message, &provider).await
        }

        // MCP Server
        Commands::Mcp { action } => cli::mcp::run(action).await,
    }
}
