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
    // Role-based templates (BMad Method inspired)
    ("architect", include_str!("templates/architect.md")),
    ("developer", include_str!("templates/developer.md")),
    ("qa", include_str!("templates/qa.md")),
    (
        "install-vibeanvil",
        include_str!("templates/install-vibeanvil.md"),
    ),
    // VibeCode Kit v4 templates
    ("debug", include_str!("templates/debug.md")),
    ("xray", include_str!("templates/xray.md")),
    ("vision", include_str!("templates/vision.md")),
    ("migrate", include_str!("templates/migrate.md")),
    ("security", include_str!("templates/security.md")),
    ("refactor", include_str!("templates/refactor.md")),
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

pub fn extract_placeholders(template: &str) -> Vec<String> {
    let mut keys = Vec::new();
    let mut i = 0;
    let bytes = template.as_bytes();

    while i + 3 < bytes.len() {
        if bytes[i] == b'{' && bytes[i + 1] == b'{' {
            let start = i + 2;
            let mut j = start;
            while j + 1 < bytes.len() {
                if bytes[j] == b'}' && bytes[j + 1] == b'}' {
                    let key = template[start..j].trim();
                    if !key.is_empty() && key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
                    {
                        let key_string = key.to_string();
                        if !keys.contains(&key_string) {
                            keys.push(key_string);
                        }
                    }
                    i = j + 2;
                    break;
                }
                j += 1;
            }
            if j + 1 >= bytes.len() {
                break;
            }
            continue;
        }
        i += 1;
    }

    keys
}

pub fn render_checked(template: &str, vars: &HashMap<String, String>) -> (String, Vec<String>) {
    let keys = extract_placeholders(template);
    let missing: Vec<String> = keys
        .iter()
        .filter(|k| !vars.contains_key((*k).as_str()))
        .cloned()
        .collect();

    let mut result = template.to_string();
    for (key, value) in vars {
        result = result.replace(&format!("{{{{{}}}}}", key), value);
    }

    (result, missing)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_unique_placeholders() {
        let t = "Hello {{name}}, stack {{tech_stack}}, repeat {{name}}";
        let keys = extract_placeholders(t);
        assert_eq!(keys, vec!["name".to_string(), "tech_stack".to_string()]);
    }

    #[test]
    fn render_checked_reports_missing() {
        let t = "{{a}} {{b}}";
        let mut vars = HashMap::new();
        vars.insert("a".to_string(), "x".to_string());
        let (rendered, missing) = render_checked(t, &vars);
        assert_eq!(rendered, "x {{b}}");
        assert_eq!(missing, vec!["b".to_string()]);
    }
}
