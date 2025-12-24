//! Patch provider - safely applies unified diffs

use anyhow::{Context as AnyhowContext, Result};
use async_trait::async_trait;
use colored::Colorize;
use std::path::{Path, PathBuf};
use tokio::fs;

use super::{Context, Provider, ProviderResponse};

/// Patch provider for unified diff workflows
///
/// Configuration:
/// - `VIBEANVIL_PATCH_FILE`: Path to unified diff file to apply
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

        // Reject absolute paths (both Windows C:\ and Unix /)
        if path.is_absolute() || path_str.starts_with('/') {
            anyhow::bail!("Absolute paths not allowed in diff: {}", path.display());
        }

        // Check for path traversal
        if path_str.contains("..") {
            anyhow::bail!("Path traversal not allowed in diff: {}", path.display());
        }

        // Path is relative and doesn't traverse up - it's safe
        // The actual file existence will be checked when git apply runs
        Ok(())
    }

    /// Parse and validate unified diff content
    fn parse_diff(content: &str, repo_root: &Path) -> Result<Vec<DiffHunk>> {
        let mut hunks = Vec::new();
        let mut current_file: Option<PathBuf> = None;
        let mut current_lines: Vec<String> = Vec::new();

        for line in content.lines() {
            if line.starts_with("--- ") || line.starts_with("+++ ") {
                // Extract file path (skip a/ or b/ prefix)
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
                // Start of a new hunk
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

        // Add final hunk
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
        // Check if patch file is provided
        let patch_file = Self::patch_file_path();

        if patch_file.is_none() {
            // No patch file - generate prompt file instead
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

        // Read and parse the diff
        let diff_content = fs::read_to_string(&patch_path)
            .await
            .with_context(|| format!("Failed to read patch file: {}", patch_path.display()))?;

        // Validate paths in the diff
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

        // Apply the patch using git apply
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
                    Ok(ProviderResponse {
                        success: false,
                        output: String::new(),
                        errors: vec![String::from_utf8_lossy(&apply_output.stderr).to_string()],
                        warnings: vec![],
                        files_modified: vec![],
                    })
                }
            }
            Ok(check_output) => Ok(ProviderResponse {
                success: false,
                output: String::new(),
                errors: vec![format!(
                    "Patch does not apply cleanly: {}",
                    String::from_utf8_lossy(&check_output.stderr)
                )],
                warnings: vec![],
                files_modified: vec![],
            }),
            Err(e) => Err(anyhow::anyhow!("Failed to run git apply: {}", e)),
        }
    }

    fn name(&self) -> &str {
        "patch"
    }

    fn is_available(&self) -> bool {
        // Patch provider requires git
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
    fn test_validate_path_rejects_absolute() {
        let repo = std::env::current_dir().unwrap();
        // Unix absolute path
        let result = PatchProvider::validate_path(Path::new("/etc/passwd"), &repo);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_path_rejects_traversal() {
        let repo = std::env::current_dir().unwrap();
        let result = PatchProvider::validate_path(Path::new("foo/../../../bar"), &repo);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_path_accepts_simple_relative() {
        let repo = std::env::current_dir().unwrap();
        let result = PatchProvider::validate_path(Path::new("src/main.rs"), &repo);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_path_accepts_nested_relative() {
        let repo = std::env::current_dir().unwrap();
        let result = PatchProvider::validate_path(Path::new("src/provider/patch.rs"), &repo);
        assert!(result.is_ok());
    }
}
