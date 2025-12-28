//! BrainPack - dynamic repo harvesting and knowledge indexing
//!
//! This module implements privacy-first repo harvesting:
//! - No external URLs stored in exports by default
//! - Anonymized source IDs using SHA-256 hashing
//! - User-driven search queries (no hardcoded repos)

pub mod harvester;
pub mod map;
pub mod pack;
pub mod presets;
pub mod storage;

#[cfg(test)]
mod core_jsonl_tests;
#[cfg(test)]
mod export_tests;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Generate anonymized source ID from repo full name
pub fn anonymize_source(repo_full_name: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(repo_full_name.as_bytes());
    let result = hasher.finalize();
    format!("src_{}", hex::encode(&result[..12]))
}

/// Signal types detected in code
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Signal {
    CommandSurface,
    StateMachine,
    ContractLock,
    IterateLoop,
    EvidenceAudit,
    ProviderAdapter,
    SecurityPattern,
}

/// Content type categories
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ContentType {
    Readme,
    Doc,
    Config,
    Code,
    Workflow,
    Template,
    Prompt,
    Other,
}

impl ContentType {
    /// Determine content type from file path
    pub fn from_path(path: &str) -> Self {
        let lower = path.to_lowercase();

        if lower.contains("readme") {
            ContentType::Readme
        } else if lower.contains("docs/") || (lower.ends_with(".md") && !lower.contains("readme")) {
            ContentType::Doc
        } else if lower.contains(".github/workflows") {
            ContentType::Workflow
        } else if lower.contains("claude") || lower.contains("prompt") || lower.contains("rules") {
            ContentType::Prompt
        } else if lower.contains("template") {
            ContentType::Template
        } else if lower.ends_with(".toml")
            || lower.ends_with(".yaml")
            || lower.ends_with(".yml")
            || lower.ends_with(".json")
            || lower.contains("config")
        {
            ContentType::Config
        } else if lower.ends_with(".rs")
            || lower.ends_with(".py")
            || lower.ends_with(".js")
            || lower.ends_with(".ts")
            || lower.ends_with(".go")
        {
            ContentType::Code
        } else {
            ContentType::Other
        }
    }

    /// Parse content type from database string (case-insensitive)
    /// Maps known type names directly, falls back to Other for unknown values
    pub fn from_db_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "readme" => ContentType::Readme,
            "doc" => ContentType::Doc,
            "config" => ContentType::Config,
            "code" => ContentType::Code,
            "workflow" => ContentType::Workflow,
            "template" => ContentType::Template,
            "prompt" => ContentType::Prompt,
            _ => ContentType::Other,
        }
    }
}

impl std::fmt::Display for ContentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContentType::Readme => write!(f, "readme"),
            ContentType::Doc => write!(f, "doc"),
            ContentType::Config => write!(f, "config"),
            ContentType::Code => write!(f, "code"),
            ContentType::Workflow => write!(f, "workflow"),
            ContentType::Template => write!(f, "template"),
            ContentType::Prompt => write!(f, "prompt"),
            ContentType::Other => write!(f, "other"),
        }
    }
}

/// A chunk of content from a file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentChunk {
    /// Unique chunk ID
    pub chunk_id: String,
    /// Text content (short snippet)
    pub text: String,
    /// Start line in original file
    pub start_line: u32,
    /// End line in original file
    pub end_line: u32,
}

/// A harvested brain record (privacy-first schema)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrainRecord {
    /// Anonymized source ID (hash, NOT URL)
    pub source_id: String,
    /// Commit SHA or opaque reference
    pub commit: String,
    /// SPDX license or "unknown"
    pub license: String,
    /// Detected language or "unknown"
    pub language: String,
    /// File path within repo
    pub path: String,
    /// Content type
    #[serde(rename = "type")]
    pub content_type: ContentType,
    /// Detected signals
    pub signals: Vec<Signal>,
    /// Short summary
    pub summary: String,
    /// Content chunks
    pub chunks: Vec<ContentChunk>,
    /// Tags
    pub tags: Vec<String>,
}

/// Source metadata (stored locally, not exported by default)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceMeta {
    /// Anonymized source ID
    pub source_id: String,
    /// Commit SHA
    pub commit: String,
    /// License
    pub license: String,
    /// Primary language
    pub language: String,
    /// When fetched
    pub fetched_at: chrono::DateTime<chrono::Utc>,
    /// Total files processed
    pub files_count: usize,
    /// Total chunks extracted
    pub chunks_count: usize,
    /// Stars (for ranking, not exported)
    #[serde(skip_serializing)]
    pub stars: u32,
}

/// BrainPack statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BrainStats {
    /// Total records
    pub total_records: usize,
    /// Total sources
    pub total_sources: usize,
    /// Total chunks
    pub total_chunks: usize,
    /// JSONL file size
    pub jsonl_size_bytes: u64,
    /// SQLite database size
    pub sqlite_size_bytes: u64,
    /// Last updated
    pub last_updated: Option<chrono::DateTime<chrono::Utc>>,
    /// By content type
    pub by_type: std::collections::HashMap<String, usize>,
    /// By language
    pub by_language: std::collections::HashMap<String, usize>,
    /// By license
    pub by_license: std::collections::HashMap<String, usize>,
}

/// Search result from brain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Chunk ID
    pub chunk_id: String,
    /// Source ID (anonymized)
    pub source_id: String,
    /// File path
    pub path: String,
    /// Content type
    pub content_type: String,
    /// Short snippet (never full file)
    pub snippet: String,
    /// Relevance score
    pub score: f64,
    /// Tags
    pub tags: Vec<String>,
}
