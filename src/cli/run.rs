//! Run command - Execute code and capture output for AI feedback (from Aider)
//!
//! Allows running arbitrary commands and optionally sharing the output with the AI.

use anyhow::Result;
use colored::*;
use std::process::Command;

use crate::evidence::EvidenceCollector;
use crate::provider::get_provider;
use crate::workspace;

/// Result of a run command
#[derive(Debug)]
pub struct RunResult {
    pub command: String,
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u128,
    pub success: bool,
}

impl RunResult {
    /// Get combined output (stdout + stderr)
    pub fn combined_output(&self) -> String {
        if self.stderr.is_empty() {
            self.stdout.clone()
        } else if self.stdout.is_empty() {
            self.stderr.clone()
        } else {
            format!("{}\n{}", self.stdout, self.stderr)
        }
    }
}

/// Run a command and capture output
pub async fn run_command(cmd: &str, capture: bool, share_with_ai: bool) -> Result<RunResult> {
    use crate::cli::style;

    style::step(&format!("Running: {}", cmd.cyan()));

    let start = std::time::Instant::now();

    // Determine shell based on OS
    let (shell, shell_arg) = if cfg!(target_os = "windows") {
        ("cmd", "/C")
    } else {
        ("sh", "-c")
    };

    let output = Command::new(shell).arg(shell_arg).arg(cmd).output()?;

    let duration = start.elapsed();
    let exit_code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    let result = RunResult {
        command: cmd.to_string(),
        exit_code,
        stdout,
        stderr,
        duration_ms: duration.as_millis(),
        success: output.status.success(),
    };

    // Display output
    if !result.stdout.is_empty() {
        println!("{}", result.stdout);
    }
    if !result.stderr.is_empty() {
        eprintln!("{}", result.stderr.red());
    }

    // Show status
    if result.success {
        println!(
            "\n{} {} ({}ms)",
            "✓".green(),
            "Command succeeded".green(),
            result.duration_ms
        );
    } else {
        println!(
            "\n{} {} (exit code: {}, {}ms)",
            "✗".red(),
            "Command failed".red(),
            exit_code,
            result.duration_ms
        );
    }

    // Capture evidence if requested
    if capture {
        capture_run_evidence(&result).await?;
    }

    // Share with AI if requested
    if share_with_ai {
        share_output_with_ai(&result).await?;
    }

    Ok(result)
}

/// Capture run output as evidence
async fn capture_run_evidence(result: &RunResult) -> Result<()> {
    use crate::cli::style;

    let state = workspace::load_state().await?;
    let session_id = state
        .current_session_id
        .clone()
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    let evidence = EvidenceCollector::new(&session_id).await?;

    let log_content = format!(
        "# Command Execution Log\n\n\
        **Command**: `{}`\n\
        **Exit Code**: {}\n\
        **Duration**: {}ms\n\
        **Timestamp**: {}\n\n\
        ## stdout\n```\n{}\n```\n\n\
        ## stderr\n```\n{}\n```\n",
        result.command,
        result.exit_code,
        result.duration_ms,
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
        result.stdout,
        result.stderr
    );

    evidence.capture_build_log(&log_content).await?;
    style::success("Output captured as evidence");

    Ok(())
}

/// Share output with AI for analysis
async fn share_output_with_ai(result: &RunResult) -> Result<()> {
    use crate::cli::style;
    use crate::provider::{get_provider, Context};

    // Only share if there's an error or user explicitly wants to
    if result.success {
        println!(
            "\n{}",
            "Command succeeded. Use --share to share output with AI anyway.".dimmed()
        );
        return Ok(());
    }

    style::step("Sharing output with AI for analysis...");

    let state = workspace::load_state().await?;
    let provider = get_provider("claude-code").or_else(|_| get_provider("mock"))?;

    let context = Context {
        working_dir: std::env::current_dir()?,
        session_id: state.current_session_id.clone().unwrap_or_default(),
        contract_hash: state.spec_hash.clone(),
    };

    let prompt = format!(
        r#"The following command failed. Analyze the error and suggest fixes.

**Command**: `{}`
**Exit Code**: {}

**stdout**:
```
{}
```

**stderr**:
```
{}
```

Please:
1. Explain what went wrong
2. Identify the root cause
3. Suggest specific fixes
4. If it's a code error, show the exact fix"#,
        result.command, result.exit_code, result.stdout, result.stderr
    );

    let response = provider.execute(&prompt, &context).await?;

    println!("\n{}", "═".repeat(60).cyan());
    println!("{}", "AI Analysis".cyan().bold());
    println!("{}\n", "═".repeat(60).cyan());
    println!("{}", response.output);

    Ok(())
}

