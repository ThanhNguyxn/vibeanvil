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

/// Result of compact operation
pub struct CompactResult {
    pub records_written: usize,
    pub chunks_count: usize,
}

/// Statistics from core import
#[derive(Debug, Default)]
pub struct ImportStats {
    pub total_lines: usize,
    pub parsed: usize,
    pub inserted: usize,
    pub skipped_errors: usize,
    pub skipped_duplicates: usize,
    pub was_upgrade: bool,
    /// Line numbers that had parse errors (for --verbose)
    pub error_lines: Vec<usize>,
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

    /// Create new brain storage for testing
    #[cfg(test)]
    pub fn new_for_test(path: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(&path)?;
        let jsonl_path = path.join("brainpack.jsonl");
        let sqlite_path = path.join("brainpack.sqlite");

        let storage = Self {
            brainpack_dir: path,
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

        // MIGRATION: Add summary, language, license columns if missing
        // We use PRAGMA table_info to check for columns reliably, avoiding "duplicate column" errors
        // if a previous check failed due to locking but the column was actually added.
        let mut existing_columns = std::collections::HashSet::new();
        let mut stmt = conn.prepare("PRAGMA table_info(brain_chunks)")?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(1))?;
        for row in rows {
            existing_columns.insert(row?);
        }

        if !existing_columns.contains("summary") {
            // Ignore error if column exists (race condition safety)
            let _ = conn.execute(
                "ALTER TABLE brain_chunks ADD COLUMN summary TEXT DEFAULT ''",
                [],
            );
        }
        if !existing_columns.contains("language") {
            let _ = conn.execute(
                "ALTER TABLE brain_chunks ADD COLUMN language TEXT DEFAULT 'unknown'",
                [],
            );
        }
        if !existing_columns.contains("license") {
            let _ = conn.execute(
                "ALTER TABLE brain_chunks ADD COLUMN license TEXT DEFAULT 'unknown'",
                [],
            );
        }

        // MIGRATION: Fix content_type quotes
        // Update any content_type that starts/ends with quotes (JSON string artifact)
        conn.execute(
            "UPDATE brain_chunks 
             SET content_type = TRIM(content_type, '\"') 
             WHERE content_type LIKE '\"%\"'",
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

        // Metadata table for tracking versions, hashes, etc.
        conn.execute(
            "CREATE TABLE IF NOT EXISTS metadata (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )",
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
                    (chunk_id, source_id, path, content_type, start_line, end_line, text, signals, tags, summary, language, license)
                    VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                    params![
                        chunk.chunk_id,
                        record.source_id,
                        record.path,
                        record.content_type.to_string(), // Use Display trait (no quotes)
                        chunk.start_line,
                        chunk.end_line,
                        chunk.text,
                        signals_json,
                        tags_str,
                        record.summary,
                        record.language,
                        record.license,
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

    /// Get metadata value by key
    pub fn get_meta(&self, key: &str) -> Option<String> {
        if !self.sqlite_path.exists() {
            return None;
        }

        let conn = Connection::open(&self.sqlite_path).ok()?;
        conn.query_row(
            "SELECT value FROM metadata WHERE key = ?",
            params![key],
            |row| row.get(0),
        )
        .ok()
    }

    /// Set metadata value
    pub fn set_meta(&self, key: &str, value: &str) -> Result<()> {
        let conn = Connection::open(&self.sqlite_path)?;
        conn.execute(
            "INSERT OR REPLACE INTO metadata (key, value) VALUES (?, ?)",
            params![key, value],
        )?;
        Ok(())
    }

    /// Get the commit/fingerprint for a source
    pub fn get_source_commit(&self, source_id: &str) -> Option<String> {
        if !self.sqlite_path.exists() {
            return None;
        }

        let conn = Connection::open(&self.sqlite_path).ok()?;
        conn.query_row(
            "SELECT \"commit\" FROM sources WHERE source_id = ?",
            params![source_id],
            |row| row.get(0),
        )
        .ok()
    }

    /// Delete all data for a source (chunks + source row)
    pub fn delete_source(&self, source_id: &str) -> Result<usize> {
        let conn = Connection::open(&self.sqlite_path)?;

        // Delete chunks first (foreign key)
        let chunks_deleted: usize = conn.execute(
            "DELETE FROM brain_chunks WHERE source_id = ?",
            params![source_id],
        )?;

        // Delete source row
        conn.execute(
            "DELETE FROM sources WHERE source_id = ?",
            params![source_id],
        )?;

        Ok(chunks_deleted)
    }

    /// Get chunk count for a specific source
    pub fn get_source_chunk_count(&self, source_id: &str) -> usize {
        if !self.sqlite_path.exists() {
            return 0;
        }

        let conn = match Connection::open(&self.sqlite_path) {
            Ok(c) => c,
            Err(_) => return 0,
        };

        conn.query_row(
            "SELECT COUNT(*) FROM brain_chunks WHERE source_id = ?",
            params![source_id],
            |row| row.get::<_, i64>(0),
        )
        .unwrap_or(0) as usize
    }

    /// Generate fingerprint for embedded core.jsonl
    /// Format: embedded@{version}@{line_count}
    pub fn core_fingerprint() -> String {
        const CORE_JSONL: &str = include_str!("../../brainpacks/core/core.jsonl");
        let line_count = CORE_JSONL.lines().filter(|l| !l.trim().is_empty()).count();
        format!("embedded@{}@{}", env!("CARGO_PKG_VERSION"), line_count)
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
        let mut output = std::fs::File::create(output_path)?;
        let conn = Connection::open(&self.sqlite_path)?;

        // Query all chunks ordered by source and path to group them
        let mut stmt = conn.prepare(
            "SELECT source_id, path, content_type, summary, language, license, 
                    chunk_id, start_line, end_line, text, signals, tags
             FROM brain_chunks 
             ORDER BY source_id, path, start_line",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,  // source_id
                row.get::<_, String>(1)?,  // path
                row.get::<_, String>(2)?,  // content_type
                row.get::<_, String>(3)?,  // summary
                row.get::<_, String>(4)?,  // language
                row.get::<_, String>(5)?,  // license
                row.get::<_, String>(6)?,  // chunk_id
                row.get::<_, u32>(7)?,     // start_line
                row.get::<_, u32>(8)?,     // end_line
                row.get::<_, String>(9)?,  // text
                row.get::<_, String>(10)?, // signals
                row.get::<_, String>(11)?, // tags
            ))
        })?;

        let mut current_record: Option<BrainRecord> = None;

        for row in rows {
            let (
                source_id,
                path,
                content_type_str,
                summary,
                language,
                license,
                chunk_id,
                start_line,
                end_line,
                text,
                signals_json,
                tags_str,
            ) = row?;

            // Check if we need to start a new record
            let is_new_record = match &current_record {
                Some(r) => r.source_id != source_id || r.path != path,
                None => true,
            };

            if is_new_record {
                // Write previous record if exists
                if let Some(mut record) = current_record.take() {
                    // Remove source_id if not requested
                    if !options.include_source_ids {
                        record.source_id = String::new();
                    }
                    writeln!(output, "{}", serde_json::to_string(&record)?)?;
                }

                // Parse content type (handle legacy quoted strings if any remain)
                let clean_type = content_type_str.trim_matches('"');
                let content_type = crate::brain::ContentType::from_path(clean_type); // Best effort mapping

                // Parse signals and tags
                let signals: Vec<crate::brain::Signal> =
                    serde_json::from_str(&signals_json).unwrap_or_default();
                let tags: Vec<String> = tags_str
                    .split(',')
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string())
                    .collect();

                // Start new record
                current_record = Some(BrainRecord {
                    source_id: source_id.clone(),
                    commit: "export".to_string(), // We don't store commit per chunk, could join sources table
                    license,
                    language,
                    path,
                    content_type,
                    signals,
                    summary,
                    chunks: Vec::new(),
                    tags,
                });
            }

            // Add chunk to current record
            if let Some(record) = &mut current_record {
                record.chunks.push(crate::brain::ContentChunk {
                    chunk_id,
                    text,
                    start_line,
                    end_line,
                });
            }
        }

        // Write last record
        if let Some(mut record) = current_record {
            if !options.include_source_ids {
                record.source_id = String::new();
            }
            writeln!(output, "{}", serde_json::to_string(&record)?)?;
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

        let conn = Connection::open(&self.sqlite_path)?;

        // Query chunks grouped by source and path
        let mut stmt = conn.prepare(
            "SELECT source_id, path, content_type, summary, language, license, text, signals
             FROM brain_chunks 
             GROUP BY source_id, path 
             ORDER BY source_id, path
             LIMIT 50", // Limit for markdown export safety
        )?;

        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?, // source_id
                row.get::<_, String>(1)?, // path
                row.get::<_, String>(2)?, // content_type
                row.get::<_, String>(3)?, // summary
                row.get::<_, String>(4)?, // language
                row.get::<_, String>(5)?, // license
                row.get::<_, String>(6)?, // text (first chunk due to GROUP BY)
                row.get::<_, String>(7)?, // signals
            ))
        })?;

        let mut current_source = String::new();

        for row in rows {
            let (source_id, path, content_type, summary, language, license, text, signals_json) =
                row?;

            if options.include_source_ids && source_id != current_source {
                current_source = source_id.clone();
                content.push_str(&format!("\n## Source: {}\n\n", current_source));
            }

            content.push_str(&format!("### {}\n\n", path));
            content.push_str(&format!(
                "**Type**: {} | **Language**: {} | **License**: {}\n\n",
                content_type.trim_matches('"'),
                language,
                license
            ));

            let signals: Vec<crate::brain::Signal> =
                serde_json::from_str(&signals_json).unwrap_or_default();
            if !signals.is_empty() {
                content.push_str("**Signals**: ");
                content.push_str(
                    &signals
                        .iter()
                        .map(|s| format!("{:?}", s))
                        .collect::<Vec<_>>()
                        .join(", "),
                );
                content.push_str("\n\n");
            }

            content.push_str(&format!("{}\n\n", summary));

            // Show first chunk
            content.push_str("```\n");
            content.push_str(&text[..text.len().min(300)]);
            if text.len() > 300 {
                content.push_str("\n... (truncated)");
            }
            content.push_str("\n```\n\n");
        }

        tokio::fs::write(output_path, &content).await?;
        Ok(output_path.to_string_lossy().to_string())
    }

    /// Import the core brainpack from embedded data
    /// If force is true, always re-import. Otherwise, only import if fingerprint changed.
    pub async fn import_core(&self, force: bool) -> Result<ImportStats> {
        // Embedded core brainpack JSONL
        const CORE_JSONL: &str = include_str!("../../brainpacks/core/core.jsonl");

        let mut stats = ImportStats::default();
        let desired_fingerprint = Self::core_fingerprint();

        // Check if we need to update using fingerprint in sources.commit
        let stored_fingerprint = self.get_source_commit("core");
        if !force {
            if let Some(ref fp) = stored_fingerprint {
                if fp == &desired_fingerprint {
                    // Already up-to-date
                    return Ok(stats);
                }
            }
        }

        // If we have old core data, delete it first
        if stored_fingerprint.is_some() || self.has_core_installed() {
            stats.was_upgrade = true;
            self.delete_source("core")?;
        }

        let conn = Connection::open(&self.sqlite_path)?;

        // Insert the "core" source record with fingerprint as commit
        conn.execute(
            "INSERT OR REPLACE INTO sources (source_id, \"commit\", license, language, fetched_at, files_count, chunks_count)
             VALUES ('core', ?, 'MIT', 'mixed', datetime('now'), 0, 0)",
            params![desired_fingerprint],
        )?;

        // Also append to JSONL file
        let mut jsonl_file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.jsonl_path)
            .context("Failed to open JSONL file")?;

        for (line_idx, line) in CORE_JSONL.lines().enumerate() {
            let line_num = line_idx + 1; // 1-indexed for user display
            if line.trim().is_empty() {
                continue;
            }

            // Parse the core record format (gracefully handle errors)
            let record: serde_json::Value = match serde_json::from_str(line) {
                Ok(r) => r,
                Err(_) => {
                    stats.skipped_errors += 1;
                    stats.error_lines.push(line_num);
                    continue;
                }
            };

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
                        (chunk_id, source_id, path, content_type, start_line, end_line, text, signals, tags, summary, language, license)
                        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                        params![
                            chunk_id,
                            source_id,
                            title, // Use title as path for core entries
                            record_type,
                            start_line,
                            end_line,
                            text,
                            signals_str,
                            tags_str,
                            summary,
                            "mixed", // Core brainpack language
                            "MIT",   // Core brainpack license
                        ],
                    )?;
                    stats.inserted += 1;
                }
            }

            // Append to JSONL
            writeln!(jsonl_file, "{}", line)?;
            stats.parsed += 1;
        }

        stats.total_lines = CORE_JSONL.lines().filter(|l| !l.trim().is_empty()).count();

        Ok(stats)
    }

    /// Compact the brain pack: rewrite JSONL from SQLite (dedup), run VACUUM
    pub async fn compact(&self) -> Result<CompactResult> {
        use std::io::Write;

        let conn = Connection::open(&self.sqlite_path)?;

        // Query all distinct records from SQLite, group by source_id + path
        let mut stmt = conn.prepare(
            "SELECT source_id, path, content_type, summary, language, license, 
                    chunk_id, start_line, end_line, text, signals, tags
             FROM brain_chunks 
             ORDER BY source_id, path, start_line",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,  // source_id
                row.get::<_, String>(1)?,  // path
                row.get::<_, String>(2)?,  // content_type
                row.get::<_, String>(3)?,  // summary
                row.get::<_, String>(4)?,  // language
                row.get::<_, String>(5)?,  // license
                row.get::<_, String>(6)?,  // chunk_id
                row.get::<_, u32>(7)?,     // start_line
                row.get::<_, u32>(8)?,     // end_line
                row.get::<_, String>(9)?,  // text
                row.get::<_, String>(10)?, // signals
                row.get::<_, String>(11)?, // tags
            ))
        })?;

        // Collect all rows
        let mut all_rows = Vec::new();
        for row in rows {
            all_rows.push(row?);
        }

        // Rewrite JSONL file from scratch
        let mut output = std::fs::File::create(&self.jsonl_path)?;
        let mut current_record: Option<BrainRecord> = None;
        let mut records_written = 0;
        let chunks_count = all_rows.len();

        for (
            source_id,
            path,
            content_type_str,
            summary,
            language,
            license,
            chunk_id,
            start_line,
            end_line,
            text,
            signals_json,
            tags_str,
        ) in all_rows
        {
            // Check if we need to start a new record
            let is_new_record = match &current_record {
                Some(r) => r.source_id != source_id || r.path != path,
                None => true,
            };

            if is_new_record {
                // Write previous record if exists
                if let Some(record) = current_record.take() {
                    writeln!(output, "{}", serde_json::to_string(&record)?)?;
                    records_written += 1;
                }

                // Parse content type
                let clean_type = content_type_str.trim_matches('"');
                let content_type = crate::brain::ContentType::from_path(clean_type);

                // Parse signals and tags
                let signals: Vec<crate::brain::Signal> =
                    serde_json::from_str(&signals_json).unwrap_or_default();
                let tags: Vec<String> = tags_str
                    .split(',')
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string())
                    .collect();

                // Start new record
                current_record = Some(BrainRecord {
                    source_id: source_id.clone(),
                    commit: "compact".to_string(),
                    license,
                    language,
                    path,
                    content_type,
                    signals,
                    summary,
                    chunks: Vec::new(),
                    tags,
                });
            }

            // Add chunk to current record
            if let Some(record) = &mut current_record {
                record.chunks.push(crate::brain::ContentChunk {
                    chunk_id,
                    text,
                    start_line,
                    end_line,
                });
            }
        }

        // Write last record
        if let Some(record) = current_record {
            writeln!(output, "{}", serde_json::to_string(&record)?)?;
            records_written += 1;
        }

        // Run VACUUM on SQLite
        conn.execute("VACUUM", [])?;

        Ok(CompactResult {
            records_written,
            chunks_count,
        })
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
