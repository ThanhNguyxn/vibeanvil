//! GitHub repo harvester with privacy-first approach
//!
//! - No URLs stored in exports
//! - Anonymized source IDs
//! - Secret redaction during extraction
//! - Configurable filters and limits

use anyhow::{Context, Result};
use regex::Regex;
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashSet;
use std::io::Read;
use std::path::PathBuf;

use super::{anonymize_source, BrainRecord, ContentChunk, ContentType, Signal, SourceMeta};
use crate::workspace;

/// GitHub repository info (internal only, not exported)
#[derive(Debug, Clone, Deserialize)]
pub struct RepoInfo {
    pub id: u64,
    pub name: String,
    pub full_name: String,
    pub description: Option<String>,
    pub stargazers_count: u32,
    pub language: Option<String>,
    pub license: Option<LicenseInfo>,
    pub default_branch: String,
    pub pushed_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LicenseInfo {
    pub key: String,
    pub spdx_id: Option<String>,
}

/// GitHub search response
#[derive(Debug, Deserialize)]
pub struct SearchResponse {
    pub total_count: u32,
    pub items: Vec<RepoInfo>,
}

/// Harvest configuration
#[derive(Debug, Clone)]
pub struct HarvestConfig {
    /// Search queries
    pub queries: Vec<String>,
    /// Topics
    pub topics: Vec<String>,
    /// Language filter
    pub language: Option<String>,
    /// Maximum repos to harvest
    pub max_repos: usize,
    /// Minimum stars
    pub min_stars: u32,
    /// Only repos updated within N days
    pub updated_within_days: u32,
    /// Download method
    pub download_method: DownloadMethod,
    /// Cache directory
    pub cache_dir: PathBuf,
    /// Ignore patterns (globs)
    pub ignore_globs: Vec<String>,
    /// Allow patterns (globs)
    pub allow_globs: Vec<String>,
    /// Maximum file size in bytes
    pub max_file_size: u64,
}

#[derive(Debug, Clone, Default)]
pub enum DownloadMethod {
    #[default]
    Tarball,
    Git,
}

impl Default for HarvestConfig {
    fn default() -> Self {
        Self {
            queries: vec![],
            topics: vec![],
            language: None,
            max_repos: 20,
            min_stars: 10,
            updated_within_days: 365,
            download_method: DownloadMethod::Tarball,
            cache_dir: workspace::cache_dir(),
            ignore_globs: vec![
                "node_modules/**".to_string(),
                "vendor/**".to_string(),
                "target/**".to_string(),
                ".git/**".to_string(),
                "**/*.min.js".to_string(),
                "**/*.map".to_string(),
            ],
            allow_globs: vec![],
            max_file_size: 100_000, // 100KB default
        }
    }
}

/// Secret patterns for redaction during harvesting
const SECRET_PATTERNS: &[&str] = &[
    r#"(?i)(api[_-]?key\s*[=:]\s*)(['"]?)[\w\-]{20,}['"]?"#,
    r#"(?i)(secret[_-]?key\s*[=:]\s*)(['"]?)[\w\-]{20,}['"]?"#,
    r#"(?i)(password\s*[=:]\s*)(['"]?)[\w\-!@#$%^&*]{8,}['"]?"#,
    r#"(?i)(token\s*[=:]\s*)(['"]?)[\w\-]{20,}['"]?"#,
    r"(?i)(bearer\s+)[\w\-\.]+",
    r"ghp_[a-zA-Z0-9]{36}",
    r"gho_[a-zA-Z0-9]{36}",
    r"sk-[a-zA-Z0-9]{48}",
    r"AKIA[0-9A-Z]{16}",
    r"-----BEGIN[\s\w]+PRIVATE KEY-----",
];

/// Redact secrets from content
fn redact_secrets(content: &str) -> String {
    let mut result = content.to_string();
    for pattern in SECRET_PATTERNS {
        if let Ok(re) = Regex::new(pattern) {
            result = re.replace_all(&result, "[REDACTED]").to_string();
        }
    }
    result
}

/// Detect signals in content
fn detect_signals(content: &str, path: &str) -> Vec<Signal> {
    let mut signals = vec![];
    let lower = content.to_lowercase();
    let path_lower = path.to_lowercase();

    // Command surface
    if lower.contains("clap")
        || lower.contains("argparse")
        || lower.contains("commander")
        || lower.contains("subcommand")
        || lower.contains("cli")
    {
        signals.push(Signal::CommandSurface);
    }

    // State machine
    if lower.contains("state machine")
        || lower.contains("statemachine")
        || lower.contains("transition")
        || lower.contains("enum state")
        || (lower.contains("state") && lower.contains("next_state"))
    {
        signals.push(Signal::StateMachine);
    }

    // Contract lock
    if lower.contains("contract") && (lower.contains("lock") || lower.contains("hash"))
        || lower.contains("spec_hash")
        || lower.contains("immutable")
    {
        signals.push(Signal::ContractLock);
    }

    // Iterate loop
    if (lower.contains("iterate") || lower.contains("loop"))
        && (lower.contains("test") || lower.contains("lint") || lower.contains("fix"))
    {
        signals.push(Signal::IterateLoop);
    }

    // Evidence/audit
    if lower.contains("evidence")
        || lower.contains("audit")
        || lower.contains("jsonl")
        || lower.contains("trail")
    {
        signals.push(Signal::EvidenceAudit);
    }

    // Provider adapter
    if lower.contains("provider") && (lower.contains("trait") || lower.contains("interface"))
        || lower.contains("adapter")
        || lower.contains("plugin")
    {
        signals.push(Signal::ProviderAdapter);
    }

    // Security pattern
    if lower.contains("redact")
        || lower.contains("secret")
        || path_lower.contains("security")
        || lower.contains("sanitize")
    {
        signals.push(Signal::SecurityPattern);
    }

    signals
}

/// Generate summary from content
fn generate_summary(content: &str, content_type: &ContentType) -> String {
    let first_lines: Vec<&str> = content.lines().take(5).collect();
    let preview = first_lines.join(" ").chars().take(150).collect::<String>();

    match content_type {
        ContentType::Readme => format!("README: {}", preview),
        ContentType::Doc => format!("Documentation: {}", preview),
        ContentType::Config => "Configuration file".to_string(),
        ContentType::Code => format!("Source code: {}", preview),
        ContentType::Workflow => "CI/CD workflow configuration".to_string(),
        ContentType::Template => "Template file".to_string(),
        ContentType::Prompt => "Prompt/rules file".to_string(),
        ContentType::Other => format!("File: {}", preview),
    }
}

/// Split content into chunks
fn chunk_content(content: &str, chunk_size: usize) -> Vec<ContentChunk> {
    let lines: Vec<&str> = content.lines().collect();
    let mut chunks = vec![];
    let mut start_line = 1u32;

    for chunk in lines.chunks(chunk_size) {
        let text = chunk.join("\n");
        let end_line = start_line + chunk.len() as u32 - 1;

        // Skip empty chunks
        if text.trim().is_empty() {
            start_line = end_line + 1;
            continue;
        }

        // Truncate text for storage (max 500 chars per chunk)
        let truncated = if text.len() > 500 {
            format!("{}...", &text[..500])
        } else {
            text
        };

        chunks.push(ContentChunk {
            chunk_id: format!("chunk_{}_{}", start_line, end_line),
            text: truncated,
            start_line,
            end_line,
        });

        start_line = end_line + 1;
    }

    chunks
}

/// Harvester for GitHub repos
pub struct Harvester {
    client: Client,
    config: HarvestConfig,
    processed_sources: HashSet<String>,
}

impl Harvester {
    /// Create new harvester with config
    pub async fn new(config: HarvestConfig) -> Result<Self> {
        tokio::fs::create_dir_all(&config.cache_dir).await?;

        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("Accept", "application/vnd.github+json".parse()?);
        headers.insert("X-GitHub-Api-Version", "2022-11-28".parse()?);

        // Use GITHUB_TOKEN if available
        if let Ok(token) = std::env::var("GITHUB_TOKEN") {
            headers.insert("Authorization", format!("Bearer {}", token).parse()?);
        } else {
            tracing::warn!("GITHUB_TOKEN not set. API rate limits will be restricted.");
        }

        let client = Client::builder()
            .user_agent("vibeanvil/0.1.0")
            .default_headers(headers)
            .build()?;

        Ok(Self {
            client,
            config,
            processed_sources: HashSet::new(),
        })
    }

