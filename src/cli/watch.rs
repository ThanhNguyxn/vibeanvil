//! Watch mode for build iterate - auto-rebuild on file changes

use anyhow::Result;
use colored::Colorize;
use notify::RecursiveMode;
use notify_debouncer_mini::new_debouncer;
use std::path::Path;
use std::sync::mpsc::channel;
use std::time::Duration;

/// Default directories to watch for changes
const WATCH_DIRS: &[&str] = &["src", "lib", "app", "tests", "examples"];

/// Files/patterns to ignore
const IGNORE_PATTERNS: &[&str] = &[
    "target",
    ".git",
    ".vibeanvil",
    "node_modules",
    ".next",
    "dist",
    "build",
    "coverage",
];

/// Watch for file changes and trigger rebuild
pub struct FileWatcher {
    debounce_ms: u64,
}

impl FileWatcher {
    pub fn new() -> Self {
        Self { debounce_ms: 500 }
    }

    /// Start watching and call callback on changes
    pub fn watch<F>(&self, callback: F) -> Result<()>
    where
        F: Fn() -> Result<()> + Send + 'static,
    {
        let cwd = std::env::current_dir()?;

        // Find directories to watch
        let watch_dirs: Vec<_> = WATCH_DIRS
            .iter()
            .map(|d| cwd.join(d))
            .filter(|p| p.exists())
            .collect();

        if watch_dirs.is_empty() {
            println!(
                "{}",
                "⚠️  No source directories found to watch. Watching current directory.".yellow()
            );
        }

        println!();
        println!("{}", "👁️  Watch Mode Active".cyan().bold());
        println!("{}", "─".repeat(40).dimmed());
        println!(
            "{}",
            "Watching for file changes... Press Ctrl+C to stop.".dimmed()
        );
        println!();

        // Setup debounced watcher
        let (tx, rx) = channel();
        let mut debouncer = new_debouncer(Duration::from_millis(self.debounce_ms), tx)?;

        // Watch directories
        if watch_dirs.is_empty() {
            debouncer.watcher().watch(&cwd, RecursiveMode::Recursive)?;
        } else {
            for dir in &watch_dirs {
                if let Err(e) = debouncer.watcher().watch(dir, RecursiveMode::Recursive) {
                    eprintln!("Warning: Could not watch {:?}: {}", dir, e);
                } else {
                    println!("  {} {}", "•".cyan(), dir.display());
                }
            }
        }
        println!();

        // Event loop
        loop {
            match rx.recv() {
                Ok(Ok(events)) => {
                    // Filter out ignored paths
                    let relevant_events: Vec<_> = events
                        .iter()
                        .filter(|e| !self.should_ignore(&e.path))
                        .collect();

                    if !relevant_events.is_empty() {
                        println!();
                        println!(
                            "🔄 {} file(s) changed",
                            relevant_events.len()
                        );

                        for event in &relevant_events {
                            let path_str = event
                                .path
                                .strip_prefix(&cwd)
                                .unwrap_or(&event.path)
                                .display();
                            println!("   {} {}", "→".dimmed(), path_str);
                        }

                        println!();
                        println!("{}", "Rebuilding...".yellow());

                        if let Err(e) = callback() {
                            eprintln!("{} {}", "❌ Build failed:".red(), e);
                        }

                        println!();
                        println!(
                            "{}",
                            "Watching for file changes... Press Ctrl+C to stop.".dimmed()
                        );
                    }
                }
                Ok(Err(e)) => {
                    eprintln!("Watch error: {:?}", e);
                }
                Err(e) => {
                    eprintln!("Channel error: {:?}", e);
                    break;
                }
            }
        }

        Ok(())
    }

    fn should_ignore(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();

        for pattern in IGNORE_PATTERNS {
            if path_str.contains(pattern) {
                return true;
            }
        }

        // Ignore hidden files
        if let Some(file_name) = path.file_name() {
            if file_name.to_string_lossy().starts_with('.') {
                return true;
            }
        }

        false
    }
}

impl Default for FileWatcher {
    fn default() -> Self {
        Self::new()
    }
}
