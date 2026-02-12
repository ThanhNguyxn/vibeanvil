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
    render_template_with_lookup(template, |key, filter| {
        vars.get(key)
            .map(|value| filter.map_or_else(|| (*value).to_string(), |f| apply_filter(value, f)))
    })
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
                    if key.chars().all(is_placeholder_char) {
                        let Some((base_key, _)) = parse_placeholder_parts(key) else {
                            i = j + 2;
                            break;
                        };
                        let key_string = base_key.to_string();
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

    let result = render_template_with_lookup(template, |key, filter| {
        vars.get(key)
            .map(|value| filter.map_or_else(|| value.to_string(), |f| apply_filter(value, f)))
    });

    (result, missing)
}

fn render_template_with_lookup<F>(template: &str, mut value_for: F) -> String
where
    F: FnMut(&str, Option<&str>) -> Option<String>,
{
    let mut result = String::with_capacity(template.len());
    let mut i = 0;
    let mut last = 0;
    let bytes = template.as_bytes();

    while i + 1 < bytes.len() {
        if bytes[i] == b'{' && bytes[i + 1] == b'{' {
            let start = i + 2;
            let mut j = start;
            while j + 1 < bytes.len() {
                if bytes[j] == b'}' && bytes[j + 1] == b'}' {
                    let raw = template[start..j].trim();
                    if raw.chars().all(is_placeholder_char) {
                        if let Some((key, filter)) = parse_placeholder_parts(raw) {
                            if let Some(value) = value_for(key, filter) {
                                result.push_str(&template[last..i]);
                                result.push_str(&value);
                                i = j + 2;
                                last = i;
                                break;
                            }
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

    result.push_str(&template[last..]);
    result
}

fn parse_placeholder_parts(raw: &str) -> Option<(&str, Option<&str>)> {
    let mut split = raw.splitn(2, '|');
    let key = split.next()?.trim();
    if key.is_empty() || !key.chars().all(is_identifier_char) {
        return None;
    }

    let filter = split.next().map(str::trim).filter(|f| !f.is_empty());
    if let Some(f) = filter {
        if !f.chars().all(is_identifier_char) {
            return None;
        }
    }

    Some((key, filter))
}

fn is_identifier_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

fn is_placeholder_char(c: char) -> bool {
    is_identifier_char(c) || c == '|'
}

fn split_words(value: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut current = String::new();
    let mut prev_was_lowercase = false;

    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            if ch.is_ascii_uppercase() && prev_was_lowercase && !current.is_empty() {
                words.push(current.to_ascii_lowercase());
                current.clear();
            }
            current.push(ch);
            prev_was_lowercase = ch.is_ascii_lowercase();
        } else {
            if !current.is_empty() {
                words.push(current.to_ascii_lowercase());
                current.clear();
            }
            prev_was_lowercase = false;
        }
    }

    if !current.is_empty() {
        words.push(current.to_ascii_lowercase());
    }

    words
}

fn capitalize(word: &str) -> String {
    let mut output = String::with_capacity(word.len());
    for (idx, ch) in word.chars().enumerate() {
        if idx == 0 {
            output.push(ch.to_ascii_uppercase());
        } else {
            output.push(ch.to_ascii_lowercase());
        }
    }
    output
}

fn apply_filter(value: &str, filter: &str) -> String {
    let words = split_words(value);

    match filter {
        "camel" => {
            let mut out = String::new();
            for (idx, word) in words.iter().enumerate() {
                if idx == 0 {
                    out.push_str(word);
                } else {
                    out.push_str(&capitalize(word));
                }
            }
            out
        }
        "pascal" => words
            .iter()
            .map(|word| capitalize(word))
            .collect::<Vec<_>>()
            .join(""),
        "kebab" => words.join("-"),
        "snake" => words.join("_"),
        "upper" => value.to_ascii_uppercase(),
        "lower" => value.to_ascii_lowercase(),
        "title" => words
            .iter()
            .map(|word| capitalize(word))
            .collect::<Vec<_>>()
            .join(" "),
        unknown => {
            eprintln!(
                "warning: unknown template filter '{}', leaving value unchanged",
                unknown
            );
            value.to_string()
        }
    }
}

fn truncate_line(value: &str, max_chars: usize) -> String {
    let trimmed = value.trim();
    if trimmed.chars().count() <= max_chars {
        return trimmed.to_string();
    }

    let truncated: String = trimmed.chars().take(max_chars.saturating_sub(3)).collect();
    format!("{}...", truncated)
}

fn template_description(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();

    fn next_non_empty_line<'a>(lines: &[&'a str], start: usize) -> Option<&'a str> {
        lines[start..]
            .iter()
            .copied()
            .find(|line| !line.trim().is_empty())
    }

    if let Some((idx, _)) = lines
        .iter()
        .enumerate()
        .find(|(_, line)| line.trim_start().starts_with("# Mission"))
    {
        if let Some(line) = next_non_empty_line(&lines, idx + 1) {
            return truncate_line(line, 80);
        }
    }

    if let Some((idx, _)) = lines
        .iter()
        .enumerate()
        .find(|(_, line)| line.trim_start().starts_with("# Role"))
    {
        if let Some(line) = next_non_empty_line(&lines, idx + 1) {
            return truncate_line(line, 80);
        }
    }

    lines
        .iter()
        .find(|line| !line.trim().is_empty())
        .map(|line| truncate_line(line, 80))
        .unwrap_or_else(|| "No description available".to_string())
}

/// List available templates
pub fn list_templates(workspace_root: &Path) -> Vec<(String, String, Vec<String>)> {
    let mut templates: Vec<(String, String, Vec<String>)> = TEMPLATES
        .iter()
        .map(|(name, content)| {
            (
                (*name).to_string(),
                template_description(content),
                extract_placeholders(content),
            )
        })
        .collect();

    // Add custom templates
    let custom_dir = workspace_root.join(".vibeanvil/prompts");
    if custom_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(custom_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(name) = path.file_stem() {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        templates.push((
                            format!("{} (custom)", name.to_string_lossy()),
                            template_description(&content),
                            extract_placeholders(&content),
                        ));
                    }
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
    fn extracts_placeholders_with_filter_syntax() {
        let t = "{{name|camel}} and {{other}}";
        let keys = extract_placeholders(t);
        assert_eq!(keys, vec!["name".to_string(), "other".to_string()]);
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

    #[test]
    fn render_checked_applies_filter() {
        let t = "{{name|pascal}}";
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "hello world".to_string());
        let (rendered, missing) = render_checked(t, &vars);
        assert_eq!(rendered, "HelloWorld");
        assert!(missing.is_empty());
    }

    #[test]
    fn apply_filter_supports_all_cases() {
        let input = "hello_world-caseValue";
        assert_eq!(apply_filter(input, "camel"), "helloWorldCaseValue");
        assert_eq!(apply_filter(input, "pascal"), "HelloWorldCaseValue");
        assert_eq!(apply_filter(input, "kebab"), "hello-world-case-value");
        assert_eq!(apply_filter(input, "snake"), "hello_world_case_value");
        assert_eq!(apply_filter(input, "upper"), "HELLO_WORLD-CASEVALUE");
        assert_eq!(apply_filter(input, "lower"), "hello_world-casevalue");
        assert_eq!(apply_filter(input, "title"), "Hello World Case Value");
    }

    #[test]
    fn template_description_prefers_mission() {
        let template =
            "# Role\nRole text\n\n# Mission\nMission statement here\n\n# Context\n{{context}}";
        assert_eq!(template_description(template), "Mission statement here");
    }
}