    /// Search GitHub for repositories
    pub async fn search_repos(&self) -> Result<Vec<RepoInfo>> {
        let mut all_repos = vec![];

        // Build search query
        let mut query_parts = vec![];

        for q in &self.config.queries {
            query_parts.push(q.clone());
        }

        for topic in &self.config.topics {
            query_parts.push(format!("topic:{}", topic));
        }

        if let Some(lang) = &self.config.language {
            query_parts.push(format!("language:{}", lang));
        }

        query_parts.push(format!("stars:>={}", self.config.min_stars));

        // Add date filter
        let cutoff =
            chrono::Utc::now() - chrono::Duration::days(self.config.updated_within_days as i64);
        query_parts.push(format!("pushed:>{}", cutoff.format("%Y-%m-%d")));

        let query = query_parts.join(" ");

        let url = format!(
            "https://api.github.com/search/repositories?q={}&sort=stars&order=desc&per_page={}",
            urlencoding::encode(&query),
            self.config.max_repos.min(100)
        );

        tracing::info!("Searching GitHub: {}", query);

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("GitHub API error {}: {}", status, body);
        }

        let search_result: SearchResponse = response
            .json()
            .await
            .context("Failed to parse GitHub search response")?;

        all_repos.extend(search_result.items.into_iter().take(self.config.max_repos));

