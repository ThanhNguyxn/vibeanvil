//! Harvest command handler with beautiful output

use anyhow::Result;
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;

use crate::audit::{generate_session_id, AuditLogger};
use crate::brain::harvester::{DownloadMethod, HarvestConfig, Harvester};
use crate::brain::presets::PresetsFile;
use crate::brain::storage::BrainStorage;
use crate::cli::{HarvestArgs, HarvestCommands};
use crate::workspace;

pub async fn run(args: HarvestArgs) -> Result<()> {
    // Handle subcommands first
    if let Some(cmd) = &args.command {
        match cmd {
            HarvestCommands::Presets => {
                return list_presets().await;
            }
        }
    }

    // Load preset if specified
    let (queries, topics, language, min_stars, max_repos, updated_within_days, ignore_globs, allow_globs) = 
        if let Some(preset_name) = &args.preset {
            let cfg = load_preset_config(preset_name, &args)?;
            (cfg.queries, cfg.topics, cfg.language, cfg.min_stars, cfg.max_repos, cfg.updated_within_days, cfg.ignore_globs, cfg.allow_globs)
        } else {
            (
                args.query.clone(),
                args.topic.clone(),
                args.language.clone(),
                args.min_stars,
                args.max_repos,
                args.updated_within_days,
                args.ignore_glob.clone(),
                args.allow_glob.clone(),
            )
        };

    // Validate inputs
    if queries.is_empty() && topics.is_empty() {
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
            "vibeanvil harvest --preset cli_framework_patterns".cyan()
        );
        println!(
            "  {} {}",
            "•".dimmed(),
            "vibeanvil harvest presets  # List available presets".cyan()
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
    if !queries.is_empty() {
        println!("  {} {}", "Queries:".dimmed(), queries.join(", ").cyan());
    }
    if !topics.is_empty() {
        println!("  {} {}", "Topics: ".dimmed(), topics.join(", ").cyan());
    }
    if let Some(lang) = &language {
        println!("  {} {}", "Language:".dimmed(), lang.cyan());
    }
    println!(
        "  {} {} stars, {} repos max",
        "Filters:".dimmed(),
        format!("≥{}", min_stars).yellow(),
        max_repos.to_string().yellow()
    );
    println!(
        "  {} within {} days",
        "Updated:".dimmed(),
        updated_within_days.to_string().yellow()
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
        queries: queries.clone(),
        topics: topics.clone(),
        language: language.clone(),
        max_repos,
        min_stars,
        updated_within_days,
        download_method: match args.download {
            crate::cli::DownloadMethod::Tarball => DownloadMethod::Tarball,
            crate::cli::DownloadMethod::Git => DownloadMethod::Git,
        },
        cache_dir,
        ignore_globs: ignore_globs.clone(),
        allow_globs: allow_globs.clone(),
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
            format!("queries={:?}", queries),
                format!("sources={}", sources_processed),
                format!("records={}", total_records),
            ],
        )
        .await?;

    Ok(())
}

/// List all available harvest presets
async fn list_presets() -> Result<()> {
    let presets = PresetsFile::load()?;

    println!();
    println!(
        "{}",
        "╔═══════════════════════════════════════════════════════════════╗".cyan()
    );
    println!(
        "{}",
        "║               🌾 Available Harvest Presets                    ║".cyan()
    );
    println!(
        "{}",
        "╚═══════════════════════════════════════════════════════════════╝".cyan()
    );
    println!();

    for (key, name, purpose) in presets.list() {
        println!("  {} {}", "•".cyan(), key.white().bold());
        println!("    {} {}", "Name:".dimmed(), name);
        println!("    {} {}", "Purpose:".dimmed(), purpose);
        println!();
    }

    println!("{}", "─".repeat(60).dimmed());
    println!("{}", "💡 Usage:".white().bold());
    println!(
        "  {} {}",
        "•".cyan(),
        "vibeanvil harvest --preset cli_framework_patterns".white()
    );
    println!(
        "  {} {}",
        "•".cyan(),
        "vibeanvil harvest --preset ai_engineering_patterns --max-repos 10".white()
    );
    println!();

    Ok(())
}

/// Configuration from preset merged with CLI overrides
struct MergedConfig {
    queries: Vec<String>,
    topics: Vec<String>,
    language: Option<String>,
    min_stars: u32,
    max_repos: usize,
    updated_within_days: u32,
    ignore_globs: Vec<String>,
    allow_globs: Vec<String>,
}

/// Load preset configuration and merge with CLI overrides
fn load_preset_config(
    preset_name: &str,
    args: &HarvestArgs,
) -> Result<MergedConfig> {
    let presets_file = PresetsFile::load()?;
    
    let preset = presets_file.get(preset_name).ok_or_else(|| {
        anyhow::anyhow!(
            "Preset '{}' not found. Run 'vibeanvil harvest presets' to see available presets.",
            preset_name
        )
    })?;

    // Start with preset values
    let queries = if args.query.is_empty() {
        preset.queries.clone()
    } else {
        args.query.clone()
    };

    let topics = args.topic.clone(); // Presets don't have topics, only queries

    let language = args.language.clone().or_else(|| {
        preset.filters.languages.first().cloned()
    });

    let min_stars = if args.min_stars == 10 {
        // Default value, use preset
        preset.filters.min_stars.unwrap_or(presets_file.defaults.min_stars)
    } else {
        args.min_stars
    };

    let max_repos = if args.max_repos == 20 {
        // Default value, use preset
        presets_file.defaults.max_repos
    } else {
        args.max_repos
    };

    let updated_within_days = if args.updated_within_days == 365 {
        // Default value, use preset
        preset.filters.updated_within_days.unwrap_or(presets_file.defaults.updated_within_days)
    } else {
        args.updated_within_days
    };

    let ignore_globs = if args.ignore_glob.is_empty() {
        presets_file.defaults.ignore_globs.clone()
    } else {
        args.ignore_glob.clone()
    };

    let allow_globs = if args.allow_glob.is_empty() {
        preset.allow_globs.clone()
    } else {
        args.allow_glob.clone()
    };

    println!("{}", format!("📦 Using preset: {}", preset.name).green().bold());

    Ok(MergedConfig {
        queries,
        topics,
        language,
        min_stars,
        max_repos,
        updated_within_days,
        ignore_globs,
        allow_globs,
    })
}