/// Run tests and capture results
pub async fn run_tests(test_cmd: Option<&str>, fix: bool) -> Result<RunResult> {
    use crate::cli::style;

    style::header("Run Tests");

    // Auto-detect test command if not provided
    let cmd = if let Some(c) = test_cmd {
        c.to_string()
    } else {
        detect_test_command()?
    };

    let result = run_command(&cmd, true, false).await?;

    if !result.success && fix {
        style::step("Tests failed. Attempting auto-fix...");

        let state = workspace::load_state().await?;
        let provider = get_provider("claude-code").or_else(|_| get_provider("mock"))?;

        let context = crate::provider::Context {
            working_dir: std::env::current_dir()?,
            session_id: state.current_session_id.clone().unwrap_or_default(),
            contract_hash: state.spec_hash.clone(),
        };

        let fix_prompt = format!(
            r#"The tests are failing. Fix the code to make them pass.

Test command: `{}`
Test output:
```
{}
```

Analyze the failures and fix the code. Output the specific file changes needed."#,
            result.command,
            result.combined_output()
        );

        let response = provider.execute(&fix_prompt, &context).await?;
        println!("\n{}", response.output);

        // Re-run tests
        style::step("Re-running tests after fix...");
        return run_command(&cmd, true, false).await;
    }

    Ok(result)
}

/// Run lint and capture results
pub async fn run_lint(lint_cmd: Option<&str>, fix: bool) -> Result<RunResult> {
    use crate::cli::style;

    style::header("Run Lint");

    // Auto-detect lint command if not provided
    let cmd = if let Some(c) = lint_cmd {
        c.to_string()
    } else {
        detect_lint_command()?
    };

    let result = run_command(&cmd, true, false).await?;

    if !result.success && fix {
        style::step("Lint errors found. Attempting auto-fix...");

        let state = workspace::load_state().await?;
        let provider = get_provider("claude-code").or_else(|_| get_provider("mock"))?;

        let context = crate::provider::Context {
            working_dir: std::env::current_dir()?,
            session_id: state.current_session_id.clone().unwrap_or_default(),
            contract_hash: state.spec_hash.clone(),
        };

        let fix_prompt = format!(
            r#"There are lint errors. Fix the code to pass linting.

Lint command: `{}`
Lint output:
```
{}
```

Fix all lint errors. Output the specific file changes needed."#,
            result.command,
            result.combined_output()
        );

        let response = provider.execute(&fix_prompt, &context).await?;
        println!("\n{}", response.output);

        // Re-run lint
        style::step("Re-running lint after fix...");
        return run_command(&cmd, true, false).await;
    }

    Ok(result)
}

/// Detect the appropriate test command for the project
fn detect_test_command() -> Result<String> {
    let cwd = std::env::current_dir()?;

    // Check for Cargo.toml (Rust)
    if cwd.join("Cargo.toml").exists() {
        return Ok("cargo test".to_string());
    }

    // Check for package.json (Node.js)
    if cwd.join("package.json").exists() {
        return Ok("npm test".to_string());
    }

    // Check for pyproject.toml or setup.py (Python)
    if cwd.join("pyproject.toml").exists() || cwd.join("setup.py").exists() {
        return Ok("pytest".to_string());
    }

    // Check for go.mod (Go)
    if cwd.join("go.mod").exists() {
        return Ok("go test ./...".to_string());
    }

    // Check for Makefile
    if cwd.join("Makefile").exists() {
        return Ok("make test".to_string());
    }

    anyhow::bail!("Could not detect test command. Please specify with --test-cmd")
}

/// Detect the appropriate lint command for the project
fn detect_lint_command() -> Result<String> {
    let cwd = std::env::current_dir()?;

    // Check for Cargo.toml (Rust)
    if cwd.join("Cargo.toml").exists() {
        return Ok("cargo clippy -- -D warnings".to_string());
    }

    // Check for package.json with eslint
    if cwd.join("package.json").exists()
        && (cwd.join(".eslintrc").exists()
            || cwd.join(".eslintrc.js").exists()
            || cwd.join(".eslintrc.json").exists())
    {
        return Ok("npm run lint".to_string());
    }

    // Check for Python
    if cwd.join("pyproject.toml").exists() || cwd.join("setup.py").exists() {
        if cwd.join(".flake8").exists() || cwd.join("setup.cfg").exists() {
            return Ok("flake8".to_string());
        }
        return Ok("ruff check".to_string());
    }

    // Check for Go
    if cwd.join("go.mod").exists() {
        return Ok("golangci-lint run".to_string());
    }

    anyhow::bail!("Could not detect lint command. Please specify with --lint-cmd")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_result_combined_output() {
        let result = RunResult {
            command: "test".to_string(),
            exit_code: 0,
            stdout: "stdout".to_string(),
            stderr: "stderr".to_string(),
            duration_ms: 100,
            success: true,
        };
        assert!(result.combined_output().contains("stdout"));
        assert!(result.combined_output().contains("stderr"));
    }

    #[test]
    fn test_run_result_stdout_only() {
        let result = RunResult {
            command: "test".to_string(),
            exit_code: 0,
            stdout: "stdout only".to_string(),
            stderr: String::new(),
            duration_ms: 100,
            success: true,
        };
        assert_eq!(result.combined_output(), "stdout only");
    }
}
