//! Repository Map implementation
//! Scans the codebase to create a high-level summary of the project structure.

use anyhow::Result;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::Path;
use walkdir::WalkDir;

/// A summary of the repository structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryMap {
    /// List of file summaries
    pub files: Vec<FileSummary>,
}

/// Summary of a single file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSummary {
    /// Relative path to the file
    pub path: String,
    /// Extracted signatures (structs, functions, classes)
    pub signatures: Vec<String>,
}

impl RepositoryMap {
    /// Generate a new repository map from the current workspace
    pub fn new() -> Self {
        Self { files: vec![] }
    }

    /// Scan the workspace and populate the map
    pub fn scan(&mut self, root: &Path) -> Result<()> {
        let pb = crate::cli::style::spinner("Scanning codebase...");

        let walker = WalkDir::new(root).into_iter();

        // Regex for Rust structs/enums/functions
        let rust_struct_re = Regex::new(r"^(pub\s+)?(struct|enum|trait)\s+(\w+)").unwrap();
        let rust_fn_re = Regex::new(r"^(pub\s+)?(async\s+)?fn\s+(\w+)").unwrap();
        let rust_impl_re = Regex::new(r"^impl\s+(\w+)").unwrap();

        // Regex for JS/TS classes/functions
        let js_class_re = Regex::new(r"^(export\s+)?class\s+(\w+)").unwrap();
        let js_fn_re = Regex::new(r"^(export\s+)?(async\s+)?function\s+(\w+)").unwrap();
        let js_const_fn_re =
            Regex::new(r"^(export\s+)?const\s+(\w+)\s*=\s*(\(.*\)|async\s*\(.*\))\s*=>").unwrap();


        for entry in walker.filter_entry(|e| !is_hidden(e)) {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
                let signatures = match ext {
                    "rs" => self
                        .extract_signatures(path, &[&rust_struct_re, &rust_fn_re, &rust_impl_re]),
                    "js" | "ts" | "jsx" | "tsx" => {
                        self.extract_signatures(path, &[&js_class_re, &js_fn_re, &js_const_fn_re])
                    }
                    _ => vec![],
                };

                if !signatures.is_empty() {
                    let relative_path = path.strip_prefix(root)?.to_string_lossy().to_string();
                    self.files.push(FileSummary {
                        path: relative_path,
                        signatures,
                    });
                    pb.set_message(format!(
                        "Scanning: {}",
                        path.file_name().unwrap_or_default().to_string_lossy()
                    ));
                }
            }
        }

        // Sort for deterministic output
        self.files.sort_by(|a, b| a.path.cmp(&b.path));

        pb.finish_and_clear();
        Ok(())
    }

    fn extract_signatures(&self, path: &Path, regexes: &[&Regex]) -> Vec<String> {
        let mut signatures = vec![];
        if let Ok(content) = std::fs::read_to_string(path) {
            for line in content.lines() {
                let trimmed = line.trim();
                for re in regexes {
                    if re.is_match(trimmed) {
                        // Keep the original indentation for hierarchy hint, but limit length
                        if trimmed.len() < 100 {
                            signatures.push(trimmed.to_string());
                        }
                        break;
                    }
                }
            }
        }
        signatures
    }

    /// Render the map as a markdown string for LLM context
    pub fn to_markdown(&self) -> String {
        let mut output = String::new();
        output.push_str("# Repository Map\n\n");

        for file in &self.files {
            output.push_str(&format!("## {}\n", file.path));
            output.push_str("```\n");
            for sig in &file.signatures {
                output.push_str(&format!("{}\n", sig));
            }
            output.push_str("```\n\n");
        }

        output
    }
}

fn is_hidden(entry: &walkdir::DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with('.') || s == "target" || s == "node_modules")
        .unwrap_or(false)
}
