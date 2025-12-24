//! Brain command handler

use anyhow::Result;
use std::path::PathBuf;

use crate::brain::storage::{BrainStorage, ExportFormat, ExportOptions};
use crate::cli::{BrainArgs, BrainCommands};

pub async fn run(args: BrainArgs) -> Result<()> {
    match args.command {
        BrainCommands::Stats => show_stats().await,
        BrainCommands::Search { query, limit } => search(&query, limit).await,
        BrainCommands::Export { format, output, include_source_ids } => {
            export(format, output, include_source_ids).await
        }
    }
}

async fn show_stats() -> Result<()> {
    let storage = BrainStorage::new().await?;
    let stats = storage.stats().await?;

    println!("🧠 BrainPack Statistics");
    println!();
    println!("  Sources:        {}", stats.total_sources);
    println!("  Records:        {}", stats.total_records);
    println!("  Chunks:         {}", stats.total_chunks);
    println!("  JSONL size:     {} bytes", stats.jsonl_size_bytes);
    println!("  SQLite size:    {} bytes", stats.sqlite_size_bytes);
    
    if let Some(updated) = stats.last_updated {
        println!("  Last updated:   {}", updated.format("%Y-%m-%d %H:%M:%S UTC"));
    }

    if !stats.by_type.is_empty() {
        println!();
        println!("  By content type:");
        for (content_type, count) in &stats.by_type {
            let ct_name = content_type.trim_matches('"');
            println!("    {:12} {}", ct_name, count);
        }
    }

    if !stats.by_language.is_empty() {
        println!();
        println!("  By language:");
        for (lang, count) in &stats.by_language {
            println!("    {:12} {}", lang, count);
        }
    }

    if !stats.by_license.is_empty() {
        println!();
        println!("  By license:");
        for (license, count) in &stats.by_license {
            println!("    {:12} {}", license, count);
        }
    }

    Ok(())
}

async fn search(query: &str, limit: usize) -> Result<()> {
    let storage = BrainStorage::new().await?;
    let results = storage.search(query, limit)?;

    if results.is_empty() {
        println!("No results found for: {}", query);
        println!();
        println!("Tips:");
        println!("  - Try broader search terms");
        println!("  - Use 'vibeanvil brain stats' to see what's indexed");
        println!("  - Run 'vibeanvil harvest --query \"...\"' to add more data");
        return Ok(());
    }

    println!("🔍 Search results for: {}", query);
    println!();

    for (i, result) in results.iter().enumerate() {
        println!("{}. {} ({})", i + 1, result.path, result.content_type);
        println!("   Source: {}", result.source_id);
        println!("   Score: {:.2}", result.score);
        
        // Show short snippet (never full file)
        let snippet = result.snippet
            .replace('\n', " ")
            .chars()
            .take(120)
            .collect::<String>();
        println!("   {}", snippet);
        
        if !result.tags.is_empty() {
            let tags: Vec<String> = result.tags.iter()
                .filter(|t| !t.is_empty())
                .take(5)
                .cloned()
                .collect();
            if !tags.is_empty() {
                println!("   Tags: {}", tags.join(", "));
            }
        }
        println!();
    }

    Ok(())
}

async fn export(
    format: crate::cli::ExportFormat,
    output: Option<String>,
    include_source_ids: bool,
) -> Result<()> {
    let storage = BrainStorage::new().await?;
    
    let options = ExportOptions {
        format: match format {
            crate::cli::ExportFormat::Jsonl => ExportFormat::Jsonl,
            crate::cli::ExportFormat::Md => ExportFormat::Markdown,
        },
        output_path: output.map(PathBuf::from),
        include_source_ids,
    };

    let output_path = storage.export(&options).await?;

    println!("✓ Exported to: {}", output_path);
    
    if !include_source_ids {
        println!();
        println!("Note: Source IDs were excluded for privacy.");
        println!("      Use --include-source-ids=true to include them.");
    }

    Ok(())
}
