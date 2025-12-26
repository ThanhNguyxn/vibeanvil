//! Safety utilities for provider hardening
//!
//! - Secret redaction
//! - Output truncation
//! - Helpful error formatting

use regex::Regex;
use std::sync::LazyLock;

// ============================================================================
// Constants
// ============================================================================

/// Default timeout for command execution (10 minutes)
pub const DEFAULT_TIMEOUT_SECS: u64 = 600;

/// Maximum output size per stream (256KB)
pub const MAX_OUTPUT_BYTES: usize = 256 * 1024;

/// Maximum lines to show in error tail
pub const ERROR_TAIL_LINES: usize = 40;

/// Maximum bytes for error tail
pub const ERROR_TAIL_BYTES: usize = 8 * 1024;

/// Default patch limits
pub const DEFAULT_PATCH_MAX_FILES: usize = 50;
pub const DEFAULT_PATCH_MAX_ADDED_LINES: usize = 5000;
pub const DEFAULT_PATCH_MAX_FILE_ADDED_LINES: usize = 2000;
pub const DEFAULT_PATCH_MAX_BYTES: usize = 2 * 1024 * 1024; // 2MB
pub const MAX_LINE_LENGTH: usize = 20_000;

// ============================================================================
// Secret Redaction
// ============================================================================

/// Compiled regex patterns for secret detection
static SECRET_PATTERNS: LazyLock<Vec<(Regex, &'static str)>> = LazyLock::new(|| {
    vec![
        // GitHub Personal Access Token (classic)
        (
            Regex::new(r"ghp_[A-Za-z0-9]{20,}").unwrap(),
            "ghp_***REDACTED***",
        ),
        // GitHub Personal Access Token (fine-grained)
        (
            Regex::new(r"github_pat_[A-Za-z0-9_]{20,}").unwrap(),
            "github_pat_***REDACTED***",
        ),
        // GitHub OAuth Token
        (
            Regex::new(r"gho_[A-Za-z0-9]{20,}").unwrap(),
            "gho_***REDACTED***",
        ),
        // GitHub App Token
        (
            Regex::new(r"ghu_[A-Za-z0-9]{20,}").unwrap(),
            "ghu_***REDACTED***",
        ),
        // GitHub Refresh Token
        (
            Regex::new(r"ghr_[A-Za-z0-9]{20,}").unwrap(),
            "ghr_***REDACTED***",
        ),
        // OpenAI API Key
        (
            Regex::new(r"sk-[A-Za-z0-9]{20,}").unwrap(),
            "sk-***REDACTED***",
        ),
        // Anthropic API Key
        (
            Regex::new(r"sk-ant-[A-Za-z0-9\-]{20,}").unwrap(),
            "sk-ant-***REDACTED***",
        ),
        // AWS Access Key
        (
            Regex::new(r"AKIA[0-9A-Z]{16}").unwrap(),
            "AKIA***REDACTED***",
        ),
        // Bearer token in headers
        (
            Regex::new(r"(?i)Bearer\s+[A-Za-z0-9\-_.~+/]+=*").unwrap(),
            "Bearer ***REDACTED***",
        ),
        // Authorization header
        (
            Regex::new(r"(?i)Authorization:\s*[^\r\n]+").unwrap(),
            "Authorization: ***REDACTED***",
        ),
        // Generic API key patterns
        (
            Regex::new(r"(?i)(api[_-]?key|apikey)\s*[=:]\s*[A-Za-z0-9_\-]{20,}").unwrap(),
            "api_key=***REDACTED***",
        ),
        // Generic secret patterns
        (
            Regex::new(r"(?i)(secret|password|passwd|token)\s*[=:]\s*[A-Za-z0-9_\-]{8,}").unwrap(),
            "secret=***REDACTED***",
        ),
    ]
});

/// Redact secrets from text
///
/// Replaces known token patterns with redacted versions.
/// The prefix is preserved for debugging (e.g., "ghp_***REDACTED***").
///
/// # Example
/// ```ignore
/// let input = "Token: ghp_abc123def456ghi789jkl0";
/// let output = redact_secrets(input);
/// assert!(output.contains("ghp_***REDACTED***"));
/// ```
pub fn redact_secrets(text: &str) -> String {
    let mut result = text.to_string();
    for (pattern, replacement) in SECRET_PATTERNS.iter() {
        result = pattern.replace_all(&result, *replacement).to_string();
    }
    result
}

