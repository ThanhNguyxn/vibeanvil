//! Repository Map - Codebase mapping for AI context (from Aider)
//!
//! Creates a structural map of the codebase to help AI understand the project layout.

use anyhow::Result;
use colored::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;

/// A symbol extracted from a file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Symbol {
    /// Symbol name
    pub name: String,
    /// Symbol kind (function, class, struct, etc.)
    pub kind: String,
    /// Line number where defined
    pub line: usize,
    /// Signature or brief definition
    pub signature: String,
}

/// Information about a single file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    /// Relative path from project root
    pub path: String,
    /// File language
    pub language: String,
    /// Line count
    pub lines: usize,
    /// Symbols defined in this file
    pub symbols: Vec<Symbol>,
    /// Imports/dependencies
    pub imports: Vec<String>,
}

/// The complete repository map
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoMap {
    /// Root path of the repository
    pub root: String,
    /// Map generation timestamp
    pub generated_at: String,
    /// Files in the repository
    pub files: Vec<FileInfo>,
    /// Language statistics
    pub language_stats: HashMap<String, LanguageStats>,
    /// Total line count
    pub total_lines: usize,
    /// Total file count
    pub total_files: usize,
}

/// Statistics for a language
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LanguageStats {
    /// Number of files
    pub files: usize,
    /// Total lines
    pub lines: usize,
}

impl RepoMap {
    /// Create a new repository map
    pub async fn new(root: &Path) -> Result<Self> {
        let mut files = Vec::new();
        let mut language_stats: HashMap<String, LanguageStats> = HashMap::new();
        let mut total_lines = 0;

        // Walk the directory tree
        let entries = collect_source_files(root).await?;

        for entry in entries {
            if let Ok(info) = analyze_file(root, &entry).await {
                total_lines += info.lines;

                // Update language stats
                let stats = language_stats.entry(info.language.clone()).or_default();
                stats.files += 1;
                stats.lines += info.lines;

                files.push(info);
            }
        }

        let total_files = files.len();

        Ok(Self {
            root: root.to_string_lossy().to_string(),
            generated_at: chrono::Utc::now().to_rfc3339(),
            files,
            language_stats,
            total_lines,
            total_files,
        })
    }

    /// Generate a compact text representation for AI context
    pub fn to_context_string(&self, max_tokens: usize) -> String {
        let mut output = String::new();
        output.push_str("# Repository Map\n\n");

        // Language summary
        output.push_str("## Languages\n");
        for (lang, stats) in &self.language_stats {
            output.push_str(&format!(
                "- {}: {} files, {} lines\n",
                lang, stats.files, stats.lines
            ));
        }
        output.push('\n');

        // File tree
        output.push_str("## File Structure\n```\n");

        // Build tree structure
        let mut tree = Tree::new();
        for file in &self.files {
            tree.add_path(&file.path);
        }
        output.push_str(&tree.to_string());
        output.push_str("```\n\n");

        // Key symbols (if we have room)
        let current_size = output.len() / 4; // Rough token estimate
        if current_size < max_tokens {
            output.push_str("## Key Symbols\n");
            for file in &self.files {
                if !file.symbols.is_empty() {
                    output.push_str(&format!("\n### {}\n", file.path));
                    for symbol in &file.symbols {
                        output.push_str(&format!(
                            "- {} {} (line {})\n",
                            symbol.kind, symbol.name, symbol.line
                        ));
                        if current_size > max_tokens {
                            break;
                        }
                    }
                }
            }
        }

        output
    }

