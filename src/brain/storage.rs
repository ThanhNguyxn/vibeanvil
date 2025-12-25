//! BrainPack storage - JSONL and SQLite FTS5
//!
//! Privacy-first storage:
//! - No URLs stored by default
//! - Anonymized source IDs
//! - Clean export options

use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use tokio::fs;

use super::{BrainRecord, BrainStats, SearchResult, SourceMeta};
use crate::workspace;

/// Export format
#[derive(Debug, Clone)]
pub enum ExportFormat {
    Jsonl,
    Markdown,
}

/// Export options
#[derive(Debug, Clone)]
pub struct ExportOptions {
    pub format: ExportFormat,
    pub output_path: Option<PathBuf>,
    /// Include anonymized source IDs (default: true)
    pub include_source_ids: bool,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            format: ExportFormat::Jsonl,
            output_path: None,
            include_source_ids: true,
        }
    }
}

/// Storage for brain records
pub struct BrainStorage {
    brainpack_dir: PathBuf,
    jsonl_path: PathBuf,
    sqlite_path: PathBuf,
}

impl BrainStorage {
    /// Create new brain storage
    pub async fn new() -> Result<Self> {
        let brainpack_dir = workspace::brainpack_dir();
        fs::create_dir_all(&brainpack_dir).await?;

        let jsonl_path = brainpack_dir.join("brainpack.jsonl");
        let sqlite_path = brainpack_dir.join("brainpack.sqlite");

        let storage = Self {
            brainpack_dir,
            jsonl_path,
            sqlite_path,
        };

        storage.init_db()?;

        Ok(storage)
    }

    /// Initialize SQLite database with FTS5
    fn init_db(&self) -> Result<()> {
        let conn = Connection::open(&self.sqlite_path)?;

        // Sources table (metadata, no URLs)
        conn.execute(
            "CREATE TABLE IF NOT EXISTS sources (
                source_id TEXT PRIMARY KEY,
                \"commit\" TEXT,
                license TEXT,
                language TEXT,
                fetched_at TEXT,
                files_count INTEGER,
                chunks_count INTEGER
            )",
            [],
        )?;

