//! Prompt Templates Module
//! Fabric-style prompt patterns for AI interactions.

use anyhow::Result;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Built-in prompt templates
pub const TEMPLATES: &[(&str, &str)] = &[
    ("plan", include_str!("templates/plan.md")),
    ("review", include_str!("templates/review.md")),
    ("commit", include_str!("templates/commit.md")),
];

/// Load a prompt template by name
pub fn load_template(name: &str) -> Result<String> {
    // First check custom templates in workspace
    let custom_path = PathBuf::from(".vibeanvil/prompts").join(format!("{}.md", name));
    if custom_path.exists() {
        return Ok(std::fs::read_to_string(custom_path)?);
    }

    // Fall back to built-in templates
    for (tmpl_name, content) in TEMPLATES {
        if *tmpl_name == name {
            return Ok(content.to_string());
        }
    }

    anyhow::bail!("Template '{}' not found", name)
}

/// Render a template with variables
pub fn render(template: &str, vars: &HashMap<&str, &str>) -> String {
    let mut result = template.to_string();
    for (key, value) in vars {
        result = result.replace(&format!("{{{{{}}}}}", key), value);
    }
    result
}

/// List available templates
pub fn list_templates(workspace_root: &Path) -> Vec<String> {
    let mut templates: Vec<String> = TEMPLATES.iter().map(|(n, _)| n.to_string()).collect();

    // Add custom templates
    let custom_dir = workspace_root.join(".vibeanvil/prompts");
    if custom_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(custom_dir) {
            for entry in entries.flatten() {
                if let Some(name) = entry.path().file_stem() {
                    templates.push(format!("{} (custom)", name.to_string_lossy()));
                }
            }
        }
    }

    templates
}
