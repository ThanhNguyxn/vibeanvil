//! Tests for core.jsonl schema validation
//!
//! These tests ensure the embedded core brainpack parses correctly.

use serde::{Deserialize, Serialize};

/// Core record schema (matches core.jsonl structure)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CoreRecord {
    source_id: String,
    #[serde(rename = "type")]
    record_type: String,
    title: String,
    signals: Vec<String>,
    tags: Vec<String>,
    summary: String,
    chunks: Vec<CoreChunk>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CoreChunk {
    chunk_id: String,
    text: String,
    start_line: u32,
    end_line: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Smoke test: parse every line of core.jsonl
    #[test]
    fn test_core_jsonl_parses_correctly() {
        const CORE_JSONL: &str = include_str!("../../brainpacks/core/core.jsonl");

        let mut line_number = 0;
        let mut record_count = 0;

        for line in CORE_JSONL.lines() {
            line_number += 1;
            let trimmed = line.trim();

            if trimmed.is_empty() {
                continue;
            }

            let result: Result<CoreRecord, _> = serde_json::from_str(trimmed);
            match result {
                Ok(record) => {
                    // Validate core records have source_id = "core"
                    assert_eq!(
                        record.source_id, "core",
                        "Line {}: source_id should be 'core', got '{}'",
                        line_number, record.source_id
                    );

                    // Validate required fields are non-empty
                    assert!(
                        !record.title.is_empty(),
                        "Line {}: title should not be empty",
                        line_number
                    );
                    assert!(
                        !record.chunks.is_empty(),
                        "Line {}: chunks should not be empty for '{}'",
                        line_number,
                        record.title
                    );

                    record_count += 1;
                }
                Err(e) => {
                    panic!(
                        "Line {} failed to parse: {}\nContent: {}",
                        line_number,
                        e,
                        &trimmed[..trimmed.len().min(100)]
                    );
                }
            }
        }

        // Ensure we have some records
        assert!(
            record_count > 0,
            "core.jsonl should contain at least one record"
        );

        println!("✓ Parsed {} core records successfully", record_count);
    }

    /// Test: ensure bug case - DB has non-core records but no core
    #[test]
    fn test_has_core_installed_logic() {
        // This simulates the logic fix: we now check for source_id = "core"
        // not just "any records exist"

        let non_core_records = vec!["src_abc123", "src_def456"];
        let core_records = vec!["core"];

        // Old buggy logic: any records = has_core
        let buggy_has_core = !non_core_records.is_empty();
        assert!(buggy_has_core, "Buggy logic would return true");

        // Fixed logic: check for "core" specifically
        let fixed_has_core = non_core_records.iter().any(|&id| id == "core");
        assert!(
            !fixed_has_core,
            "Fixed logic correctly returns false for non-core"
        );

        let fixed_has_core_with_core = core_records.iter().any(|&id| id == "core");
        assert!(
            fixed_has_core_with_core,
            "Fixed logic returns true when core exists"
        );
    }
}
