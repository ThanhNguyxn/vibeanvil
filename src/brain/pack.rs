//! Context Packer implementation
//! Packs the codebase into a single AI-friendly format (XML or Markdown).

use anyhow::Result;
use ignore::WalkBuilder;
use std::path::Path;

/// Output format for the packed context
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackFormat {
    Xml,
    Markdown,
}

impl std::fmt::Display for PackFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PackFormat::Xml => write!(f, "xml"),
            PackFormat::Markdown => write!(f, "markdown"),
        }
    }
}

impl std::str::FromStr for PackFormat {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "xml" => Ok(PackFormat::Xml),
            "md" | "markdown" => Ok(PackFormat::Markdown),
            _ => anyhow::bail!("Invalid format. Use 'xml' or 'markdown'"),
        }
    }
}

/// Pack the codebase into a string
pub fn pack_codebase(root: &Path, format: PackFormat) -> Result<String> {
    let mut output = String::new();
    let mut total_tokens = 0;

    // Header
    match format {
        PackFormat::Xml => {
            output.push_str("<codebase>\n");
        }
        PackFormat::Markdown => {
            output.push_str("# Codebase Context\n\n");
        }
    }

    // Walk files respecting gitignore
    let walker = WalkBuilder::new(root)
        .hidden(false) // Allow hidden files if not gitignored
        .git_ignore(true)
        .ignore(false) // Don't use .ignore files by default, just .gitignore
        .add_custom_ignore_filename(".vibeanvilignore")
        .build();

    for result in walker {
        match result {
            Ok(entry) => {
                let path = entry.path();
                if path.is_file() {
                    // Skip .git directory explicitly if walker doesn't catch it
                    if path.components().any(|c| c.as_os_str() == ".git") {
                        continue;
                    }

                    // Skip lock files and other binary/large files
                    if is_skippable(path) {
                        continue;
                    }

                    if let Ok(content) = std::fs::read_to_string(path) {
                        let relative_path = path.strip_prefix(root).unwrap_or(path);
                        let path_str = relative_path.to_string_lossy();

                        // Estimate tokens (char / 4)
                        total_tokens += content.len() / 4;

                        match format {
                            PackFormat::Xml => {
                                output.push_str(&format!("<file path=\"{}\">\n", path_str));
                                output.push_str(&content);
                                output.push_str("\n</file>\n");
                            }
                            PackFormat::Markdown => {
                                output.push_str(&format!("## File: {}\n\n", path_str));
                                output.push_str(&format!("```{}\n", detect_lang(path)));
                                output.push_str(&content);
                                output.push_str("\n```\n\n");
                            }
                        }
                    }
                }
            }
            Err(err) => {
                crate::cli::style::warn(&format!("Error walking file: {}", err));
            }
        }
    }

    // Footer
    if format == PackFormat::Xml {
        output.push_str("</codebase>\n");
    }

    // Summary
    crate::cli::style::info(&format!(
        "Packed {} chars (~{} tokens)",
        output.len(),
        total_tokens
    ));

    Ok(output)
}

fn is_skippable(path: &Path) -> bool {
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
    matches!(
        ext,
        "lock"
            | "png"
            | "jpg"
            | "jpeg"
            | "gif"
            | "ico"
            | "pdf"
            | "zip"
            | "tar"
            | "gz"
            | "exe"
            | "dll"
            | "so"
            | "dylib"
            | "class"
            | "o"
            | "obj"
    )
}

fn detect_lang(path: &Path) -> &str {
    path.extension().and_then(|s| s.to_str()).unwrap_or("text")
}
