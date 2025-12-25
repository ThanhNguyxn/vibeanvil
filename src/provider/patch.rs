//! Patch provider - safely applies unified diffs with validation
//!
//! Features:
//! - Path safety validation (no absolute paths, traversal, or .git/)
//! - Size/impact limits (configurable via env vars)
//! - Binary diff rejection
//! - Clear summary before applying

use anyhow::{Context as AnyhowContext, Result};
use async_trait::async_trait;
use colored::Colorize;
use std::path::{Path, PathBuf};
use tokio::fs;

use super::safety::{
    get_patch_max_added_lines, get_patch_max_bytes, get_patch_max_file_added_lines,
    get_patch_max_files, is_binary_content, is_forbidden_path, redact_secrets, PatchStats,
};
use super::{Context, Provider, ProviderResponse};

/// Patch provider for unified diff workflows
///
/// Configuration:
/// - `VIBEANVIL_PATCH_FILE`: Path to unified diff file to apply
///
/// Safety limits (env vars):
/// - `VIBEANVIL_PATCH_MAX_FILES`: Max files in patch (default: 50)
/// - `VIBEANVIL_PATCH_MAX_ADDED_LINES`: Max total added lines (default: 5000)
/// - `VIBEANVIL_PATCH_MAX_FILE_ADDED_LINES`: Max added lines per file (default: 2000)
/// - `VIBEANVIL_PATCH_MAX_BYTES`: Max patch file size (default: 2MB)
pub struct PatchProvider;

impl PatchProvider {
    pub fn new() -> Self {
        Self
    }

    /// Get the patch file path from env
    fn patch_file_path() -> Option<PathBuf> {
        std::env::var("VIBEANVIL_PATCH_FILE")
            .ok()
            .map(PathBuf::from)
    }

    /// Generate the prompt file path for a session
    fn prompt_file_path(context: &Context) -> PathBuf {
        context
            .working_dir
            .join(".vibeanvil")
            .join("sessions")
            .join(&context.session_id)
            .join("patch_prompt.md")
    }

    /// Validate a diff path is safe
    fn validate_path(path: &Path, _repo_root: &Path) -> Result<()> {
        let path_str = path.to_string_lossy();

        // Use shared forbidden path checker
        if let Some(reason) = is_forbidden_path(&path_str) {
            anyhow::bail!("{}: {}", reason, path.display());
        }

        Ok(())
    }

    /// Validate the entire patch content
    fn validate_patch(content: &str, repo_root: &Path) -> Result<PatchStats> {
        // Check file size
        let max_bytes = get_patch_max_bytes();
        if content.len() > max_bytes {
            anyhow::bail!(
                "Patch file too large: {} bytes (max: {} bytes).\n\n\
                 To increase the limit, set:\n  \
                 export VIBEANVIL_PATCH_MAX_BYTES=<bytes>",
                content.len(),
                max_bytes
            );
        }

        // Check for binary content
        if is_binary_content(content) {
            anyhow::bail!(
                "Binary patches are not allowed.\n\n\
                 The patch contains binary content markers or very long lines.\n\
                 Please use text-based unified diffs only."
            );
        }

        // Parse and validate paths
        let mut files_in_patch: Vec<String> = Vec::new();
        for line in content.lines() {
            if line.starts_with("+++ ") {
                let path_part = line.split_whitespace().nth(1).unwrap_or("");
                let clean_path = path_part
                    .strip_prefix("b/")
                    .or_else(|| path_part.strip_prefix("a/"))
                    .unwrap_or(path_part);

                if !clean_path.is_empty() && clean_path != "/dev/null" {
                    let path = PathBuf::from(clean_path);
                    Self::validate_path(&path, repo_root)?;
                    files_in_patch.push(clean_path.to_string());
                }
            }
        }

        // Calculate stats
        let stats = PatchStats::from_diff(content);

        // Check limits
        let max_files = get_patch_max_files();
        if stats.files_changed > max_files {
            anyhow::bail!(
                "Too many files in patch: {} (max: {}).\n\n\
                 To increase the limit, set:\n  \
                 export VIBEANVIL_PATCH_MAX_FILES=<count>",
                stats.files_changed,
                max_files
            );
        }

        let max_added = get_patch_max_added_lines();
        if stats.lines_added > max_added {
            anyhow::bail!(
                "Too many added lines: {} (max: {}).\n\n\
                 To increase the limit, set:\n  \
                 export VIBEANVIL_PATCH_MAX_ADDED_LINES=<count>",
                stats.lines_added,
                max_added
            );
        }

        let max_file_added = get_patch_max_file_added_lines();
        if stats.max_file_lines_added > max_file_added {
            anyhow::bail!(
                "Too many added lines in a single file: {} (max: {}).\n\n\
                 To increase the limit, set:\n  \
                 export VIBEANVIL_PATCH_MAX_FILE_ADDED_LINES=<count>",
                stats.max_file_lines_added,
                max_file_added
            );
        }

        Ok(stats)
    }