    /// Display the map to console
    pub fn display(&self) {
        println!("\n{}", "═".repeat(60).cyan());
        println!("{}", "Repository Map".cyan().bold());
        println!("{}\n", "═".repeat(60).cyan());

        println!("{}", "Languages:".yellow());
        let mut langs: Vec<_> = self.language_stats.iter().collect();
        langs.sort_by(|a, b| b.1.lines.cmp(&a.1.lines));
        for (lang, stats) in langs {
            let bar_width = (stats.lines as f32 / self.total_lines as f32 * 30.0) as usize;
            println!(
                "  {:12} {:>5} files {:>6} lines [{}{}]",
                lang,
                stats.files,
                stats.lines,
                "█".repeat(bar_width).green(),
                "░".repeat(30 - bar_width).dimmed()
            );
        }

        println!("\n{}", "File Structure:".yellow());
        let mut tree = Tree::new();
        for file in &self.files {
            tree.add_path(&file.path);
        }
        println!("{}", tree.to_string());

        println!(
            "\n{} {} files, {} lines",
            "Total:".yellow(),
            self.total_files,
            self.total_lines
        );
    }
}

/// Simple tree structure for display
struct Tree {
    children: HashMap<String, Tree>,
    is_file: bool,
}

impl Tree {
    fn new() -> Self {
        Self {
            children: HashMap::new(),
            is_file: false,
        }
    }

    fn add_path(&mut self, path: &str) {
        let parts: Vec<&str> = path.split('/').collect();
        let mut current = self;

        for (i, part) in parts.iter().enumerate() {
            let child = current
                .children
                .entry(part.to_string())
                .or_insert_with(Tree::new);
            if i == parts.len() - 1 {
                child.is_file = true;
            }
            current = child;
        }
    }

    fn to_string(&self) -> String {
        self.format_tree("", true)
    }

    fn format_tree(&self, prefix: &str, _is_last: bool) -> String {
        let mut output = String::new();
        let mut entries: Vec<_> = self.children.iter().collect();
        entries.sort_by(|a, b| a.0.cmp(b.0));

        for (i, (name, tree)) in entries.iter().enumerate() {
            let is_last_entry = i == entries.len() - 1;
            let connector = if is_last_entry {
                "└── "
            } else {
                "├── "
            };
            let extension = if is_last_entry { "    " } else { "│   " };

            let display_name = if tree.is_file {
                name.to_string()
            } else {
                format!("{}/", name)
            };

            output.push_str(&format!("{}{}{}\n", prefix, connector, display_name));

            if !tree.children.is_empty() {
                output.push_str(
                    &tree.format_tree(&format!("{}{}", prefix, extension), is_last_entry),
                );
            }
        }

        output
    }
}

/// Collect all source files in a directory
async fn collect_source_files(root: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    collect_files_recursive(root, root, &mut files).await?;
    Ok(files)
}

#[async_recursion::async_recursion]
async fn collect_files_recursive(root: &Path, dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    let mut entries = fs::read_dir(dir).await?;

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        let file_name = path.file_name().unwrap_or_default().to_string_lossy();

        // Skip hidden files and common ignore patterns
        if file_name.starts_with('.')
            || file_name == "node_modules"
            || file_name == "target"
            || file_name == "__pycache__"
            || file_name == "venv"
            || file_name == ".git"
            || file_name == "dist"
            || file_name == "build"
        {
            continue;
        }

        if path.is_dir() {
            collect_files_recursive(root, &path, files).await?;
        } else if is_source_file(&path) {
            files.push(path);
        }
    }

    Ok(())
}

/// Check if a file is a source file
fn is_source_file(path: &Path) -> bool {
    let extensions = [
        "rs", "py", "js", "ts", "jsx", "tsx", "go", "java", "c", "cpp", "h", "hpp", "rb", "php",
        "swift", "kt", "scala", "cs", "fs", "ml", "hs", "ex", "exs", "lua", "r", "jl", "nim",
        "zig", "v", "d", "dart", "vue", "svelte", "sql", "sh", "bash", "zsh", "ps1", "md", "yaml",
        "yml", "toml", "json", "xml", "html", "css", "scss", "sass", "less", "graphql", "prisma",
        "proto",
    ];

    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| extensions.contains(&e.to_lowercase().as_str()))
        .unwrap_or(false)
}

