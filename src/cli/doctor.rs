//! Doctor command - Check system and workspace health

use anyhow::Result;
use colored::Colorize;
use std::process::Command;

use crate::brain::storage::BrainStorage;
use crate::state::State;
use crate::workspace;

/// Health check result
struct HealthCheck {
    name: String,
    status: CheckStatus,
    message: String,
}

enum CheckStatus {
    Ok,
    Warning,
    Error,
    Info,
}

impl HealthCheck {
    fn ok(name: &str, message: &str) -> Self {
        Self {
            name: name.to_string(),
            status: CheckStatus::Ok,
            message: message.to_string(),
        }
    }

    fn warning(name: &str, message: &str) -> Self {
        Self {
            name: name.to_string(),
            status: CheckStatus::Warning,
            message: message.to_string(),
        }
    }

    fn error(name: &str, message: &str) -> Self {
        Self {
            name: name.to_string(),
            status: CheckStatus::Error,
            message: message.to_string(),
        }
    }

    fn info(name: &str, message: &str) -> Self {
        Self {
            name: name.to_string(),
            status: CheckStatus::Info,
            message: message.to_string(),
        }
    }

    fn print(&self) {
        let (icon, color_fn): (&str, fn(&str) -> colored::ColoredString) = match self.status {
            CheckStatus::Ok => ("✅", |s| s.green()),
            CheckStatus::Warning => ("⚠️ ", |s| s.yellow()),
            CheckStatus::Error => ("❌", |s| s.red()),
            CheckStatus::Info => ("ℹ️ ", |s| s.cyan()),
        };

        println!(
            "  {} {}: {}",
            icon,
            self.name.white(),
            color_fn(&self.message)
        );
    }
}

pub async fn run() -> Result<()> {
    println!();
    println!(
        "{}",
        "╔═══════════════════════════════════════════════════════════════╗".cyan()
    );
    println!(
        "{}",
        "║               🔍 VibeAnvil Doctor                             ║".cyan()
    );
    println!(
        "{}",
        "╠═══════════════════════════════════════════════════════════════╣".cyan()
    );
    println!(
        "{}",
        "║   Checking installation and workspace health...               ║".white()
    );
    println!(
        "{}",
        "╚═══════════════════════════════════════════════════════════════╝".cyan()
    );
    println!();

    let mut checks: Vec<HealthCheck> = Vec::new();
    let mut has_errors = false;
    let mut has_warnings = false;

    // 1. Check VibeAnvil version
    let version = env!("CARGO_PKG_VERSION");
    checks.push(HealthCheck::ok("VibeAnvil CLI", &format!("v{}", version)));

    // 2. Check Rust version (if available)
    match Command::new("rustc").arg("--version").output() {
        Ok(output) => {
            let version_str = String::from_utf8_lossy(&output.stdout);
            let version_clean = version_str.trim();
            checks.push(HealthCheck::ok("Rust Compiler", version_clean));
        }
        Err(_) => {
            checks.push(HealthCheck::info("Rust Compiler", "Not found (optional)"));
        }
    }

    // 3. Check Git
    match Command::new("git").arg("--version").output() {
        Ok(output) => {
            let version_str = String::from_utf8_lossy(&output.stdout);
            let version_clean = version_str.trim();
            checks.push(HealthCheck::ok("Git", version_clean));
        }
        Err(_) => {
            checks.push(HealthCheck::warning("Git", "Not found (recommended)"));
            has_warnings = true;
        }
    }

    // 4. Check workspace
    if workspace::workspace_exists().await {
        checks.push(HealthCheck::ok("Workspace", ".vibeanvil/ found"));

        // 5. Check state.json
        match workspace::load_state().await {
            Ok(state) => {
                checks.push(HealthCheck::ok(
                    "State",
                    &format!("state.json valid ({:?})", state.current_state),
                ));

                // 6. Check contract lock
                if state.current_state == State::ContractLocked
                    || state.current_state.is_at_least(State::ContractLocked)
                {
                    if let Some(hash) = &state.spec_hash {
                        let short_hash = if hash.len() > 8 { &hash[..8] } else { hash };
                        checks.push(HealthCheck::ok(
                            "Contract",
                            &format!("Locked ({}...)", short_hash),
                        ));
                    } else {
                        checks.push(HealthCheck::ok("Contract", "Locked"));
                    }
                } else {
                    checks.push(HealthCheck::warning("Contract", "Not locked"));
                    has_warnings = true;
                }
            }
            Err(e) => {
                checks.push(HealthCheck::error("State", &format!("Invalid: {}", e)));
                has_errors = true;
            }
        }
    } else {
        checks.push(HealthCheck::warning(
            "Workspace",
            "Not initialized (run: vibeanvil init)",
        ));
        has_warnings = true;
    }

    // 7. Check BrainPack
    match BrainStorage::new().await {
        Ok(storage) => match storage.stats().await {
            Ok(stats) => {
                let record_count = stats.total_records;
                if record_count > 0 {
                    checks.push(HealthCheck::ok(
                        "BrainPack",
                        &format!(
                            "{} records, {} chunks",
                            stats.total_records, stats.total_chunks
                        ),
                    ));
                } else {
                    checks.push(HealthCheck::warning(
                        "BrainPack",
                        "Empty (run: vibeanvil brain ensure)",
                    ));
                    has_warnings = true;
                }
            }
            Err(_) => {
                checks.push(HealthCheck::warning(
                    "BrainPack",
                    "Not initialized (run: vibeanvil brain ensure)",
                ));
                has_warnings = true;
            }
        },
        Err(_) => {
            checks.push(HealthCheck::warning("BrainPack", "Database not accessible"));
            has_warnings = true;
        }
    }

    // 8. Check GITHUB_TOKEN
    if std::env::var("GITHUB_TOKEN").is_ok() {
        checks.push(HealthCheck::ok("GitHub Token", "Set (harvest ready)"));
    } else {
        checks.push(HealthCheck::info(
            "GitHub Token",
            "Not set (harvest will have rate limits)",
        ));
    }

    // Print all checks
    println!("{}", "📋 Health Checks:".white().bold());
    println!();
    for check in &checks {
        check.print();
    }

    // Summary
    println!();
    println!("{}", "─".repeat(50).dimmed());
    if has_errors {
        println!(
            "{}",
            "  ❌ Some checks failed. Please fix the errors above.".red()
        );
    } else if has_warnings {
        println!("{}", "  ⚠️  All checks passed! (with warnings)".yellow());
    } else {
        println!("{}", "  ✅ All checks passed!".green());
    }
    println!();

    // Next steps suggestions
    if has_warnings || has_errors {
        println!("{}", "💡 Suggestions:".white().bold());
        for check in &checks {
            match check.status {
                CheckStatus::Warning | CheckStatus::Error => {
                    if check.name == "Workspace" {
                        println!("  {} {}", "•".cyan(), "vibeanvil init".white());
                    } else if check.name == "BrainPack" {
                        println!("  {} {}", "•".cyan(), "vibeanvil brain ensure".white());
                    } else if check.name == "Contract" {
                        println!("  {} {}", "•".cyan(), "vibeanvil contract lock".white());
                    }
                }
                _ => {}
            }
        }
        println!();
    }

    Ok(())
}
