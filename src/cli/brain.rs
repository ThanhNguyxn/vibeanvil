//! Brain command handler with beautiful output

use anyhow::Result;
use colored::Colorize;
use std::path::PathBuf;

use crate::brain::storage::{BrainStorage, ExportFormat, ExportOptions};
use crate::cli::{BrainArgs, BrainCommands};

pub async fn run(args: BrainArgs) -> Result<()> {
    match args.command {
        BrainCommands::Ensure {
            refresh_core,
            verbose,
        } => ensure_core(refresh_core, verbose).await,
        BrainCommands::Stats => show_stats().await,
        BrainCommands::Search { query, limit } => search(&query, limit).await,
        BrainCommands::Export {
            format,
            output,
            include_source_ids,
            limit,
        } => export(format, output, include_source_ids, limit).await,
        BrainCommands::Compact => compact().await,
    }
}

fn format_bytes(bytes: u64) -> String {
    if bytes >= 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} bytes", bytes)
    }
}

async fn show_stats() -> Result<()> {
    let storage = BrainStorage::new().await?;
    let stats = storage.stats().await?;

    println!();
    println!(
        "{}",
        "╔═══════════════════════════════════════════════════════════════╗".cyan()
    );
    println!(
        "{}",
        "║               🧠 BrainPack Statistics                         ║".cyan()
    );
    println!(
        "{}",
        "╚═══════════════════════════════════════════════════════════════╝".cyan()
    );
    println!();

    // Main stats box
    println!("{}", "┌─────────────────────────────────────────┐".white());
    println!(
        "│  {} {:>12}                    │",
        "📚 Sources:   ".white().bold(),
        stats.total_sources.to_string().green().bold()
    );
    println!(
        "│  {} {:>12}                    │",
        "📄 Records:   ".white().bold(),
        stats.total_records.to_string().green().bold()
    );
    println!(
        "│  {} {:>12}                    │",
        "🧩 Chunks:    ".white().bold(),
        stats.total_chunks.to_string().green().bold()
    );
    println!(
        "│  {} {:>12}                    │",
        "📁 JSONL:     ".white().bold(),
        format_bytes(stats.jsonl_size_bytes).cyan()
    );
    println!(
        "│  {} {:>12}                    │",
        "🗄️  SQLite:    ".white().bold(),
        format_bytes(stats.sqlite_size_bytes).cyan()
    );
    println!("{}", "└─────────────────────────────────────────┘".white());

    if let Some(updated) = stats.last_updated {
        println!();
        println!(
            "  {} {}",
            "🕐 Last updated:".dimmed(),
            updated.format("%Y-%m-%d %H:%M:%S UTC").to_string().dimmed()
        );
    }

    if !stats.by_type.is_empty() {
        println!();
        println!("{}", "  📊 By Content Type:".white().bold());
        for (content_type, count) in &stats.by_type {
            let ct_name = content_type.trim_matches('"');
            let bar_len = (*count as u64 * 20) / (stats.total_chunks as u64).max(1);
            let bar = "█".repeat(bar_len as usize);
            println!(
                "    {} {:12} {:<20} ({})",
                "•".cyan(),
                ct_name,
                bar.green(),
                count.to_string().dimmed()
            );
        }
    }

    if !stats.by_language.is_empty() {
        println!();
        println!("{}", "  💻 By Language:".white().bold());
        for (lang, count) in &stats.by_language {
            let bar_len = (*count as u64 * 20) / (stats.total_chunks as u64).max(1);
            let bar = "█".repeat(bar_len as usize);
            println!(
                "    {} {:12} {:<20} ({})",
                "•".cyan(),
                lang,
                bar.blue(),
                count.to_string().dimmed()
            );
        }
    }

    if !stats.by_license.is_empty() {
        println!();
        println!("{}", "  📜 By License:".white().bold());
        for (license, count) in &stats.by_license {
            println!(
                "    {} {:20} {}",
                "•".cyan(),
                license,
                count.to_string().dimmed()
            );
        }
    }

    // Tips
    println!();
    println!("{}", "─".repeat(50).dimmed());
    println!(
        "{}",
        "💡 Tip: Use 'vibeanvil brain search <query>' to find knowledge".dimmed()
    );
    println!();

    Ok(())
}