/// Get the language from file extension
fn get_language(path: &Path) -> String {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "rs" => "Rust",
        "py" => "Python",
        "js" => "JavaScript",
        "ts" => "TypeScript",
        "jsx" | "tsx" => "React",
        "go" => "Go",
        "java" => "Java",
        "c" | "h" => "C",
        "cpp" | "hpp" | "cc" => "C++",
        "rb" => "Ruby",
        "php" => "PHP",
        "swift" => "Swift",
        "kt" => "Kotlin",
        "scala" => "Scala",
        "cs" => "C#",
        "fs" => "F#",
        "ml" => "OCaml",
        "hs" => "Haskell",
        "ex" | "exs" => "Elixir",
        "lua" => "Lua",
        "r" => "R",
        "jl" => "Julia",
        "nim" => "Nim",
        "zig" => "Zig",
        "dart" => "Dart",
        "vue" => "Vue",
        "svelte" => "Svelte",
        "sql" => "SQL",
        "sh" | "bash" => "Shell",
        "ps1" => "PowerShell",
        "md" => "Markdown",
        "yaml" | "yml" => "YAML",
        "toml" => "TOML",
        "json" => "JSON",
        "xml" => "XML",
        "html" => "HTML",
        "css" | "scss" | "sass" | "less" => "CSS",
        "graphql" => "GraphQL",
        "proto" => "Protobuf",
        _ => "Other",
    }
    .to_string()
}

/// Analyze a single file
async fn analyze_file(root: &Path, path: &Path) -> Result<FileInfo> {
    let content = fs::read_to_string(path).await?;
    let lines = content.lines().count();
    let language = get_language(path);

    let relative_path = path
        .strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/");

    // Extract symbols (basic implementation)
    let symbols = extract_symbols(&content, &language);
    let imports = extract_imports(&content, &language);

    Ok(FileInfo {
        path: relative_path,
        language,
        lines,
        symbols,
        imports,
    })
}

/// Extract symbols from file content (basic implementation)
fn extract_symbols(content: &str, language: &str) -> Vec<Symbol> {
    let mut symbols = Vec::new();

    for (line_num, line) in content.lines().enumerate() {
        let trimmed = line.trim();

        match language {
            "Rust" => {
                if let Some(name) = extract_rust_symbol(trimmed) {
                    symbols.push(name.with_line(line_num + 1));
                }
            }
            "Python" => {
                if let Some(name) = extract_python_symbol(trimmed) {
                    symbols.push(name.with_line(line_num + 1));
                }
            }
            "JavaScript" | "TypeScript" | "React" => {
                if let Some(name) = extract_js_symbol(trimmed) {
                    symbols.push(name.with_line(line_num + 1));
                }
            }
            "Go" => {
                if let Some(name) = extract_go_symbol(trimmed) {
                    symbols.push(name.with_line(line_num + 1));
                }
            }
            _ => {}
        }
    }

    symbols
}

/// Extract imports from file content
fn extract_imports(content: &str, language: &str) -> Vec<String> {
    let mut imports = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();

        match language {
            "Rust" => {
                if trimmed.starts_with("use ") {
                    imports.push(trimmed.to_string());
                }
            }
            "Python" => {
                if trimmed.starts_with("import ") || trimmed.starts_with("from ") {
                    imports.push(trimmed.to_string());
                }
            }
            "JavaScript" | "TypeScript" | "React" => {
                if trimmed.starts_with("import ") || trimmed.contains("require(") {
                    imports.push(trimmed.to_string());
                }
            }
            "Go" => {
                if trimmed.starts_with("import ") {
                    imports.push(trimmed.to_string());
                }
            }
            _ => {}
        }
    }

    imports
}

