//! Risk Classifier - Analyze diffs to determine A/B/C risk level

use super::RiskLevel;
use regex::Regex;
use std::path::Path;

/// Classification result with risk level and reasoning
#[derive(Debug, Clone)]
pub struct ClassificationResult {
    /// Determined risk level
    pub risk: RiskLevel,
    /// Reasons for this classification
    pub reasons: Vec<String>,
    /// Files touched by the change
    pub touched_files: Vec<String>,
    /// Whether public API surface is affected
    pub public_surface_changes: bool,
}

/// Risk classifier for analyzing diffs
pub struct RiskClassifier {
    public_api_patterns: Vec<Regex>,
    security_patterns: Vec<Regex>,
    config_patterns: Vec<Regex>,
}

impl Default for RiskClassifier {
    fn default() -> Self {
        Self::new()
    }
}

impl RiskClassifier {
    /// Create a new classifier with default heuristics
    pub fn new() -> Self {
        Self {
            public_api_patterns: vec![
                Regex::new(r"^\+\s*pub\s+(fn|struct|enum|trait|type|const|static)\s+").unwrap(),
                Regex::new(r"^\-\s*pub\s+(fn|struct|enum|trait|type|const|static)\s+").unwrap(),
                Regex::new(r"^\+\s*export\s+(default\s+)?(function|class|const|let|var)\s+").unwrap(),
                Regex::new(r"^\+\s*module\.exports").unwrap(),
            ],
            security_patterns: vec![
                Regex::new(r"(?i)(auth|password|secret|token|key|credential)").unwrap(),
                Regex::new(r"(?i)(encrypt|decrypt|hash|sign|verify)").unwrap(),
                Regex::new(r"(?i)(permission|access|role|privilege)").unwrap(),
            ],
            config_patterns: vec![
                Regex::new(r"^\+\s*\[dependencies\]").unwrap(),
                Regex::new(r#"^\+\s*"dependencies""#).unwrap(),
                Regex::new(r#"^\+.*=\s*"[0-9]+\.[0-9]+"#).unwrap(),
            ],
        }
    }

    /// Classify a diff and return the risk level with reasons
    pub fn classify(&self, diff: &str) -> ClassificationResult {
        let mut reasons = Vec::new();
        let mut touched_files = Vec::new();
        let mut public_surface = false;
        let mut max_risk = RiskLevel::A;

        // Extract touched files from diff headers
        for line in diff.lines() {
            if line.starts_with("+++ b/") || line.starts_with("--- a/") {
                let path = line
                    .trim_start_matches("+++ b/")
                    .trim_start_matches("--- a/")
                    .trim_start_matches("+++ ")
                    .trim_start_matches("--- ");
                if path != "/dev/null" && !touched_files.contains(&path.to_string()) {
                    touched_files.push(path.to_string());
                }
            }
        }

        // Classify each touched file
        for file in &touched_files {
            let file_risk = self.classify_file(file, diff);
            if file_risk.0 as u8 > max_risk as u8 {
                max_risk = file_risk.0;
            }
            if !file_risk.1.is_empty() {
                reasons.extend(file_risk.1);
            }
            if file_risk.2 {
                public_surface = true;
            }
        }

        // Check for public API changes in diff content
        for pattern in &self.public_api_patterns {
            if pattern.is_match(diff) {
                public_surface = true;
                if (max_risk as u8) < (RiskLevel::C as u8) {
                    max_risk = RiskLevel::C;
                    reasons.push("Public API signature changed".to_string());
                }
                break;
            }
        }

        // Check for security-sensitive patterns
        for line in diff.lines() {
            if line.starts_with('+') || line.starts_with('-') {
                for pattern in &self.security_patterns {
                    if pattern.is_match(line) {
                        if (max_risk as u8) < (RiskLevel::C as u8) {
                            max_risk = RiskLevel::C;
                            reasons.push("Security-sensitive code modified".to_string());
                        }
                        break;
                    }
                }
            }
        }

        // Check for config/dependency changes
        for pattern in &self.config_patterns {
            if pattern.is_match(diff) {
                if (max_risk as u8) < (RiskLevel::C as u8) {
                    max_risk = RiskLevel::C;
                    reasons.push("Dependency or config change".to_string());
                }
                break;
            }
        }

        reasons.sort();
        reasons.dedup();

        ClassificationResult {
            risk: max_risk,
            reasons,
            touched_files,
            public_surface_changes: public_surface,
        }
    }

    fn classify_file(&self, path: &str, diff: &str) -> (RiskLevel, Vec<String>, bool) {
        let path = Path::new(path);
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let path_str = path.to_string_lossy().to_lowercase();

        let mut reasons = Vec::new();

        // Level A: Documentation files
        let level_a_exts = ["md", "txt", "rst", "adoc"];
        let level_a_files = ["readme", "changelog", "license", "contributing", "authors"];

        if level_a_exts.contains(&ext.to_lowercase().as_str())
            || level_a_files.iter().any(|f| filename.to_lowercase().starts_with(f))
        {
            reasons.push(format!("Documentation file: {}", filename));
            return (RiskLevel::A, reasons, false);
        }

        // Level C: Dependency manifests
        let level_c_files = ["cargo.toml", "package.json", "pyproject.toml", "go.mod"];
        if level_c_files.iter().any(|f| filename.to_lowercase() == *f) {
            reasons.push(format!("Dependency manifest: {}", filename));
            return (RiskLevel::C, reasons, true);
        }

        // Level C: CI/Build files
        if path_str.contains(".github/workflows") || filename.to_lowercase() == "dockerfile" {
            reasons.push(format!("CI/Build file: {}", filename));
            return (RiskLevel::C, reasons, false);
        }

        // Level C: Config files
        if filename.starts_with(".env") || filename.to_lowercase().starts_with("config.") {
            reasons.push(format!("Configuration file: {}", filename));
            return (RiskLevel::C, reasons, false);
        }

        // Level C: Security paths
        if path_str.contains("auth") || path_str.contains("security") {
            reasons.push(format!("Security-sensitive path: {}", path_str));
            return (RiskLevel::C, reasons, false);
        }

        // Level B: Test files
        if path_str.contains("test") || path_str.contains("spec") {
            reasons.push(format!("Test file: {}", filename));
            return (RiskLevel::B, reasons, false);
        }

        // Level B: Code files
        let code_exts = ["rs", "py", "js", "ts", "go", "java", "c", "cpp"];
        if code_exts.contains(&ext.to_lowercase().as_str()) {
            if self.has_public_changes_for_file(path, diff) {
                reasons.push(format!("Code with public API: {}", filename));
                return (RiskLevel::C, reasons, true);
            }
            reasons.push(format!("Code file: {}", filename));
            return (RiskLevel::B, reasons, false);
        }

        reasons.push(format!("Other file: {}", filename));
        (RiskLevel::A, reasons, false)
    }

    fn has_public_changes_for_file(&self, file_path: &Path, diff: &str) -> bool {
        let file_str = file_path.to_string_lossy();
        let mut in_file_section = false;

        for line in diff.lines() {
            if line.starts_with("+++ b/") || line.starts_with("+++ ") {
                let path = line.trim_start_matches("+++ b/").trim_start_matches("+++ ");
                in_file_section = path == file_str || path.ends_with(&*file_str);
            }
            if in_file_section {
                for pattern in &self.public_api_patterns {
                    if pattern.is_match(line) {
                        return true;
                    }
                }
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_docs() {
        let classifier = RiskClassifier::new();
        let diff = "--- a/README.md\n+++ b/README.md\n@@ -1 +1,2 @@\n # Project\n+Added docs";
        let result = classifier.classify(diff);
        assert_eq!(result.risk, RiskLevel::A);
    }

    #[test]
    fn test_classify_code() {
        let classifier = RiskClassifier::new();
        let diff = "--- a/src/main.rs\n+++ b/src/main.rs\n@@ -1 +1,2 @@\n fn main() {\n+    println!(\"Hi\");";
        let result = classifier.classify(diff);
        assert_eq!(result.risk, RiskLevel::B);
    }

    #[test]
    fn test_classify_public_api() {
        let classifier = RiskClassifier::new();
        let diff = "--- a/src/lib.rs\n+++ b/src/lib.rs\n@@ -1 +1,2 @@\n+pub fn new_api() {}";
        let result = classifier.classify(diff);
        assert_eq!(result.risk, RiskLevel::C);
    }

    #[test]
    fn test_classify_mixed_content() {
        let classifier = RiskClassifier::new();
        // Docs (A) + Code (B) -> Should be B
        let diff = "--- a/README.md\n+++ b/README.md\n@@ -1 +1 @@\n+Docs\n--- a/src/main.rs\n+++ b/src/main.rs\n@@ -1 +1 @@\n+fn code() {}";
        let result = classifier.classify(diff);
        assert_eq!(result.risk, RiskLevel::B);
        assert!(result.reasons.iter().any(|r| r.contains("Code file")));
    }

    #[test]
    fn test_classify_security_pattern() {
        let classifier = RiskClassifier::new();
        let diff = "--- a/src/utils.rs\n+++ b/src/utils.rs\n@@ -1 +1 @@\n+let api_key = \"secret\";";
        let result = classifier.classify(diff);
        assert_eq!(result.risk, RiskLevel::C);
        assert!(result.reasons.iter().any(|r| r.contains("Security-sensitive")));
    }

    #[test]
    fn test_classify_config_change() {
        let classifier = RiskClassifier::new();
        let diff = "--- a/Cargo.toml\n+++ b/Cargo.toml\n@@ -1 +1 @@\n+[dependencies]";
        let result = classifier.classify(diff);
        assert_eq!(result.risk, RiskLevel::C);
    }

    #[test]
    fn test_classify_public_api_deletion() {
        let classifier = RiskClassifier::new();
        let diff = "--- a/src/lib.rs\n+++ b/src/lib.rs\n@@ -1 +0 @@\n-pub fn old_api() {}";
        let result = classifier.classify(diff);
        assert_eq!(result.risk, RiskLevel::C);
        assert!(result.public_surface_changes);
    }
}