    /// Parse and validate unified diff content (legacy, for hunk extraction)
    fn parse_diff(content: &str, repo_root: &Path) -> Result<Vec<DiffHunk>> {
        let mut hunks = Vec::new();
        let mut current_file: Option<PathBuf> = None;
        let mut current_lines: Vec<String> = Vec::new();

        for line in content.lines() {
            if line.starts_with("--- ") || line.starts_with("+++ ") {
                let path_part = line.split_whitespace().nth(1).unwrap_or("");
                let clean_path = path_part
                    .strip_prefix("a/")
                    .or_else(|| path_part.strip_prefix("b/"))
                    .unwrap_or(path_part);

                if !clean_path.is_empty() && clean_path != "/dev/null" {
                    let path = PathBuf::from(clean_path);
                    Self::validate_path(&path, repo_root)?;

                    if line.starts_with("+++ ") {
                        current_file = Some(path);
                    }
                }
            } else if line.starts_with("@@") {
                if let Some(ref file) = current_file {
                    if !current_lines.is_empty() {
                        hunks.push(DiffHunk {
                            file: file.clone(),
                            lines: std::mem::take(&mut current_lines),
                        });
                    }
                }
            } else if current_file.is_some()
                && (line.starts_with('+') || line.starts_with('-') || line.starts_with(' '))
            {
                current_lines.push(line.to_string());
            }
        }

        if let Some(file) = current_file {
            if !current_lines.is_empty() {
                hunks.push(DiffHunk {
                    file,
                    lines: current_lines,
                });
            }
        }

        Ok(hunks)
    }

    /// Generate prompt file for patch workflow
    async fn generate_prompt_file(prompt: &str, context: &Context) -> Result<PathBuf> {
        let prompt_path = Self::prompt_file_path(context);

        if let Some(parent) = prompt_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let content = format!(
            r#"# VibeAnvil Patch Prompt

## ⚠️ Safety Instructions

- **Do NOT include secrets** (API keys, tokens, passwords) in the diff
- **Do NOT modify files outside the repository**
- **Do NOT use absolute paths** in the diff
- **Run tests after applying** to verify changes

## Instructions

Generate a **unified diff** for the following task. Output ONLY the diff, no explanations.

Example format:
```diff
--- a/src/file.rs
+++ b/src/file.rs
@@ -10,6 +10,7 @@
 existing line
+new line
 existing line
```

After generating the diff:
1. Save it to a file (e.g., `changes.patch`)
2. Set the environment variable:
   ```bash
   export VIBEANVIL_PATCH_FILE=changes.patch
   ```
3. Re-run the build:
   ```bash
   vibeanvil build iterate --provider patch
   ```

---

## Context

- **Session ID**: `{}`
- **Contract Hash**: `{}`

---

## Task

{}

---

## Output Format

Output a valid unified diff only. No markdown code blocks, no explanations.
"#,
            context.session_id,
            context.contract_hash.as_deref().unwrap_or("none"),
            prompt
        );

        fs::write(&prompt_path, &content).await?;
        Ok(prompt_path)
    }

    /// Print patch summary before applying
    fn print_summary(stats: &PatchStats) {
        println!();
        println!("{}", "📊 Patch Summary:".yellow().bold());
        println!(
            "   Files: {} | Added: {} lines | Removed: {} lines",
            stats.files_changed, stats.lines_added, stats.lines_removed
        );
        println!();
    }
}

impl Default for PatchProvider {
    fn default() -> Self {
        Self::new()
    }
}

struct DiffHunk {
    file: PathBuf,
    lines: Vec<String>,
}