fn extract_rust_symbol(line: &str) -> Option<Symbol> {
    if line.starts_with("pub fn ") || line.starts_with("fn ") {
        let sig = line.split('{').next()?.trim();
        let name = sig
            .strip_prefix("pub fn ")
            .or_else(|| sig.strip_prefix("fn "))?;
        let name = name.split('(').next()?.trim();
        return Some(Symbol {
            name: name.to_string(),
            kind: "fn".to_string(),
            line: 0,
            signature: sig.to_string(),
        });
    }

    if line.starts_with("pub struct ") || line.starts_with("struct ") {
        let name = line
            .strip_prefix("pub struct ")
            .or_else(|| line.strip_prefix("struct "))?;
        let name = name.split(['<', '{', ' ']).next()?.trim();
        return Some(Symbol {
            name: name.to_string(),
            kind: "struct".to_string(),
            line: 0,
            signature: line.to_string(),
        });
    }

    if line.starts_with("pub enum ") || line.starts_with("enum ") {
        let name = line
            .strip_prefix("pub enum ")
            .or_else(|| line.strip_prefix("enum "))?;
        let name = name.split(['<', '{', ' ']).next()?.trim();
        return Some(Symbol {
            name: name.to_string(),
            kind: "enum".to_string(),
            line: 0,
            signature: line.to_string(),
        });
    }

    if line.starts_with("pub trait ") || line.starts_with("trait ") {
        let name = line
            .strip_prefix("pub trait ")
            .or_else(|| line.strip_prefix("trait "))?;
        let name = name.split(['<', '{', ' ']).next()?.trim();
        return Some(Symbol {
            name: name.to_string(),
            kind: "trait".to_string(),
            line: 0,
            signature: line.to_string(),
        });
    }

    if line.starts_with("impl ") {
        let rest = line.strip_prefix("impl ")?;
        let name = rest.split(['{', ' ']).next()?.trim();
        return Some(Symbol {
            name: name.to_string(),
            kind: "impl".to_string(),
            line: 0,
            signature: line.to_string(),
        });
    }

    None
}

fn extract_python_symbol(line: &str) -> Option<Symbol> {
    if line.starts_with("def ") {
        let sig = line.split(':').next()?.trim();
        let name = sig.strip_prefix("def ")?;
        let name = name.split('(').next()?.trim();
        return Some(Symbol {
            name: name.to_string(),
            kind: "def".to_string(),
            line: 0,
            signature: sig.to_string(),
        });
    }

    if line.starts_with("class ") {
        let sig = line.split(':').next()?.trim();
        let name = sig.strip_prefix("class ")?;
        let name = name.split(['(', ' ']).next()?.trim();
        return Some(Symbol {
            name: name.to_string(),
            kind: "class".to_string(),
            line: 0,
            signature: sig.to_string(),
        });
    }

    if line.starts_with("async def ") {
        let sig = line.split(':').next()?.trim();
        let name = sig.strip_prefix("async def ")?;
        let name = name.split('(').next()?.trim();
        return Some(Symbol {
            name: name.to_string(),
            kind: "async def".to_string(),
            line: 0,
            signature: sig.to_string(),
        });
    }

    None
}

fn extract_js_symbol(line: &str) -> Option<Symbol> {
    // function declarations
    if line.starts_with("function ") {
        let name = line.strip_prefix("function ")?;
        let name = name.split('(').next()?.trim();
        return Some(Symbol {
            name: name.to_string(),
            kind: "function".to_string(),
            line: 0,
            signature: line.to_string(),
        });
    }

    // async function
    if line.starts_with("async function ") {
        let name = line.strip_prefix("async function ")?;
        let name = name.split('(').next()?.trim();
        return Some(Symbol {
            name: name.to_string(),
            kind: "async function".to_string(),
            line: 0,
            signature: line.to_string(),
        });
    }

    // class
    if line.starts_with("class ") {
        let name = line.strip_prefix("class ")?;
        let name = name.split([' ', '{']).next()?.trim();
        return Some(Symbol {
            name: name.to_string(),
            kind: "class".to_string(),
            line: 0,
            signature: line.to_string(),
        });
    }

    // export function
    if line.starts_with("export function ") || line.starts_with("export async function ") {
        let rest = line
            .strip_prefix("export async function ")
            .or_else(|| line.strip_prefix("export function "))?;
        let name = rest.split('(').next()?.trim();
        return Some(Symbol {
            name: name.to_string(),
            kind: "export function".to_string(),
            line: 0,
            signature: line.to_string(),
        });
    }

    // const arrow function
    if line.starts_with("const ") && (line.contains(" = (") || line.contains(" = async (")) {
        let name = line.strip_prefix("const ")?;
        let name = name.split([' ', '=']).next()?.trim();
        return Some(Symbol {
            name: name.to_string(),
            kind: "const".to_string(),
            line: 0,
            signature: line.to_string(),
        });
    }

    None
}