        // Chunks table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS brain_chunks (
                chunk_id TEXT PRIMARY KEY,
                source_id TEXT NOT NULL,
                path TEXT NOT NULL,
                content_type TEXT NOT NULL,
                start_line INTEGER,
                end_line INTEGER,
                text TEXT NOT NULL,
                signals TEXT,
                tags TEXT,
                FOREIGN KEY (source_id) REFERENCES sources(source_id)
            )",
            [],
        )?;

        // FTS5 virtual table for search
        conn.execute(
            "CREATE VIRTUAL TABLE IF NOT EXISTS chunks_fts USING fts5(
                text,
                tags,
                path,
                content='brain_chunks',
                content_rowid='rowid'
            )",
            [],
        )?;

        // Triggers to keep FTS in sync
        conn.execute(
            "CREATE TRIGGER IF NOT EXISTS chunks_ai AFTER INSERT ON brain_chunks BEGIN
                INSERT INTO chunks_fts(rowid, text, tags, path)
                VALUES (new.rowid, new.text, new.tags, new.path);
            END",
            [],
        )?;

        conn.execute(
            "CREATE TRIGGER IF NOT EXISTS chunks_ad AFTER DELETE ON brain_chunks BEGIN
                INSERT INTO chunks_fts(chunks_fts, rowid, text, tags, path)
                VALUES('delete', old.rowid, old.text, old.tags, old.path);
            END",
            [],
        )?;

        Ok(())
    }

    /// Save source metadata
    pub async fn save_source(&self, source: &SourceMeta) -> Result<()> {
        let conn = Connection::open(&self.sqlite_path)?;

        conn.execute(
            "INSERT OR REPLACE INTO sources 
            (source_id, \"commit\", license, language, fetched_at, files_count, chunks_count)
            VALUES (?, ?, ?, ?, ?, ?, ?)",
            params![
                source.source_id,
                source.commit,
                source.license,
                source.language,
                source.fetched_at.to_rfc3339(),
                source.files_count,
                source.chunks_count,
            ],
        )?;

        Ok(())
    }

    /// Save records (append to JSONL, insert to SQLite)
    pub async fn save_records(&self, records: &[BrainRecord]) -> Result<usize> {
        if records.is_empty() {
            return Ok(0);
        }

        // Append to JSONL
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.jsonl_path)
            .context("Failed to open JSONL file")?;

        for record in records {
            let json = serde_json::to_string(record)?;
            writeln!(file, "{}", json)?;
        }

        // Insert chunks to SQLite
        let conn = Connection::open(&self.sqlite_path)?;

        for record in records {
            let signals_json = serde_json::to_string(&record.signals)?;
            let tags_str = record.tags.join(",");

            for chunk in &record.chunks {
                conn.execute(
                    "INSERT OR REPLACE INTO brain_chunks 
                    (chunk_id, source_id, path, content_type, start_line, end_line, text, signals, tags)
                    VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
                    params![
                        chunk.chunk_id,
                        record.source_id,
                        record.path,
                        serde_json::to_string(&record.content_type)?,
                        chunk.start_line,
                        chunk.end_line,
                        chunk.text,
                        signals_json,
                        tags_str,
                    ],
                )?;
            }
        }

        Ok(records.len())
    }

    /// Search the brain using FTS5
    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        let conn = Connection::open(&self.sqlite_path)?;

        // Use FTS5 search with BM25 ranking
        let mut stmt = conn.prepare(
            "SELECT c.chunk_id, c.source_id, c.path, c.content_type,
                    snippet(chunks_fts, 0, '→', '←', '...', 20) as snippet,
                    bm25(chunks_fts) as score,
                    c.tags
            FROM chunks_fts
            JOIN brain_chunks c ON chunks_fts.rowid = c.rowid
            WHERE chunks_fts MATCH ?
            ORDER BY score
            LIMIT ?",
        )?;

        let results = stmt.query_map(params![query, limit as i64], |row| {
            let tags_str: String = row.get(6)?;
            let tags: Vec<String> = tags_str.split(',').map(|s| s.to_string()).collect();

            Ok(SearchResult {
                chunk_id: row.get(0)?,
                source_id: row.get(1)?,
                path: row.get(2)?,
                content_type: row.get(3)?,
                snippet: row.get(4)?,
                score: row.get::<_, f64>(5)?.abs(),
                tags,
            })
        })?;

        let mut search_results = vec![];
        for result in results {
            search_results.push(result?);
        }

        Ok(search_results)
    }

    /// Get statistics
    pub async fn stats(&self) -> Result<BrainStats> {
        let mut stats = BrainStats::default();

        // File sizes
        if self.jsonl_path.exists() {
            stats.jsonl_size_bytes = fs::metadata(&self.jsonl_path).await?.len();
        }
        if self.sqlite_path.exists() {
            stats.sqlite_size_bytes = fs::metadata(&self.sqlite_path).await?.len();
        }

        // Query SQLite for counts
        if self.sqlite_path.exists() {
            let conn = Connection::open(&self.sqlite_path)?;

            // Total sources
            let sources: i64 = conn
                .query_row("SELECT COUNT(*) FROM sources", [], |row| row.get(0))
                .unwrap_or(0);
            stats.total_sources = sources as usize;

            // Total chunks
            let chunks: i64 = conn
                .query_row("SELECT COUNT(*) FROM brain_chunks", [], |row| row.get(0))
                .unwrap_or(0);
            stats.total_chunks = chunks as usize;

            // Count JSONL records
            if self.jsonl_path.exists() {
                let file = std::fs::File::open(&self.jsonl_path)?;
                let reader = BufReader::new(file);
                stats.total_records = reader.lines().count();
            }

            // By content type
            let mut stmt = conn
                .prepare("SELECT content_type, COUNT(*) FROM brain_chunks GROUP BY content_type")?;
            let results = stmt.query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
            })?;
            for (ct, count) in results.flatten() {
                stats.by_type.insert(ct, count as usize);
            }

            // By language
            let mut stmt =
                conn.prepare("SELECT language, COUNT(*) FROM sources GROUP BY language")?;
            let results = stmt.query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
            })?;
            for (lang, count) in results.flatten() {
                stats.by_language.insert(lang, count as usize);
            }

            // By license
            let mut stmt =
                conn.prepare("SELECT license, COUNT(*) FROM sources GROUP BY license")?;
            let results = stmt.query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
            })?;
            for (license, count) in results.flatten() {
                stats.by_license.insert(license, count as usize);
            }

            // Last updated
            let last: Option<String> = conn
                .query_row("SELECT MAX(fetched_at) FROM sources", [], |row| row.get(0))
                .ok();

            if let Some(timestamp) = last {
                stats.last_updated = chrono::DateTime::parse_from_rfc3339(&timestamp)
                    .ok()
                    .map(|dt| dt.with_timezone(&chrono::Utc));
            }
        }

        Ok(stats)
    }

    /// Check if core brainpack is installed (source_id = "core")
    pub fn has_core_installed(&self) -> bool {
        if !self.sqlite_path.exists() {
            return false;
        }

        let conn = match Connection::open(&self.sqlite_path) {
            Ok(c) => c,
            Err(_) => return false,
        };

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM brain_chunks WHERE source_id = 'core'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);

        count > 0
    }

    /// Export to JSONL (privacy-clean by default)
    pub async fn export(&self, options: &ExportOptions) -> Result<String> {
        let output_path = options.output_path.clone().unwrap_or_else(|| {
            let ext = match options.format {
                ExportFormat::Jsonl => "jsonl",
                ExportFormat::Markdown => "md",
            };
            std::env::current_dir()
                .unwrap()
                .join(format!("brainpack_export.{}", ext))
        });

        match options.format {
            ExportFormat::Jsonl => self.export_jsonl(&output_path, options).await,
            ExportFormat::Markdown => self.export_markdown(&output_path, options).await,
        }
    }

    async fn export_jsonl(&self, output_path: &PathBuf, options: &ExportOptions) -> Result<String> {
        if !self.jsonl_path.exists() {
            tokio::fs::write(output_path, "").await?;
            return Ok(output_path.to_string_lossy().to_string());
        }

        let input = std::fs::File::open(&self.jsonl_path)?;
        let reader = BufReader::new(input);
        let mut output = std::fs::File::create(output_path)?;

        for line in reader.lines().map_while(Result::ok) {
            if let Ok(mut record) = serde_json::from_str::<serde_json::Value>(&line) {
                // Remove source_id if not requested
                if !options.include_source_ids {
                    if let Some(obj) = record.as_object_mut() {
                        obj.remove("source_id");
                    }
                }
                writeln!(output, "{}", serde_json::to_string(&record)?)?;
            }
        }

        Ok(output_path.to_string_lossy().to_string())
    }

    async fn export_markdown(
        &self,
        output_path: &PathBuf,
        options: &ExportOptions,
    ) -> Result<String> {
        let mut content = String::from("# BrainPack Export\n\n");
        content.push_str(&format!(
            "Exported at: {}\n\n",
            chrono::Utc::now().to_rfc3339()
        ));

        if !self.jsonl_path.exists() {
            content.push_str("*No records found.*\n");
            tokio::fs::write(output_path, &content).await?;
            return Ok(output_path.to_string_lossy().to_string());
        }

        let file = std::fs::File::open(&self.jsonl_path)?;
        let reader = BufReader::new(file);

        let mut current_source = String::new();
        let mut record_count = 0;

        for line in reader.lines().map_while(Result::ok) {
            // Try parsing as BrainRecord first (harvested records)
            if let Ok(record) = serde_json::from_str::<BrainRecord>(&line) {
                record_count += 1;

                if options.include_source_ids && record.source_id != current_source {
                    current_source = record.source_id.clone();
                    content.push_str(&format!("\n## Source: {}\n\n", current_source));
                }

                content.push_str(&format!("### {}\n\n", record.path));
                content.push_str(&format!(
                    "**Type**: {:?} | **Language**: {} | **License**: {}\n\n",
                    record.content_type, record.language, record.license
                ));

                if !record.signals.is_empty() {
                    content.push_str("**Signals**: ");
                    content.push_str(
                        &record
                            .signals
                            .iter()
                            .map(|s| format!("{:?}", s))
                            .collect::<Vec<_>>()
                            .join(", "),
                    );
                    content.push_str("\n\n");
                }

                content.push_str(&format!("{}\n\n", record.summary));

                // Only show first chunk
                if let Some(chunk) = record.chunks.first() {
                    content.push_str("```\n");
                    content.push_str(&chunk.text[..chunk.text.len().min(300)]);
                    if chunk.text.len() > 300 {
                        content.push_str("\n... (truncated)");
                    }
                    content.push_str("\n```\n\n");
                }
            } else if let Ok(core_entry) = serde_json::from_str::<serde_json::Value>(&line) {
                // Fallback: parse core entries (different schema)
                record_count += 1;

                let source_id = core_entry["source_id"].as_str().unwrap_or("unknown");
                if options.include_source_ids && source_id != current_source {
                    current_source = source_id.to_string();
                    content.push_str(&format!("\n## Source: {}\n\n", current_source));
                }

                let title = core_entry["title"].as_str().unwrap_or("Untitled");
                let entry_type = core_entry["type"].as_str().unwrap_or("unknown");
                let summary = core_entry["summary"].as_str().unwrap_or("");

                content.push_str(&format!("### {}\n\n", title));
                content.push_str(&format!("**Type**: {}\n\n", entry_type));
                content.push_str(&format!("{}\n\n", summary));

                // Show first chunk if available
                if let Some(chunks) = core_entry["chunks"].as_array() {
                    if let Some(chunk) = chunks.first() {
                        if let Some(text) = chunk["text"].as_str() {
                            content.push_str("```\n");
                            content.push_str(&text[..text.len().min(300)]);
                            if text.len() > 300 {
                                content.push_str("\n... (truncated)");
                            }
                            content.push_str("\n```\n\n");
                        }
                    }
                }
            }

            // Limit to 50 records in markdown
            if record_count >= 50 {
                content.push_str("\n*... and more records (truncated for readability)*\n");
                break;
            }
        }

        tokio::fs::write(output_path, &content).await?;
        Ok(output_path.to_string_lossy().to_string())
    }

    /// Import the core brainpack from embedded data
    pub async fn import_core(&self) -> Result<usize> {
        // Embedded core brainpack JSONL
        const CORE_JSONL: &str = include_str!("../../brainpacks/core/core.jsonl");

        let mut count = 0;
        let conn = Connection::open(&self.sqlite_path)?;

        // First, insert the "core" source record to satisfy foreign key
        conn.execute(
            "INSERT OR REPLACE INTO sources (source_id, \"commit\", license, language, fetched_at, files_count, chunks_count)
             VALUES ('core', 'embedded', 'MIT', 'mixed', datetime('now'), 0, 0)",
            [],
        )?;

        // Also append to JSONL file
        let mut jsonl_file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.jsonl_path)
            .context("Failed to open JSONL file")?;

        for line in CORE_JSONL.lines() {
            if line.trim().is_empty() {
                continue;
            }

            // Parse the core record format
            let record: serde_json::Value =
                serde_json::from_str(line).context("Failed to parse core JSONL line")?;

            let source_id = record["source_id"].as_str().unwrap_or("core");
            let record_type = record["type"].as_str().unwrap_or("template");
            let title = record["title"].as_str().unwrap_or("");
            let summary = record["summary"].as_str().unwrap_or("");
            let signals = record["signals"].clone();
            let tags = record["tags"].clone();

            let signals_str = serde_json::to_string(&signals)?;
            let tags_str = if let Some(arr) = tags.as_array() {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .collect::<Vec<_>>()
                    .join(",")
            } else {
                String::new()
            };

            // Insert chunks
            if let Some(chunks) = record["chunks"].as_array() {
                for chunk in chunks {
                    let chunk_id = chunk["chunk_id"].as_str().unwrap_or("");
                    let text = chunk["text"].as_str().unwrap_or("");
                    let start_line = chunk["start_line"].as_u64().unwrap_or(1) as u32;
                    let end_line = chunk["end_line"].as_u64().unwrap_or(1) as u32;

                    conn.execute(
                        "INSERT OR REPLACE INTO brain_chunks 
                        (chunk_id, source_id, path, content_type, start_line, end_line, text, signals, tags)
                        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
                        params![
                            chunk_id,
                            source_id,
                            title, // Use title as path for core entries
                            record_type,
                            start_line,
                            end_line,
                            format!("{}\n\n{}", summary, text), // Combine summary and text for search
                            signals_str,
                            tags_str,
                        ],
                    )?;
                }
            }

            // Append to JSONL
            writeln!(jsonl_file, "{}", line)?;
            count += 1;
        }

        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_options_default() {
        let opts = ExportOptions::default();
        assert!(opts.include_source_ids);
    }
}