async fn search(query: &str, limit: usize) -> Result<()> {
    let storage = BrainStorage::new().await?;

    println!();
    println!(
        "{} {} {}",
        "🔍".cyan(),
        "Searching for:".white().bold(),
        query.cyan().bold()
    );
    println!();

    let results = storage.search(query, limit)?;

    if results.is_empty() {
        println!("{}", "┌─────────────────────────────────────────┐".yellow());
        println!(
            "{}",
            "│  ⚠️  No results found                    │".yellow()
        );
        println!("{}", "└─────────────────────────────────────────┘".yellow());
        println!();
        println!("{}", "💡 Tips:".white().bold());
        println!("  {} Try broader search terms", "•".dimmed());
        println!(
            "  {} Use 'vibeanvil brain stats' to see what's indexed",
            "•".dimmed()
        );
        println!(
            "  {} Run 'vibeanvil harvest -q \"...\"' to add more data",
            "•".dimmed()
        );
        println!();
        return Ok(());
    }

    println!(
        "  Found {} results",
        results.len().to_string().green().bold()
    );
    println!();

    for (i, result) in results.iter().enumerate() {
        // Result header
        println!(
            "{}",
            format!("┌─── Result {} ───────────────────────────────┐", i + 1).cyan()
        );

        // Score and type
        println!(
            "│  {} {:.2}  │  {} {}",
            "Score:".dimmed(),
            result.score,
            "Type:".dimmed(),
            result.content_type.green()
        );

        // Path
        println!("│  {} {}", "Path:".dimmed(), result.path.cyan());

        // Source (anonymized)
        println!(
            "│  {} {}...",
            "Source:".dimmed(),
            result.source_id[..16.min(result.source_id.len())].dimmed()
        );

        // Snippet
        let snippet = result
            .snippet
            .replace('\n', " ")
            .chars()
            .take(100)
            .collect::<String>();
        println!("│");
        println!("│  {}", snippet.white());

        // Tags
        if !result.tags.is_empty() {
            let tags: Vec<String> = result
                .tags
                .iter()
                .filter(|t| !t.is_empty())
                .take(5)
                .cloned()
                .collect();
            if !tags.is_empty() {
                println!("│");
                println!(
                    "│  {} {}",
                    "Tags:".dimmed(),
                    tags.iter()
                        .map(|t| format!("#{}", t).yellow().to_string())
                        .collect::<Vec<_>>()
                        .join(" ")
                );
            }
        }

        println!(
            "{}",
            "└─────────────────────────────────────────────┘".cyan()
        );
        println!();
    }

    Ok(())
}

async fn export(
    format: crate::cli::ExportFormat,
    output: Option<String>,
    include_source_ids: bool,
    limit: usize,
) -> Result<()> {
    println!();
    println!(
        "{} {}",
        "📤".cyan(),
        "Exporting BrainPack...".white().bold()
    );

    let storage = BrainStorage::new().await?;

    let options = ExportOptions {
        format: match format {
            crate::cli::ExportFormat::Jsonl => ExportFormat::Jsonl,
            crate::cli::ExportFormat::Md => ExportFormat::Markdown,
        },
        output_path: output.map(PathBuf::from),
        include_source_ids,
        limit,
    };

    let output_path = storage.export(&options).await?;

    println!();
    println!(
        "{}",
        "┌─────────────────────────────────────────────┐".green()
    );
    println!(
        "{}",
        "│  ✅ Export completed successfully!          │".green()
    );
    println!(
        "{}",
        "└─────────────────────────────────────────────┘".green()
    );
    println!();
    println!("  {} {}", "📁 Output:".white().bold(), output_path.cyan());

    if !include_source_ids {
        println!();
        println!(
            "  {} {}",
            "🔒".dimmed(),
            "Source IDs excluded for privacy".dimmed()
        );
        println!(
            "  {} {}",
            "   ".dimmed(),
            "Use --include-source-ids to include them".dimmed()
        );
    }

    println!();

    Ok(())
}