fn extract_go_symbol(line: &str) -> Option<Symbol> {
    if line.starts_with("func ") {
        let sig = line.split('{').next()?.trim();
        let name = sig.strip_prefix("func ")?;
        let name = if name.starts_with('(') {
            // Method: func (r *Receiver) MethodName
            name.split(')').nth(1)?.trim().split('(').next()?.trim()
        } else {
            name.split('(').next()?.trim()
        };
        return Some(Symbol {
            name: name.to_string(),
            kind: "func".to_string(),
            line: 0,
            signature: sig.to_string(),
        });
    }

    if line.starts_with("type ") && line.contains(" struct ") {
        let rest = line.strip_prefix("type ")?;
        let name = rest.split([' ', '{']).next()?.trim();
        return Some(Symbol {
            name: name.to_string(),
            kind: "struct".to_string(),
            line: 0,
            signature: line.to_string(),
        });
    }

    if line.starts_with("type ") && line.contains(" interface ") {
        let rest = line.strip_prefix("type ")?;
        let name = rest.split([' ', '{']).next()?.trim();
        return Some(Symbol {
            name: name.to_string(),
            kind: "interface".to_string(),
            line: 0,
            signature: line.to_string(),
        });
    }

    None
}

impl Symbol {
    fn with_line(mut self, line: usize) -> Self {
        self.line = line;
        self
    }
}

/// Generate the repository map
pub async fn run_map(max_tokens: Option<usize>) -> Result<()> {
    use crate::cli::style;
    use crate::workspace;

    style::header("Repository Map");

    let cwd = std::env::current_dir()?;
    let map = RepoMap::new(&cwd).await?;

    map.display();

    // Save the map
    let anvil_dir = workspace::get_anvil_dir()?;
    let map_path = anvil_dir.join("repomap.json");
    let map_json = serde_json::to_string_pretty(&map)?;
    tokio::fs::write(&map_path, &map_json).await?;

    // Also save compact context version
    let context = map.to_context_string(max_tokens.unwrap_or(4000));
    let context_path = anvil_dir.join("repomap.md");
    tokio::fs::write(&context_path, &context).await?;

    style::success("Repository map saved to .anvil/repomap.json and .anvil/repomap.md");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_language() {
        assert_eq!(get_language(Path::new("test.rs")), "Rust");
        assert_eq!(get_language(Path::new("test.py")), "Python");
        assert_eq!(get_language(Path::new("test.ts")), "TypeScript");
        assert_eq!(get_language(Path::new("test.go")), "Go");
    }

    #[test]
    fn test_extract_rust_symbol() {
        let sym = extract_rust_symbol("pub fn hello_world() {").unwrap();
        assert_eq!(sym.name, "hello_world");
        assert_eq!(sym.kind, "fn");

        let sym = extract_rust_symbol("struct MyStruct {").unwrap();
        assert_eq!(sym.name, "MyStruct");
        assert_eq!(sym.kind, "struct");
    }

    #[test]
    fn test_extract_python_symbol() {
        let sym = extract_python_symbol("def my_function():").unwrap();
        assert_eq!(sym.name, "my_function");
        assert_eq!(sym.kind, "def");

        let sym = extract_python_symbol("class MyClass:").unwrap();
        assert_eq!(sym.name, "MyClass");
        assert_eq!(sym.kind, "class");
    }

    #[test]
    fn test_tree() {
        let mut tree = Tree::new();
        tree.add_path("src/main.rs");
        tree.add_path("src/lib.rs");
        tree.add_path("tests/test.rs");

        let output = tree.to_string();
        assert!(output.contains("src/"));
        assert!(output.contains("main.rs"));
    }
}
