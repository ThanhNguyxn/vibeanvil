//! Tests for core.jsonl schema validation
//!
//! These tests ensure the embedded core brainpack parses correctly.

use serde::Deserialize;

/// Core record schema (matches core.jsonl structure)
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
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

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
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

        let non_core_records: &[&str] = &["src_abc123", "src_def456"];
        let core_records: &[&str] = &["core"];

        // Old buggy logic: any records = has_core
        let buggy_has_core = !non_core_records.is_empty();
        assert!(buggy_has_core, "Buggy logic would return true");

        // Fixed logic: check for "core" specifically
        let fixed_has_core = non_core_records.contains(&"core");
        assert!(
            !fixed_has_core,
            "Fixed logic correctly returns false for non-core"
        );

        let fixed_has_core_with_core = core_records.contains(&"core");
        assert!(
            fixed_has_core_with_core,
            "Fixed logic returns true when core exists"
        );
    }

    /// Smoke test: brain ensure imports core and search returns results
    #[tokio::test]
    async fn test_brain_ensure_and_search() {
        use crate::brain::storage::BrainStorage;
        use tempfile::TempDir;

        // Create temp directory for test DB
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let storage = BrainStorage::new_for_test(temp_dir.path().to_path_buf())
            .expect("Failed to create storage");

        // Import core (force = true to ensure fresh import)
        let stats = storage
            .import_core(true)
            .await
            .expect("import_core should succeed");

        // Verify import worked
        assert!(
            stats.inserted > 0,
            "Should have inserted some records, got {}",
            stats.inserted
        );
        assert!(
            stats.total_lines > 0,
            "Should have parsed some lines, got {}",
            stats.total_lines
        );

        // Verify search works
        let results = storage
            .search("acceptance criteria", 10)
            .expect("search should succeed");

        assert!(
            !results.is_empty(),
            "Search for 'acceptance criteria' should return at least 1 result"
        );

        // Verify first result has expected properties
        let first = &results[0];
        assert_eq!(first.source_id, "core", "Result should be from core");
        assert!(!first.snippet.is_empty(), "Snippet should not be empty");
    }

    /// Test: fingerprint mismatch triggers core refresh
    #[tokio::test]
    async fn test_fingerprint_mismatch_refreshes_core() {
        use crate::brain::storage::BrainStorage;
        use rusqlite::Connection;
        use tempfile::TempDir;

        // Create temp directory for test DB
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let storage = BrainStorage::new_for_test(temp_dir.path().to_path_buf())
            .expect("Failed to create storage");

        // Import core initially
        let stats = storage
            .import_core(false)
            .await
            .expect("import_core should succeed");
        assert!(stats.inserted > 0, "First import should insert records");

        let initial_count = storage.get_source_chunk_count("core");
        assert!(initial_count > 0, "Should have core chunks");

        // Manually update sources.commit to simulate fingerprint mismatch
        let db_path = temp_dir.path().join("brainpack.sqlite");
        {
            let conn = Connection::open(&db_path).expect("Failed to open DB");
            conn.execute(
                "UPDATE sources SET \"commit\" = 'old_fingerprint' WHERE source_id = 'core'",
                [],
            )
            .expect("Failed to update fingerprint");
        }

        // Verify fingerprint was changed
        let stored = storage.get_source_commit("core");
        assert_eq!(stored, Some("old_fingerprint".to_string()));

        // Run import_core again (without force) - should detect mismatch and refresh
        let refresh_stats = storage
            .import_core(false)
            .await
            .expect("refresh import should succeed");

        // Should have refreshed because fingerprint didn't match
        assert!(
            refresh_stats.was_upgrade,
            "Should detect upgrade due to fingerprint mismatch"
        );
        assert!(
            refresh_stats.inserted > 0,
            "Should re-insert records after refresh"
        );

        // Verify fingerprint is now correct
        let new_fingerprint = storage.get_source_commit("core");
        let expected_fingerprint = BrainStorage::core_fingerprint();
        assert_eq!(
            new_fingerprint,
            Some(expected_fingerprint.clone()),
            "Fingerprint should be updated to current"
        );

        // Verify chunk count matches current embedded CORE_JSONL
        let final_count = storage.get_source_chunk_count("core");
        assert_eq!(
            final_count, initial_count,
            "Chunk count should match initial import"
        );
    }
}
