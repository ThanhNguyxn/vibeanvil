//! Prompt command handler

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::Path;

use crate::cli::PromptKind;
use crate::prompt;

pub async fn run(
    kind: Option<PromptKind>,
    list: bool,
    render: bool,
    strict_vars: bool,
    vars: Vec<String>,
) -> Result<()> {
    if list {
        let cwd = std::env::current_dir()?;
        let templates = prompt::list_templates(&cwd);
        let name_width = templates
            .iter()
            .map(|(name, _, _)| name.len())
            .max()
            .unwrap_or(0);

        for (name, description, required_vars) in templates {
            let vars = format!("[{}]", required_vars.join(", "));
            println!("{name:<width$} - {description} {vars}", width = name_width);
        }
        return Ok(());
    }

    let kind = kind.context("Prompt kind is required unless --list is set")?;
    let template_name = match kind {
        PromptKind::Install => "install-vibeanvil",
        PromptKind::Architect => "architect",
        PromptKind::Developer => "developer",
        PromptKind::Qa => "qa",
        PromptKind::Plan => "plan",
        PromptKind::Review => "review",
        PromptKind::Commit => "commit",
        PromptKind::Debug => "debug",
        PromptKind::Xray => "xray",
        PromptKind::Vision => "vision",
        PromptKind::Security => "security",
        PromptKind::Migrate => "migrate",
        PromptKind::Refactor => "refactor",
    };

    let content = prompt::load_template(template_name)?;

    if !render {
        println!("{}", content);
        return Ok(());
    }

    let mut runtime_vars = parse_kv_vars(&vars)?;
    inject_workspace_defaults(&mut runtime_vars);

    let (rendered, missing) = prompt::render_checked(&content, &runtime_vars);
    if !missing.is_empty() {
        let keys = missing.join(", ");
        if strict_vars {
            anyhow::bail!(
                "Missing required template variables for '{}': {}. Provide with --var key=value",
                template_name,
                keys
            );
        }
        eprintln!(
            "warning: missing template variables for '{}': {} (use --strict-vars to fail)",
            template_name, keys
        );
    }

    println!("{}", rendered);

    Ok(())
}

fn parse_kv_vars(entries: &[String]) -> Result<HashMap<String, String>> {
    let mut vars = HashMap::new();
    for entry in entries {
        let Some((k, v)) = entry.split_once('=') else {
            anyhow::bail!("Invalid --var '{}'. Expected key=value", entry);
        };
        let key = k.trim();
        let value = v.trim();
        if key.is_empty() {
            anyhow::bail!("Invalid --var '{}'. Key cannot be empty", entry);
        }
        vars.insert(key.to_string(), value.to_string());
    }
    Ok(vars)
}

fn inject_workspace_defaults(vars: &mut HashMap<String, String>) {
    let intake_path = Path::new(".vibeanvil/intake.md");
    if let Ok(intake) = std::fs::read_to_string(intake_path) {
        let trimmed = intake.trim();
        if !trimmed.is_empty() {
            vars.entry("context".to_string())
                .or_insert_with(|| trimmed.to_string());
            vars.entry("description".to_string())
                .or_insert_with(|| trimmed.to_string());
        }
    }

    vars.entry("tech_stack".to_string())
        .or_insert_with(String::new);
}