// ============================================================================
// Output Truncation
// ============================================================================

/// Truncate output to a maximum size
///
/// If the text exceeds `max_bytes`, it is truncated and a marker is appended.
///
/// # Example
/// ```ignore
/// let long_text = "a".repeat(300_000);
/// let truncated = truncate_output(&long_text, 256 * 1024);
/// assert!(truncated.ends_with("[truncated]"));
/// ```
pub fn truncate_output(text: &str, max_bytes: usize) -> String {
    if text.len() <= max_bytes {
        return text.to_string();
    }

    // Find a safe UTF-8 boundary
    let mut end = max_bytes;
    while end > 0 && !text.is_char_boundary(end) {
        end -= 1;
    }

    format!(
        "{}\n\n[truncated - output exceeded {} bytes]",
        &text[..end],
        max_bytes
    )
}

/// Get the last N lines of text (for error display)
///
/// Returns up to `max_lines` lines from the end of the text,
/// also respecting `max_bytes` limit.
pub fn tail_lines(text: &str, max_lines: usize, max_bytes: usize) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let start = lines.len().saturating_sub(max_lines);
    let tail: String = lines[start..].join("\n");

    if tail.len() > max_bytes {
        truncate_output(&tail, max_bytes)
    } else if start > 0 {
        format!("[... {} lines omitted ...]\n{}", start, tail)
    } else {
        tail
    }
}

// ============================================================================
// Patch Validation Helpers
// ============================================================================

/// Check if a path is an absolute Windows path
pub fn is_windows_absolute(path: &str) -> bool {
    // Drive letter: C:\, D:\, etc.
    if path.len() >= 3 {
        let bytes = path.as_bytes();
        if bytes[0].is_ascii_alphabetic()
            && bytes[1] == b':'
            && (bytes[2] == b'\\' || bytes[2] == b'/')
        {
            return true;
        }
    }
    // UNC path: \\server\share
    if path.starts_with("\\\\") || path.starts_with("//") {
        return true;
    }
    false
}

/// Check if a path contains forbidden patterns
pub fn is_forbidden_path(path: &str) -> Option<&'static str> {
    let normalized = path.replace('\\', "/").to_lowercase();

    // Check for .git directory
    if normalized.starts_with(".git/") || normalized.contains("/.git/") || normalized == ".git" {
        return Some("Modifying .git/ directory is not allowed");
    }

    // Check for path traversal
    if normalized.contains("..") {
        return Some("Path traversal (..) is not allowed");
    }

    // Check for Unix absolute path
    if path.starts_with('/') {
        return Some("Absolute Unix paths are not allowed");
    }

    // Check for Windows absolute path
    if is_windows_absolute(path) {
        return Some("Absolute Windows paths are not allowed");
    }

    None
}

/// Check if content looks like binary
pub fn is_binary_content(content: &str) -> bool {
    // Check for explicit binary patch marker
    if content.contains("GIT binary patch") {
        return true;
    }

    // Check for very long lines (likely minified/binary)
    for line in content.lines() {
        if line.len() > MAX_LINE_LENGTH {
            return true;
        }
    }

    false
}

/// Parse patch statistics (files changed, lines added/removed)
#[derive(Debug)]
pub struct PatchStats {
    pub files_changed: usize,
    pub lines_added: usize,
    pub lines_removed: usize,
    pub max_file_lines_added: usize,
}

impl PatchStats {
    pub fn from_diff(content: &str) -> Self {
        let mut files_changed = 0;
        let mut lines_added = 0;
        let mut lines_removed = 0;
        let mut current_file_lines_added = 0;
        let mut max_file_lines_added = 0;

        for line in content.lines() {
            if line.starts_with("+++ ") {
                // New file in diff
                if current_file_lines_added > max_file_lines_added {
                    max_file_lines_added = current_file_lines_added;
                }
                current_file_lines_added = 0;
                files_changed += 1;
            } else if line.starts_with('+') && !line.starts_with("+++") {
                lines_added += 1;
                current_file_lines_added += 1;
            } else if line.starts_with('-') && !line.starts_with("---") {
                lines_removed += 1;
            }
        }

        // Check last file
        if current_file_lines_added > max_file_lines_added {
            max_file_lines_added = current_file_lines_added;
        }

        Self {
            files_changed,
            lines_added,
            lines_removed,
            max_file_lines_added,
        }
    }
}

