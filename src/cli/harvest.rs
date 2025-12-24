//! Harvest command handler

use anyhow::Result;
use std::path::PathBuf;

use crate::audit::{AuditLogger, generate_session_id};
use crate::brain::harvester::{DownloadMethod, HarvestConfig, Harvester};
use crate::brain::storage::BrainStorage;
use crate::cli::HarvestArgs;
use crate::workspace;

pub async fn run(args: HarvestArgs) -> Result<()> {
    // Validate inputs
    if args.query.is_empty() && args.topic.is_empty() {
        anyhow::bail!("At least one --query or --topic is required");
    }

    let session_id = generate_session_id();
    let logger = AuditLogger::new(&session_id);

    println!("🧠 Harvesting repos with dynamic search");
    println!();
    println!("  Queries: {:?}", args.query);
    println!("  Topics:  {:?}", args.topic);
    if let Some(lang) = &args.language {
        println!("  Language: {}", lang);
    }
    println!("  Min stars: {}, Max repos: {}", args.min_stars, args.max_repos);
    println!("  Updated within: {} days", args.updated_within_days);
    println!();

    // Check for GITHUB_TOKEN
    if std::env::var("GITHUB_TOKEN").is_err() {
        println!("⚠️  GITHUB_TOKEN not set. API rate limits will be restricted.");
        println!("   Set GITHUB_TOKEN environment variable for higher limits.");
        println!();
    }

    // Build config
    let cache_dir = args.cache_dir
        .map(PathBuf::from)
        .unwrap_or_else(|| workspace::cache_dir());

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

    // Search for repos
    println!("→ Searching GitHub...");
    let repos = harvester.search_repos().await?;
    
    if repos.is_empty() {
        println!("No repositories found matching the criteria.");
        return Ok(());
    }

    println!("  Found {} repositories", repos.len());
    println!();

    // Harvest each repo
    let mut total_records = 0;
    let mut total_chunks = 0;
    let mut sources_processed = 0;

    for repo in &repos {
        match harvester.harvest_repo(repo).await {
            Ok((source_meta, records)) => {
                if !records.is_empty() {
                    let chunk_count: usize = records.iter().map(|r| r.chunks.len()).sum();
                    storage.save_source(&source_meta).await?;
                    storage.save_records(&records).await?;
                    total_records += records.len();
                    total_chunks += chunk_count;
                    sources_processed += 1;
                    println!("    ✓ {} files, {} chunks", records.len(), chunk_count);
                } else if source_meta.license != "cached" {
                    println!("    ○ No relevant files");
                }
            }
            Err(e) => {
                println!("    ✗ Error: {}", e);
            }
        }
    }

    println!();
    println!("✓ Harvest complete");
    println!("  Sources processed: {}", sources_processed);
    println!("  Total records: {}", total_records);
    println!("  Total chunks: {}", total_chunks);
    println!();
    println!("Use 'vibeanvil brain stats' to view statistics");
    println!("Use 'vibeanvil brain search \"<query>\"' to search");

    logger.log_command(
        "harvest",
        vec![
            format!("queries={:?}", args.query),
            format!("sources={}", sources_processed),
            format!("records={}", total_records),
        ],
    ).await?;

    Ok(())
}
