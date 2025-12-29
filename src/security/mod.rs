//! Path Security Module
//! Provides path validation and sanitization to prevent traversal attacks.

pub mod secrets;

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// Validate that a path is safe and doesn't escape the base directory.
/// Returns the canonicalized path if valid.
pub fn validate_path(base: &Path, user_path: &str) -> Result<PathBuf> {
    // Reject obvious traversal attempts
    if user_path.contains("..") {
        anyhow::bail!("Path traversal detected: '..' not allowed");
    }

    // Reject absolute paths when expecting relative
    let path = Path::new(user_path);
    if path.is_absolute() {
        anyhow::bail!("Absolute paths not allowed");
    }

    // Reject null bytes
    if user_path.contains('\0') {
        anyhow::bail!("Null bytes in path not allowed");
    }

    // Join and canonicalize
    let full_path = base.join(user_path);
    let canonical = full_path
        .canonicalize()
        .with_context(|| format!("Failed to resolve path: {}", user_path))?;

    // Ensure it's within the base directory
    let canonical_base = base
        .canonicalize()
        .with_context(|| "Failed to resolve base directory")?;

    if !canonical.starts_with(&canonical_base) {
        anyhow::bail!(
            "Path escapes base directory: {} is not under {}",
            canonical.display(),
            canonical_base.display()
        );
    }

    Ok(canonical)
}

/// Validate a filename (no path components)
pub fn validate_filename(filename: &str) -> Result<&str> {
    // Reject empty
    if filename.is_empty() {
        anyhow::bail!("Filename cannot be empty");
    }

    // Reject path separators
    if filename.contains('/') || filename.contains('\\') {
        anyhow::bail!("Filename cannot contain path separators");
    }

    // Reject null bytes
    if filename.contains('\0') {
        anyhow::bail!("Filename cannot contain null bytes");
    }

    // Reject . and ..
    if filename == "." || filename == ".." {
        anyhow::bail!("Filename cannot be '.' or '..'");
    }

    // Reject Windows reserved names
    let upper = filename.to_uppercase();
    let reserved = [
        "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
        "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
    ];
    for r in reserved {
        if upper == r || upper.starts_with(&format!("{}.", r)) {
            anyhow::bail!("Filename uses reserved name: {}", r);
        }
    }

    Ok(filename)
}

/// Sanitize a string for use as a filename
pub fn sanitize_filename(input: &str) -> String {
    input
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_' || *c == '.' || *c == ' ')
        .collect::<String>()
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_validate_path_traversal() {
        let base = tempdir().unwrap();
        assert!(validate_path(base.path(), "../etc/passwd").is_err());
        assert!(validate_path(base.path(), "foo/../../../etc").is_err());
    }

    #[test]
    fn test_validate_path_absolute() {
        let base = tempdir().unwrap();
        assert!(validate_path(base.path(), "/etc/passwd").is_err());
    }

    #[test]
    fn test_validate_path_null_byte() {
        let base = tempdir().unwrap();
        assert!(validate_path(base.path(), "file\0.txt").is_err());
    }

    #[test]
    fn test_validate_path_valid() {
        let base = tempdir().unwrap();
        let sub = base.path().join("subdir");
        fs::create_dir(&sub).unwrap();
        let file = sub.join("test.txt");
        fs::write(&file, "test").unwrap();

        let result = validate_path(base.path(), "subdir/test.txt");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_filename() {
        assert!(validate_filename("").is_err());
        assert!(validate_filename("..").is_err());
        assert!(validate_filename("foo/bar").is_err());
        assert!(validate_filename("CON").is_err());
        assert!(validate_filename("valid.txt").is_ok());
    }

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("hello<>world"), "helloworld");
        assert_eq!(sanitize_filename("file.txt"), "file.txt");
        assert_eq!(sanitize_filename("my-file_v2.rs"), "my-file_v2.rs");
    }
}