#[async_trait]
impl Provider for PatchProvider {
    async fn execute(&self, prompt: &str, context: &Context) -> Result<ProviderResponse> {
        let patch_file = Self::patch_file_path();

        if patch_file.is_none() {
            let prompt_path = Self::generate_prompt_file(prompt, context).await?;

            println!();
            println!("{}", "═".repeat(60).cyan());
            println!(
                "{}",
                " 📋 Patch Provider - Unified Diff Workflow".cyan().bold()
            );
            println!("{}", "═".repeat(60).cyan());
            println!();
            println!("{}", "A prompt file has been generated.".white());
            println!();
            println!("{}", "📋 Steps:".yellow().bold());
            println!("  {} Open the prompt file:", "1.".cyan());
            println!("     {}", prompt_path.display().to_string().green());
            println!();
            println!("  {} Generate a unified diff using your AI", "2.".cyan());
            println!();
            println!("  {} Save the diff to a file and set:", "3.".cyan());
            println!(
                "     {}",
                "export VIBEANVIL_PATCH_FILE=your_changes.patch".green()
            );
            println!();
            println!("  {} Re-run the build:", "4.".cyan());
            println!(
                "     {}",
                "vibeanvil build iterate --provider patch".green()
            );
            println!();
            println!("{}", "═".repeat(60).cyan());

            return Ok(ProviderResponse {
                success: true,
                output: format!("Prompt file generated: {}", prompt_path.display()),
                errors: vec![],
                warnings: vec![
                    "No VIBEANVIL_PATCH_FILE set. Generated prompt file instead.".to_string(),
                ],
                files_modified: vec![prompt_path.display().to_string()],
            });
        }

        let patch_path = patch_file.unwrap();

        // Read the diff
        let diff_content = fs::read_to_string(&patch_path)
            .await
            .with_context(|| format!("Failed to read patch file: {}", patch_path.display()))?;

        // Validate the patch (this checks all safety rules)
        let stats = Self::validate_patch(&diff_content, &context.working_dir)?;

        // Print summary
        Self::print_summary(&stats);

        // Parse hunks for file list
        let hunks = Self::parse_diff(&diff_content, &context.working_dir)?;

        if hunks.is_empty() {
            return Ok(ProviderResponse {
                success: false,
                output: String::new(),
                errors: vec!["No valid hunks found in patch file".to_string()],
                warnings: vec![],
                files_modified: vec![],
            });
        }

        // Apply the patch using git apply --check first
        let output = std::process::Command::new("git")
            .arg("apply")
            .arg("--check")
            .arg(&patch_path)
            .current_dir(&context.working_dir)
            .output();

        match output {
            Ok(check_output) if check_output.status.success() => {
                // Apply for real
                let apply_output = std::process::Command::new("git")
                    .arg("apply")
                    .arg(&patch_path)
                    .current_dir(&context.working_dir)
                    .output()?;

                let files_modified: Vec<String> =
                    hunks.iter().map(|h| h.file.display().to_string()).collect();

                if apply_output.status.success() {
                    println!("{}", "✅ Patch applied successfully!".green());
                    for file in &files_modified {
                        println!("  {} {}", "•".cyan(), file);
                    }

                    Ok(ProviderResponse {
                        success: true,
                        output: format!("Applied patch from: {}", patch_path.display()),
                        errors: vec![],
                        warnings: vec![],
                        files_modified,
                    })
                } else {
                    let stderr = String::from_utf8_lossy(&apply_output.stderr).to_string();
                    Ok(ProviderResponse {
                        success: false,
                        output: String::new(),
                        errors: vec![redact_secrets(&stderr)],
                        warnings: vec![],
                        files_modified: vec![],
                    })
                }
            }
            Ok(check_output) => {
                let stderr = String::from_utf8_lossy(&check_output.stderr).to_string();
                Ok(ProviderResponse {
                    success: false,
                    output: String::new(),
                    errors: vec![format!(
                        "Patch does not apply cleanly:\n{}",
                        redact_secrets(&stderr)
                    )],
                    warnings: vec![],
                    files_modified: vec![],
                })
            }
            Err(e) => Err(anyhow::anyhow!("Failed to run git apply: {}", e)),
        }
    }

    fn name(&self) -> &str {
        "patch"
    }

    fn is_available(&self) -> bool {
        which::which("git").is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_name() {
        let provider = PatchProvider::new();
        assert_eq!(provider.name(), "patch");
    }

    #[test]
    fn test_validate_path_rejects_absolute_unix() {
        let repo = std::env::current_dir().unwrap();
        let result = PatchProvider::validate_path(Path::new("/etc/passwd"), &repo);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_path_rejects_absolute_windows() {
        let repo = std::env::current_dir().unwrap();
        let result = PatchProvider::validate_path(Path::new("C:\\Windows\\System32"), &repo);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_path_rejects_traversal() {
        let repo = std::env::current_dir().unwrap();
        let result = PatchProvider::validate_path(Path::new("foo/../../../bar"), &repo);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_path_rejects_git_dir() {
        let repo = std::env::current_dir().unwrap();
        let result = PatchProvider::validate_path(Path::new(".git/config"), &repo);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_path_accepts_simple_relative() {
        let repo = std::env::current_dir().unwrap();
        let result = PatchProvider::validate_path(Path::new("src/main.rs"), &repo);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_patch_rejects_binary() {
        let repo = std::env::current_dir().unwrap();
        let diff = "diff --git a/file.bin b/file.bin\nGIT binary patch\nliteral 1234";
        let result = PatchProvider::validate_patch(diff, &repo);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Binary"));
    }

    #[test]
    fn test_validate_patch_rejects_too_many_files() {
        let repo = std::env::current_dir().unwrap();
        // Create a diff with 60 files (over default limit of 50)
        let mut diff = String::new();
        for i in 0..60 {
            diff.push_str(&format!(
                "--- a/file{}.rs\n+++ b/file{}.rs\n@@ -1,1 +1,2 @@\n line\n+added\n",
                i, i
            ));
        }
        let result = PatchProvider::validate_patch(&diff, &repo);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Too many files"));
    }

    #[test]
    fn test_validate_patch_accepts_small_patch() {
        let repo = std::env::current_dir().unwrap();
        let diff = r#"
--- a/src/main.rs
+++ b/src/main.rs
@@ -1,3 +1,4 @@
 fn main() {
+    println!("Hello");
 }
"#;
        let result = PatchProvider::validate_patch(diff, &repo);
        assert!(result.is_ok());
        let stats = result.unwrap();
        assert_eq!(stats.files_changed, 1);
        assert_eq!(stats.lines_added, 1);
    }
}
