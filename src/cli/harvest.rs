//! Harvest command handler with beautiful output

use anyhow::Result;
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;

use crate::audit::{generate_session_id, AuditLogger};
use crate::brain::harvester::{DownloadMethod, HarvestConfig, Harvester};
use crate::brain::storage::BrainStorage;
use crate::cli::HarvestArgs;
use crate::workspace;

pub async fn run(args: HarvestArgs) -> Result<()> {
    // Validate inputs
    if args.query.is_empty() && args.topic.is_empty() {
        println!();
        println!(
            "{}",
            "┌─────────────────────────────────────────────┐".red()
        );
        println!(
            "{}",
            "│  ❌ At least one --query or --topic required │".red()
        );
        println!(
            "{}",
            "└─────────────────────────────────────────────┘".red()
        );
        println!();
        println!("{}", "💡 Examples:".white().bold());
        println!(
            "  {} {}",
            "•".dimmed(),
            "vibeanvil harvest -t rust -t cli".cyan()
        );
        println!(
            "  {} {}",
            "•".dimmed(),
            "vibeanvil harvest -q \"machine learning\" -l python".cyan()
        );
        println!();
        return Ok(());
    }

    let session_id = generate_session_id();
    let logger = AuditLogger::new(&session_id);

    // Print beautiful header
    println!();
    println!(
        "{}",
        "╔═══════════════════════════════════════════════════════════════╗".cyan()
    );
    println!(
        "{}",
        "║               🌾 VibeAnvil Harvester                          ║".cyan()
    );
    println!(
        "{}",
        "╚═══════════════════════════════════════════════════════════════╝".cyan()
    );
    println!();

    // Show search parameters
    println!("{}", "📋 Search Parameters:".white().bold());
    if !args.query.is_empty() {
        println!("  {} {}", "Queries:".dimmed(), args.query.join(", ").cyan());
    }
    if !args.topic.is_empty() {
        println!("  {} {}", "Topics: ".dimmed(), args.topic.join(", ").cyan());
    }
    if let Some(lang) = &args.language {
        println!("  {} {}", "Language:".dimmed(), lang.cyan());
    }
    println!(
        "  {} {} stars, {} repos max",
        "Filters:".dimmed(),
        format!("≥{}", args.min_stars).yellow(),
        args.max_repos.to_string().yellow()
    );
    println!(
        "  {} within {} days",
        "Updated:".dimmed(),
        args.updated_within_days.to_string().yellow()
    );
    println!();

    // Check for GITHUB_TOKEN
    if std::env::var("GITHUB_TOKEN").is_err() {
        println!(
            "{}",
            "┌─────────────────────────────────────────────────────────┐".yellow()
        );
        println!(
            "{}",
            "│  ⚠️  GITHUB_TOKEN not set - API rate limits restricted   │".yellow()
        );
        println!(
            "{}",
            "│  Set GITHUB_TOKEN environment variable for more         │".white()
        );
        println!(
            "{}",
            "└─────────────────────────────────────────────────────────┘".yellow()
        );
        println!();
    }

    // Build config
    let cache_dir = args
        .cache_dir
        .map(PathBuf::from)
        .unwrap_or_else(workspace::cache_dir);

    let config = HarvestConfig {
        queries: args.query.clone(),
        topics: args.topic.clone(),
        language: args.language.clone(),
        max_repos: args.max_repos,
        min_stars: args.min_stars,
        updated_within_days: args.updated_within_days,
        download_method: match args.download {
            crate::cli::DownloadMethod::Tarball => DownloadMethod::Tarball,
            crate::cli::DownloadMethod::Git => DownloadMethod::Git,
        },
        cache_dir,
        ignore_globs: args.ignore_glob.clone(),
        allow_globs: args.allow_glob.clone(),
        ..Default::default()
    };

    let mut harvester = Harvester::new(config).await?;
    let storage = BrainStorage::new().await?;

    // Search spinner
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    spinner.set_message("Searching GitHub...");
    spinner.enable_steady_tick(std::time::Duration::from_millis(100));

    let repos = harvester.search_repos().await?;
    spinner.finish_and_clear();

    if repos.is_empty() {
        println!(
            "{}",
            "┌─────────────────────────────────────────────┐".yellow()
        );
        println!(
            "{}",
            "│  ⚠️  No repositories found                   │".yellow()
        );
        println!(
            "{}",
            "└─────────────────────────────────────────────┘".yellow()
        );
        println!();
        println!("{}", "💡 Try:".white().bold());
        println!("  {} Broader search terms", "•".dimmed());
        println!("  {} Lower --min-stars value", "•".dimmed());
        println!("  {} Different topics or language", "•".dimmed());
        println!();
        return Ok(());
    }

    println!(
        "  {} Found {} repositories",
        "✓".green(),
        repos.len().to_string().green().bold()
    );
    println!();

    // Progress bar for harvesting
    let progress = ProgressBar::new(repos.len() as u64);
    progress.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("█▓░"),
    );

    // Harvest each repo
    let mut total_records = 0;
    let mut total_chunks = 0;
    let mut sources_processed = 0;
    let mut errors = 0;

    for repo in &repos {
        let short_name = if repo.full_name.len() > 30 {
            format!("{}...", &repo.full_name[..27])
        } else {
            repo.full_name.clone()
        };
        progress.set_message(short_name);

        match harvester.harvest_repo(repo).await {
            Ok((source_meta, records)) => {
                if !records.is_empty() {
                    let chunk_count: usize = records.iter().map(|r| r.chunks.len()).sum();
                    storage.save_source(&source_meta).await?;
                    storage.save_records(&records).await?;
                    total_records += records.len();
                    total_chunks += chunk_count;
                    sources_processed += 1;
                }
            }
            Err(_) => {
                errors += 1;
            }
        }
        progress.inc(1);
    }
    progress.finish_and_clear();

    // Summary
    println!();
    println!(
        "{}",
        "╔═══════════════════════════════════════════╗".green()
    );
    println!(
        "{}",
        "║     🎉 Harvest Complete!                  ║".green()
    );
    println!(
        "{}",
        "╠═══════════════════════════════════════════╣".green()
    );
    println!(
        "║  {} {:>8}                       ║",
        "📚 Sources:".white(),
        sources_processed.to_string().cyan()
    );
    println!(
        "║  {} {:>8}                       ║",
        "📄 Records:".white(),
        total_records.to_string().cyan()
    );
    println!(
        "║  {} {:>8}                       ║",
        "🧩 Chunks: ".white(),
        total_chunks.to_string().cyan()
    );
    if errors > 0 {
        println!(
            "║  {} {:>8}                       ║",
            "❌ Errors: ".white(),
            errors.to_string().red()
        );
    }
    println!(
        "{}",
        "╚═══════════════════════════════════════════╝".green()
    );
    println!();

    // Next steps
    println!("{}", "─".repeat(50).dimmed());
    println!("{}", "💡 Next steps:".white().bold());
    println!("  {} {}", "•".cyan(), "vibeanvil brain stats".white());
    println!(
        "  {} {}",
        "•".cyan(),
        "vibeanvil brain search \"<query>\"".white()
    );
    println!();

    logger
        .log_command(
            "harvest",
            vec![
                format!("queries={:?}", args.query),
                format!("sources={}", sources_processed),
                format!("records={}", total_records),
            ],
        )
        .await?;

    Ok(())
}