        Ok(all_repos)
    }

    /// Check if source is already cached
    fn is_cached(&self, source_id: &str, commit: &str) -> bool {
        let cache_path = self.config.cache_dir.join(format!(
            "{}_{}.done",
            source_id,
            &commit[..8.min(commit.len())]
        ));
        cache_path.exists()
    }

    /// Mark source as cached
    async fn mark_cached(&self, source_id: &str, commit: &str) -> Result<()> {
        let cache_path = self.config.cache_dir.join(format!(
            "{}_{}.done",
            source_id,
            &commit[..8.min(commit.len())]
        ));
        tokio::fs::write(&cache_path, chrono::Utc::now().to_rfc3339()).await?;
        Ok(())
    }

    /// Harvest a single repository
    pub async fn harvest_repo(
        &mut self,
        repo: &RepoInfo,
    ) -> Result<(SourceMeta, Vec<BrainRecord>)> {
        let source_id = anonymize_source(&repo.full_name);
        let commit = repo.default_branch.clone(); // Ideally fetch HEAD SHA

        // Check cache
        if self.is_cached(&source_id, &commit) {
            tracing::info!("  ↩ Cached: {}", source_id);
            return Ok((
                SourceMeta {
                    source_id: source_id.clone(),
                    commit,
                    license: "cached".to_string(),
                    language: "cached".to_string(),
                    fetched_at: chrono::Utc::now(),
                    files_count: 0,
                    chunks_count: 0,
                    stars: repo.stargazers_count,
                },
                vec![],
            ));
        }

        println!("  → Harvesting: {} (★{})", source_id, repo.stargazers_count);

        // Download tarball
        let tarball_url = format!(
            "https://api.github.com/repos/{}/tarball/{}",
            repo.full_name, repo.default_branch
        );

        let response = self.client.get(&tarball_url).send().await?;

        if !response.status().is_success() {
            tracing::warn!("Failed to download tarball: {}", response.status());
            return Ok((
                SourceMeta {
                    source_id: source_id.clone(),
                    commit,
                    license: "error".to_string(),
                    language: "error".to_string(),
                    fetched_at: chrono::Utc::now(),
                    files_count: 0,
                    chunks_count: 0,
                    stars: repo.stargazers_count,
                },
                vec![],
            ));
        }

        let bytes = response.bytes().await?;

        // Process tarball
        let records = self.process_tarball(&bytes, &source_id, &commit, repo)?;

        let license = repo
            .license
            .as_ref()
            .and_then(|l| l.spdx_id.clone())
            .unwrap_or_else(|| "unknown".to_string());

        let language = repo
            .language
            .clone()
            .unwrap_or_else(|| "unknown".to_string());

        let source_meta = SourceMeta {
            source_id: source_id.clone(),
            commit: commit.clone(),
            license,
            language,
            fetched_at: chrono::Utc::now(),
            files_count: records.len(),
            chunks_count: records.iter().map(|r| r.chunks.len()).sum(),
            stars: repo.stargazers_count,
        };

        // Mark as cached
        self.mark_cached(&source_id, &commit).await?;
        self.processed_sources.insert(source_id);

        Ok((source_meta, records))
    }

