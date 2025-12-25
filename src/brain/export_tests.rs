use super::storage::{BrainStorage, ExportFormat, ExportOptions};
use super::{BrainRecord, ContentType, Signal};
use anyhow::Result;
use tempfile::TempDir;

#[tokio::test]
async fn test_canonical_export_deduplication() -> Result<()> {
    // Setup temp workspace
    let temp_dir = TempDir::new()?;
    let brain_dir = temp_dir.path().join("brain");
    let storage = BrainStorage::new_for_test(brain_dir.clone())?;

    // Create a record
    let record = BrainRecord {
        source_id: "src_test".to_string(),
        commit: "abc".to_string(),
        license: "MIT".to_string(),
        language: "Rust".to_string(),
        path: "test.rs".to_string(),
        content_type: ContentType::Code,
        signals: vec![Signal::CommandSurface],
        summary: "Test summary".to_string(),
        chunks: vec![crate::brain::ContentChunk {
            chunk_id: "chunk1".to_string(),
            text: "fn main() {}".to_string(),
            start_line: 1,
            end_line: 1,
        }],
        tags: vec!["test".to_string()],
    };

    // Manually insert source to satisfy FK
    let conn = rusqlite::Connection::open(brain_dir.join("brainpack.sqlite"))?;
    conn.execute(
        "INSERT INTO sources (source_id, \"commit\", license, language, fetched_at, files_count, chunks_count)
         VALUES ('src_test', 'abc', 'MIT', 'Rust', '2023-01-01T00:00:00Z', 1, 1)",
        [],
    )?;

    // Save record twice (simulating repeated harvest)
    println!("Saving record 1...");
    storage
        .save_records(&[record.clone()])
        .await
        .expect("Failed to save record 1");
    println!("Saving record 2...");
    storage
        .save_records(&[record.clone()])
        .await
        .expect("Failed to save record 2");

    // Export JSONL
    let export_path = temp_dir.path().join("export.jsonl");
    let options = ExportOptions {
        format: ExportFormat::Jsonl,
        output_path: Some(export_path.clone()),
        include_source_ids: true,
    };
    println!("Exporting to {:?}", export_path);
    storage.export(&options).await.expect("Failed to export");
    println!("Export done");

    // Verify export has only 1 record (deduplicated)
    let content = std::fs::read_to_string(&export_path)?;
    let lines: Vec<&str> = content.lines().collect();
    assert_eq!(lines.len(), 1, "Export should be deduplicated");

    let exported_record: BrainRecord = serde_json::from_str(lines[0])?;
    assert_eq!(exported_record.source_id, "src_test");
    assert_eq!(exported_record.chunks.len(), 1);

    Ok(())
}

#[tokio::test]
async fn test_content_type_migration() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let brain_dir = temp_dir.path().join("brain");
    let storage = BrainStorage::new_for_test(brain_dir.clone())?;

    // Manually insert a record with quoted content_type (simulating old data)
    let conn = rusqlite::Connection::open(brain_dir.join("brainpack.sqlite"))?;

    conn.execute("INSERT INTO sources (source_id) VALUES ('src_old')", [])?;

    conn.execute(
        "INSERT INTO brain_chunks (chunk_id, source_id, path, content_type, start_line, end_line, text)
         VALUES ('c1', 'src_old', 'old.rs', '\"code\"', 1, 1, 'text')",
        [],
    )?;

    // Re-init storage to trigger migration
    // We can't easily call init_db again on the same struct since it's private,
    // but creating a new storage instance on the same path will call init_db.
    let _storage2 = BrainStorage::new_for_test(brain_dir.clone())?;

    // Check if quotes were trimmed
    let content_type: String = conn.query_row(
        "SELECT content_type FROM brain_chunks WHERE chunk_id = 'c1'",
        [],
        |row| row.get(0),
    )?;

    assert_eq!(
        content_type, "code",
        "Quotes should be trimmed by migration"
    );

    Ok(())
}
