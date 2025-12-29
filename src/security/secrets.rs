//! Secret Detection Module
//! Scans text for credentials, API keys, and sensitive data.
//! Inspired by Secretlint patterns.

use regex::Regex;
use std::sync::LazyLock;

/// A detected secret
#[derive(Debug, Clone)]
pub struct SecretMatch {
    /// Type of secret (e.g., "aws_key", "github_token")
    pub secret_type: String,
    /// The matched text (partially redacted)
    pub redacted: String,
    /// Line number (1-indexed)
    pub line: usize,
    /// Column position
    pub column: usize,
}

/// Secret patterns to detect
static PATTERNS: LazyLock<Vec<(&'static str, Regex)>> = LazyLock::new(|| {
    vec![
        // AWS
        ("aws_access_key", Regex::new(r"AKIA[0-9A-Z]{16}").unwrap()),
        ("aws_secret_key", Regex::new(r#"(?i)aws[_\-]?secret[_\-]?access[_\-]?key['\"]?\s*[:=]\s*['\"]?([A-Za-z0-9/+=]{40})"#).unwrap()),
        // GitHub
        ("github_token", Regex::new(r"gh[pousr]_[A-Za-z0-9_]{36,}").unwrap()),
        ("github_pat", Regex::new(r"github_pat_[A-Za-z0-9_]{22,}").unwrap()),
        // Generic API Keys
        ("api_key", Regex::new(r#"(?i)(api[_\-]?key|apikey)['\"]?\s*[:=]\s*['\"]?([a-zA-Z0-9\-_]{20,})"#).unwrap()),
        ("secret_key", Regex::new(r#"(?i)(secret[_\-]?key|secretkey)['\"]?\s*[:=]\s*['\"]?([a-zA-Z0-9\-_]{20,})"#).unwrap()),
        // Passwords
        ("password", Regex::new(r#"(?i)(password|passwd|pwd)['\"]?\s*[:=]\s*['\"]?([^\s'"]{8,})"#).unwrap()),
        // Private Keys
        ("private_key", Regex::new(r"-----BEGIN (RSA |EC |OPENSSH )?PRIVATE KEY-----").unwrap()),
        // JWT
        ("jwt", Regex::new(r"eyJ[A-Za-z0-9\-_=]+\.eyJ[A-Za-z0-9\-_=]+\.[A-Za-z0-9\-_.+/=]+").unwrap()),
        // Slack
        ("slack_token", Regex::new(r"xox[baprs]-[0-9]{10,13}-[0-9]{10,13}[a-zA-Z0-9-]*").unwrap()),
        // Database URLs
        ("database_url", Regex::new(r#"(?i)(postgres|mysql|mongodb|redis)://[^\s'"]+:[^\s'"]+@[^\s'"]+"#).unwrap()),
    ]
});

/// Scan text for secrets
pub fn scan_text(text: &str) -> Vec<SecretMatch> {
    let mut matches = Vec::new();

    for (line_num, line) in text.lines().enumerate() {
        for (secret_type, pattern) in PATTERNS.iter() {
            if let Some(m) = pattern.find(line) {
                matches.push(SecretMatch {
                    secret_type: secret_type.to_string(),
                    redacted: redact_match(m.as_str()),
                    line: line_num + 1,
                    column: m.start() + 1,
                });
            }
        }
    }

    matches
}

/// Redact a matched secret, showing only first/last few chars
fn redact_match(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();

    if len <= 8 {
        "*".repeat(len)
    } else {
        let prefix: String = chars[..4].iter().collect();
        let suffix: String = chars[len - 4..].iter().collect();
        format!("{}...{}", prefix, suffix)
    }
}

/// Check if text contains any secrets
pub fn has_secrets(text: &str) -> bool {
    for (_, pattern) in PATTERNS.iter() {
        if pattern.is_match(text) {
            return true;
        }
    }
    false
}

/// Redact all secrets in text, replacing with [REDACTED]
pub fn redact_secrets(text: &str) -> String {
    let mut result = text.to_string();

    for (_, pattern) in PATTERNS.iter() {
        result = pattern.replace_all(&result, "[REDACTED]").to_string();
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_aws_key() {
        let text = "aws_key = AKIAIOSFODNN7EXAMPLE";
        let matches = scan_text(text);
        assert!(!matches.is_empty());
        assert_eq!(matches[0].secret_type, "aws_access_key");
    }

    #[test]
    fn test_detect_github_token() {
        let text = "token: ghp_1234567890abcdefghijklmnopqrstuvwxyz";
        let matches = scan_text(text);
        assert!(!matches.is_empty());
        assert_eq!(matches[0].secret_type, "github_token");
    }

    #[test]
    fn test_detect_password() {
        let text = "password = 'mysecretpassword123'";
        let matches = scan_text(text);
        assert!(!matches.is_empty());
        assert_eq!(matches[0].secret_type, "password");
    }

    #[test]
    fn test_detect_private_key() {
        let text = "-----BEGIN RSA PRIVATE KEY-----";
        let matches = scan_text(text);
        assert!(!matches.is_empty());
        assert_eq!(matches[0].secret_type, "private_key");
    }

    #[test]
    fn test_redact_secrets() {
        let text = "my_key = AKIAIOSFODNN7EXAMPLE";
        let redacted = redact_secrets(text);
        assert!(redacted.contains("[REDACTED]"));
        assert!(!redacted.contains("AKIAIOSFODNN7EXAMPLE"));
    }

    #[test]
    fn test_no_false_positives() {
        let text = "This is normal text with no secrets";
        let matches = scan_text(text);
        assert!(matches.is_empty());
    }
}
