//! Self-update command - check and install updates

use anyhow::Result;
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use std::env;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

const REPO: &str = "ThanhNguyxn/vibeanvil";
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Get latest release info from GitHub
async fn get_latest_release() -> Result<(String, String)> {
    let client = Client::new();
    let url = format!("https://api.github.com/repos/{}/releases/latest", REPO);

    let response = client
        .get(&url)
        .header("User-Agent", "vibeanvil")
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    let tag = response["tag_name"]
        .as_str()
        .unwrap_or("v0.0.0")
        .trim_start_matches('v')
        .to_string();

    let html_url = response["html_url"].as_str().unwrap_or("").to_string();

    Ok((tag, html_url))
}

/// Get download URL for current platform
fn get_download_url(version: &str) -> String {
    #[cfg(target_os = "windows")]
    let artifact = "vibeanvil-windows-x64.exe";

    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    let artifact = "vibeanvil-linux-x64";

    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    let artifact = "vibeanvil-linux-arm64";

    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    let artifact = "vibeanvil-macos-x64";

    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    let artifact = "vibeanvil-macos-arm64";

    #[cfg(not(any(
        target_os = "windows",
        all(target_os = "linux", target_arch = "x86_64"),
        all(target_os = "linux", target_arch = "aarch64"),
        all(target_os = "macos", target_arch = "x86_64"),
        all(target_os = "macos", target_arch = "aarch64")
    )))]
    let artifact = "vibeanvil-linux-x64";

    format!(
        "https://github.com/{}/releases/download/v{}/{}",
        REPO, version, artifact
    )
}

/// Check for updates
pub async fn check_update() -> Result<()> {
    println!();
    println!(
        "{} {}",
        "🔍".cyan(),
        "Checking for updates...".white().bold()
    );

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    pb.set_message("Fetching latest release...");
    pb.enable_steady_tick(std::time::Duration::from_millis(100));

    let (latest, url) = get_latest_release().await?;
    pb.finish_and_clear();

    let current = semver::Version::parse(CURRENT_VERSION)?;
    let latest_ver = semver::Version::parse(&latest)?;

    if latest_ver > current {
        println!();
        println!("{}", "┌─────────────────────────────────────────┐".green());
        println!(
            "│  {} {:<27} │",
            "🚀 New version available:".green().bold(),
            format!("v{}", latest).green().bold()
        );
        println!(
            "│  {} {:<27} │",
            "📦 Current version:".dimmed(),
            format!("v{}", CURRENT_VERSION).dimmed()
        );
        println!("{}", "├─────────────────────────────────────────┤".green());
        println!(
            "│  {} {:<27} │",
            "💡 Tip:".yellow(),
            "Run 'vibeanvil upgrade'".white()
        );
        println!("{}", "└─────────────────────────────────────────┘".green());
        println!();
        println!("  {} {}", "🔗 Release notes:".dimmed(), url.cyan());
        println!();
    } else {
        println!();
        println!("{}", "┌─────────────────────────────────────────┐".green());
        println!(
            "│  {} {:<27} │",
            "✅ You are up to date!".green().bold(),
            format!("v{}", CURRENT_VERSION).green()
        );
        println!("{}", "└─────────────────────────────────────────┘".green());
        println!();
    }

    Ok(())
}

/// Download and install the latest version
pub async fn upgrade() -> Result<()> {
    println!();
    println!(
        "{}",
        "╔═══════════════════════════════════════════════════════════════╗".cyan()
    );
    println!(
        "{}",
        "║               🚀 VibeAnvil Self-Upgrade                       ║".cyan()
    );
    println!(
        "{}",
        "╚═══════════════════════════════════════════════════════════════╝".cyan()
    );
    println!();

    // Check latest version
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    pb.set_message("Checking latest version...");
    pb.enable_steady_tick(std::time::Duration::from_millis(100));

    let (latest, _) = get_latest_release().await?;
    pb.finish_and_clear();

    let current = semver::Version::parse(CURRENT_VERSION)?;
    let latest_ver = semver::Version::parse(&latest)?;

    if latest_ver <= current {
        println!(
            "  {} {} (v{})",
            "✅".green(),
            "Already on the latest version".green().bold(),
            CURRENT_VERSION
        );
        println!();
        return Ok(());
    }

    println!(
        "  {} v{} {} v{}",
        "📦 Upgrading:".white().bold(),
        CURRENT_VERSION.yellow(),
        "→".dimmed(),
        latest.green().bold()
    );

    // Get current executable path
    let current_exe = env::current_exe()?;
    let download_url = get_download_url(&latest);

    println!(
        "  {} {}",
        "📥 Downloading from:".dimmed(),
        download_url.cyan()
    );
    println!();

    // Download with progress
    let client = Client::new();
    let response = client
        .get(&download_url)
        .header("User-Agent", "vibeanvil")
        .send()
        .await?;

    let total_size = response.content_length().unwrap_or(0);

    let pb = ProgressBar::new(total_size);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
            .unwrap()
            .progress_chars("█▓░"),
    );

    let content = response.bytes().await?;
    pb.finish_and_clear();

    println!("  {} {}", "✅".green(), "Download complete!".green().bold());

    // Save to temp file
    let temp_path = PathBuf::from(format!("{}.new", current_exe.display()));
    let mut file = fs::File::create(&temp_path)?;
    file.write_all(&content)?;

    // Make executable on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&temp_path, fs::Permissions::from_mode(0o755))?;
    }

    // Replace current binary
    let backup_path = PathBuf::from(format!("{}.bak", current_exe.display()));

    #[cfg(windows)]
    {
        // On Windows, we need to rename first
        if backup_path.exists() {
            fs::remove_file(&backup_path)?;
        }
        fs::rename(&current_exe, &backup_path)?;
        fs::rename(&temp_path, &current_exe)?;
    }

    #[cfg(unix)]
    {
        fs::rename(&current_exe, &backup_path)?;
        fs::rename(&temp_path, &current_exe)?;
    }

    println!();
    println!("{}", "┌─────────────────────────────────────────┐".green());
    println!(
        "│  {} {:<27} │",
        "🎉 Upgraded to:".green().bold(),
        format!("v{}", latest).green().bold()
    );
    println!(
        "│  {} {:<27} │",
        "💡 Tip:".yellow(),
        "Restart terminal to use".white()
    );
    println!("{}", "└─────────────────────────────────────────┘".green());
    println!();

    Ok(())
}