// ============================================================================
// Environment Variable Helpers
// ============================================================================

/// Get timeout from environment variable
/// Supports both VIBEANVIL_PROVIDER_TIMEOUT_SECS (preferred) and
/// VIBEANVIL_PROVIDER_TIMEOUT (legacy) for backward compatibility
pub fn get_timeout_secs() -> u64 {
    // Prefer new name, fallback to legacy
    std::env::var("VIBEANVIL_PROVIDER_TIMEOUT_SECS")
        .ok()
        .or_else(|| std::env::var("VIBEANVIL_PROVIDER_TIMEOUT").ok())
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_TIMEOUT_SECS)
}

/// Get patch limits from environment variables
pub fn get_patch_max_files() -> usize {
    std::env::var("VIBEANVIL_PATCH_MAX_FILES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_PATCH_MAX_FILES)
}

pub fn get_patch_max_added_lines() -> usize {
    std::env::var("VIBEANVIL_PATCH_MAX_ADDED_LINES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_PATCH_MAX_ADDED_LINES)
}

pub fn get_patch_max_file_added_lines() -> usize {
    std::env::var("VIBEANVIL_PATCH_MAX_FILE_ADDED_LINES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_PATCH_MAX_FILE_ADDED_LINES)
}

pub fn get_patch_max_bytes() -> usize {
    std::env::var("VIBEANVIL_PATCH_MAX_BYTES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_PATCH_MAX_BYTES)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ------------------------------------------------------------------------
    // Redaction Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_redact_github_pat() {
        let input = "Token: ghp_abc123def456ghi789jkl012345678901234567";
        let output = redact_secrets(input);
        assert!(output.contains("ghp_***REDACTED***"));
        assert!(!output.contains("abc123"));
    }

    #[test]
    fn test_redact_github_pat_fine_grained() {
        let input = "github_pat_abcdefghijklmnopqrstuvwxyz123456789012345678901234567890";
        let output = redact_secrets(input);
        assert!(output.contains("github_pat_***REDACTED***"));
    }

    #[test]
    fn test_redact_openai_key() {
        let input = "OPENAI_API_KEY=sk-abcdefghijklmnopqrstuvwxyz1234567890";
        let output = redact_secrets(input);
        assert!(output.contains("sk-***REDACTED***"));
    }

    #[test]
    fn test_redact_bearer_token() {
        let input = "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.payload.signature";
        let output = redact_secrets(input);
        assert!(
            output.contains("Bearer ***REDACTED***")
                || output.contains("Authorization: ***REDACTED***")
        );
    }

    #[test]
    fn test_redact_preserves_normal_text() {
        let input =
            "This is normal text with no secrets. The word 'token' appears but no actual token.";
        let output = redact_secrets(input);
        // Should not over-redact
        assert!(output.contains("normal text"));
        assert!(output.contains("word 'token'"));
    }

    #[test]
    fn test_redact_multiple_secrets() {
        let input = "ghp_secret1234567890123456789012 and sk-anothersecret123456789012345678";
        let output = redact_secrets(input);
        assert!(output.contains("ghp_***REDACTED***"));
        assert!(output.contains("sk-***REDACTED***"));
    }

    // ------------------------------------------------------------------------
    // Truncation Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_truncate_short_text() {
        let input = "Short text";
        let output = truncate_output(input, 1024);
        assert_eq!(output, input);
    }

    #[test]
    fn test_truncate_long_text() {
        let input = "a".repeat(1000);
        let output = truncate_output(&input, 100);
        assert!(output.len() < 200); // truncated + marker
        assert!(output.contains("[truncated"));
    }

    #[test]
    fn test_tail_lines() {
        let input = (1..=100)
            .map(|i| format!("Line {}", i))
            .collect::<Vec<_>>()
            .join("\n");
        let output = tail_lines(&input, 10, 8192);
        assert!(output.contains("Line 100"));
        assert!(output.contains("Line 91"));
        assert!(output.contains("[... 90 lines omitted ...]"));
    }

    // ------------------------------------------------------------------------
    // Path Validation Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_windows_absolute_drive() {
        assert!(is_windows_absolute("C:\\Users\\test"));
        assert!(is_windows_absolute("D:/path/to/file"));
    }

    #[test]
    fn test_windows_absolute_unc() {
        assert!(is_windows_absolute("\\\\server\\share\\file"));
        assert!(is_windows_absolute("//server/share/file"));
    }

    #[test]
    fn test_not_windows_absolute() {
        assert!(!is_windows_absolute("relative/path"));
        assert!(!is_windows_absolute("./local/path"));
    }

    #[test]
    fn test_forbidden_git_path() {
        assert!(is_forbidden_path(".git/config").is_some());
        assert!(is_forbidden_path("src/.git/hooks").is_some());
        assert!(is_forbidden_path(".git").is_some());
    }

    #[test]
    fn test_forbidden_traversal() {
        assert!(is_forbidden_path("../etc/passwd").is_some());
        assert!(is_forbidden_path("foo/../../bar").is_some());
    }

    #[test]
    fn test_forbidden_absolute_unix() {
        assert!(is_forbidden_path("/etc/passwd").is_some());
    }

    #[test]
    fn test_forbidden_absolute_windows() {
        assert!(is_forbidden_path("C:\\Windows\\System32").is_some());
    }

    #[test]
    fn test_allowed_relative_path() {
        assert!(is_forbidden_path("src/main.rs").is_none());
        assert!(is_forbidden_path("tests/integration.rs").is_none());
    }

    // ------------------------------------------------------------------------
    // Binary Detection Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_binary_git_patch() {
        let content = "diff --git a/file.bin b/file.bin\nGIT binary patch\nliteral 1234";
        assert!(is_binary_content(content));
    }

    #[test]
    fn test_binary_long_line() {
        let long_line = "a".repeat(25_000);
        let content = format!("normal line\n{}\nanother line", long_line);
        assert!(is_binary_content(&content));
    }

    #[test]
    fn test_not_binary() {
        let content = "normal diff content\n+added line\n-removed line";
        assert!(!is_binary_content(content));
    }

    // ------------------------------------------------------------------------
    // Patch Stats Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_patch_stats() {
        let diff = r#"
--- a/file1.rs
+++ b/file1.rs
@@ -1,3 +1,5 @@
 line1
+added1
+added2
 line2
-removed1
--- a/file2.rs
+++ b/file2.rs
@@ -1,2 +1,3 @@
 existing
+new line
"#;
        let stats = PatchStats::from_diff(diff);
        assert_eq!(stats.files_changed, 2);
        assert_eq!(stats.lines_added, 3);
        assert_eq!(stats.lines_removed, 1);
        assert_eq!(stats.max_file_lines_added, 2);
    }

    // ------------------------------------------------------------------------
    // Env Var Backwards Compatibility Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_timeout_env_var_new_name() {
        std::env::set_var("VIBEANVIL_PROVIDER_TIMEOUT_SECS", "120");
        std::env::remove_var("VIBEANVIL_PROVIDER_TIMEOUT");
        assert_eq!(get_timeout_secs(), 120);
        std::env::remove_var("VIBEANVIL_PROVIDER_TIMEOUT_SECS");
    }

    #[test]
    fn test_timeout_env_var_legacy_fallback() {
        std::env::remove_var("VIBEANVIL_PROVIDER_TIMEOUT_SECS");
        std::env::set_var("VIBEANVIL_PROVIDER_TIMEOUT", "300");
        assert_eq!(get_timeout_secs(), 300);
        std::env::remove_var("VIBEANVIL_PROVIDER_TIMEOUT");
    }

    #[test]
    fn test_timeout_env_var_new_takes_precedence() {
        std::env::set_var("VIBEANVIL_PROVIDER_TIMEOUT_SECS", "60");
        std::env::set_var("VIBEANVIL_PROVIDER_TIMEOUT", "999");
        assert_eq!(get_timeout_secs(), 60);
        std::env::remove_var("VIBEANVIL_PROVIDER_TIMEOUT_SECS");
        std::env::remove_var("VIBEANVIL_PROVIDER_TIMEOUT");
    }

    #[test]
    fn test_timeout_env_var_default() {
        std::env::remove_var("VIBEANVIL_PROVIDER_TIMEOUT_SECS");
        std::env::remove_var("VIBEANVIL_PROVIDER_TIMEOUT");
        assert_eq!(get_timeout_secs(), DEFAULT_TIMEOUT_SECS);
    }
}