async fn ensure_core(refresh_core: bool, verbose: bool) -> Result<()> {
    use indicatif::{ProgressBar, ProgressStyle};

    println!();
    println!(
        "{}",
        "╔═══════════════════════════════════════════════════════════════╗".cyan()
    );
    println!(
        "{}",
        "║               🧠 Core BrainPack Setup                         ║".cyan()
    );
    println!(
        "{}",
        "╚═══════════════════════════════════════════════════════════════╝".cyan()
    );
    println!();

    let storage = BrainStorage::new().await?;
    let fingerprint = crate::brain::storage::BrainStorage::core_fingerprint();
    let brainpack_dir = crate::workspace::brainpack_dir();

    // Import core brainpack (handles upgrade detection internally)
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    spinner.set_message(if refresh_core {
        "Force refreshing Core BrainPack..."
    } else {
        "Checking Core BrainPack..."
    });
    spinner.enable_steady_tick(std::time::Duration::from_millis(100));

    let stats = storage.import_core(refresh_core).await?;
    spinner.finish_and_clear();

    // Check if nothing was imported (already up-to-date)
    if stats.inserted == 0 && !refresh_core {
        let core_chunks = storage.get_source_chunk_count("core");
        println!(
            "  {} {}",
            "✓".green(),
            "Core BrainPack already up-to-date".green().bold()
        );
        println!(
            "  {} {} core chunks installed",
            "📊".dimmed(),
            core_chunks.to_string().cyan(),
        );
        println!("  {} Fingerprint: {}", "🔑".dimmed(), fingerprint.dimmed());
        println!();
        println!("{}", "─".repeat(50).dimmed());
        println!(
            "{}",
            "💡 Try: vibeanvil brain search 'acceptance criteria'".dimmed()
        );
        println!("{}", "💡 Tip: Use --refresh-core to force refresh".dimmed());
        println!();
        return Ok(());
    }

    // Show success message
    let action = if stats.was_upgrade {
        "upgraded"
    } else {
        "installed"
    };
    println!(
        "{}",
        "┌─────────────────────────────────────────────┐".green()
    );
    println!(
        "│  {} Core BrainPack {} successfully!  │",
        "✅".green(),
        action.green().bold()
    );
    println!(
        "{}",
        "└─────────────────────────────────────────────┘".green()
    );
    println!();

    // Show detailed import statistics
    println!(
        "  {} {} chunks inserted ({} lines parsed)",
        "📦".white(),
        stats.inserted.to_string().cyan().bold(),
        stats.total_lines.to_string().dimmed()
    );

    if stats.skipped_errors > 0 {
        println!(
            "  {} {} lines skipped (parse errors)",
            "⚠️".yellow(),
            stats.skipped_errors.to_string().yellow()
        );

        // Show error line numbers in verbose mode
        if verbose && !stats.error_lines.is_empty() {
            println!(
                "  {} Error line numbers: {}",
                "📋".dimmed(),
                stats
                    .error_lines
                    .iter()
                    .map(|n| n.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
                    .dimmed()
            );
        }
    }

    if stats.was_upgrade {
        println!(
            "  {} {}",
            "🔄".white(),
            "Previous core data was replaced with new version".dimmed()
        );
    }

    println!("  {} Fingerprint: {}", "🔑".dimmed(), fingerprint.dimmed());

    // Show log/data path
    println!(
        "  {} Data: {}",
        "📁".dimmed(),
        brainpack_dir.display().to_string().dimmed()
    );

    println!();
    println!("{}", "─".repeat(50).dimmed());
    println!("{}", "💡 Quick starts:".white().bold());
    println!(
        "  {} {}",
        "•".cyan(),
        "vibeanvil brain search 'web contract'".white()
    );
    println!(
        "  {} {}",
        "•".cyan(),
        "vibeanvil brain search 'acceptance criteria'".white()
    );
    println!(
        "  {} {}",
        "•".cyan(),
        "vibeanvil brain search 'iterate loop'".white()
    );

    // Add hint to run compact for clean stats
    if stats.was_upgrade {
        println!();
        println!(
            "{}",
            "💡 Tip: Run 'vibeanvil brain compact' to keep JSONL stats clean.".dimmed()
        );
    }
    println!();

    Ok(())
}

/// Compact the brain pack (dedup JSONL, optimize SQLite)
async fn compact() -> Result<()> {
    println!();
    println!(
        "{}",
        "╔═══════════════════════════════════════════════════════════════╗".cyan()
    );
    println!(
        "{}",
        "║               🧹 BrainPack Compact                            ║".cyan()
    );
    println!(
        "{}",
        "╚═══════════════════════════════════════════════════════════════╝".cyan()
    );
    println!();

    let storage = BrainStorage::new().await?;
    let before_stats = storage.stats().await?;

    println!("{}", "📊 Before:".white().bold());
    println!(
        "  {} {}",
        "JSONL:".dimmed(),
        format_bytes(before_stats.jsonl_size_bytes).cyan()
    );
    println!(
        "  {} {}",
        "SQLite:".dimmed(),
        format_bytes(before_stats.sqlite_size_bytes).cyan()
    );
    println!(
        "  {} {}",
        "Chunks:".dimmed(),
        before_stats.total_chunks.to_string().cyan()
    );
    println!();

    println!("{}", "⏳ Compacting...".yellow());
    let result = storage.compact().await?;

    let after_stats = storage.stats().await?;

    println!();
    println!("{}", "📊 After:".white().bold());
    println!(
        "  {} {}",
        "JSONL:".dimmed(),
        format_bytes(after_stats.jsonl_size_bytes).green()
    );
    println!(
        "  {} {}",
        "SQLite:".dimmed(),
        format_bytes(after_stats.sqlite_size_bytes).green()
    );
    println!(
        "  {} {}",
        "Records:".dimmed(),
        result.records_written.to_string().green()
    );
    println!();

    let saved = before_stats
        .jsonl_size_bytes
        .saturating_sub(after_stats.jsonl_size_bytes)
        + before_stats
            .sqlite_size_bytes
            .saturating_sub(after_stats.sqlite_size_bytes);

    if saved > 0 {
        println!(
            "{} {}",
            "✅ Saved:".green().bold(),
            format_bytes(saved).green().bold()
        );
    } else {
        println!("{}", "✅ Already compact!".green().bold());
    }
    println!();

    Ok(())
}