    /// Process tarball and extract records
    fn process_tarball(
        &self,
        bytes: &[u8],
        source_id: &str,
        commit: &str,
        repo: &RepoInfo,
    ) -> Result<Vec<BrainRecord>> {
        let mut records = vec![];

        let decoder = flate2::read::GzDecoder::new(bytes);
        let mut archive = tar::Archive::new(decoder);

        let license = repo
            .license
            .as_ref()
            .and_then(|l| l.spdx_id.clone())
            .unwrap_or_else(|| "unknown".to_string());

        let language = repo
            .language
            .clone()
            .unwrap_or_else(|| "unknown".to_string());

        for entry in archive.entries()? {
            let mut entry = entry?;
            let path = entry.path()?.to_string_lossy().to_string();

            // Skip based on size
            if entry.size() > self.config.max_file_size {
                continue;
            }

            // Apply ignore/allow globs
            if !self.should_process_file(&path) {
                continue;
            }

            let mut content = String::new();
            if entry.read_to_string(&mut content).is_err() {
                continue; // Skip binary files
            }

            // Redact secrets
            let safe_content = redact_secrets(&content);

            // Determine content type
            let content_type = ContentType::from_path(&path);

            // Extract relative path
            let file_path = path.split('/').skip(1).collect::<Vec<_>>().join("/");

            // Detect signals
            let signals = detect_signals(&safe_content, &file_path);

            // Generate summary
            let summary = generate_summary(&safe_content, &content_type);

            // Chunk content
            let chunks = chunk_content(&safe_content, 50);

            // Generate tags
            let mut tags = vec![];
            if !signals.is_empty() {
                tags.extend(signals.iter().map(|s| format!("{:?}", s).to_lowercase()));
            }
            tags.push(format!("lang:{}", language.to_lowercase()));

            let record = BrainRecord {
                source_id: source_id.to_string(),
                commit: commit.to_string(),
                license: license.clone(),
                language: language.clone(),
                path: file_path,
                content_type,
                signals,
                summary,
                chunks,
                tags,
            };

            records.push(record);
        }

        Ok(records)
    }

    /// Check if file should be processed
    fn should_process_file(&self, path: &str) -> bool {
        let lower = path.to_lowercase();

        // Default ignores
        let default_ignores = [
            "node_modules/",
            "vendor/",
            "target/",
            ".git/",
            "__pycache__/",
            ".min.",
            ".map",
            ".lock",
            "package-lock.json",
            "yarn.lock",
            "Cargo.lock",
        ];

        for pattern in default_ignores {
            if lower.contains(pattern) {
                return false;
            }
        }

        // Check custom ignore globs
        for pattern in &self.config.ignore_globs {
            if glob::Pattern::new(pattern)
                .map(|p| p.matches(&lower))
                .unwrap_or(false)
            {
                return false;
            }
        }

        // If allow globs specified, file must match at least one
        if !self.config.allow_globs.is_empty() {
            let matches_allow = self.config.allow_globs.iter().any(|pattern| {
                glob::Pattern::new(pattern)
                    .map(|p| p.matches(&lower))
                    .unwrap_or(false)
            });
            if !matches_allow {
                return false;
            }
        }

        // Include common interesting files
        let extensions = [
            ".rs", ".py", ".js", ".ts", ".go", ".md", ".toml", ".yaml", ".yml", ".json",
        ];

        for ext in extensions {
            if lower.ends_with(ext) {
                return true;
            }
        }

        // Include specific filenames
        let filenames = [
            "readme",
            "claude",
            "prompt",
            "rules",
            "contributing",
            "security",
        ];
        for name in filenames {
            if lower.contains(name) {
                return true;
            }
        }

        false
    }

    /// Get count of processed sources
    pub fn processed_count(&self) -> usize {
        self.processed_sources.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anonymize_source() {
        let id1 = anonymize_source("owner/repo");
        let id2 = anonymize_source("owner/repo");
        let id3 = anonymize_source("other/repo");

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
        assert!(id1.starts_with("src_"));
    }

    #[test]
    fn test_detect_signals() {
        let content = "impl StateMachine { fn transition(&mut self) {} }";
        let signals = detect_signals(content, "src/state.rs");
        assert!(signals.contains(&Signal::StateMachine));
    }

    #[test]
    fn test_redact_secrets() {
        let content = "token = ghp_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx";
        let redacted = redact_secrets(content);
        assert!(redacted.contains("[REDACTED]"));
        assert!(!redacted.contains("ghp_"));
    }
}
