//! Workflow Library Module
//! Provides reusable contract and plan templates.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A workflow template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowTemplate {
    /// Template name
    pub name: String,
    /// Description
    pub description: String,
    /// Category (e.g., "web", "cli", "api")
    pub category: String,
    /// Tags for search
    pub tags: Vec<String>,
    /// Template content (contract or plan markdown)
    pub content: String,
    /// Variables that can be substituted
    pub variables: Vec<String>,
}

/// Built-in workflow templates
pub const TEMPLATES: &[(&str, &str, &str, &str)] = &[
    // (name, category, description, content)
    (
        "web-app",
        "web",
        "Standard web application with auth and CRUD",
        r#"# Web Application Contract

## Requirements
- User authentication (login/register)
- CRUD operations for {{resource}}
- Responsive UI

## Acceptance Criteria
- [ ] Users can register and login
- [ ] {{resource}} can be created, read, updated, deleted
- [ ] Works on mobile and desktop
"#,
    ),
    (
        "cli-tool",
        "cli",
        "Command-line tool with subcommands",
        r#"# CLI Tool Contract

## Requirements
- Main command: {{name}}
- Subcommands: {{subcommands}}
- Help and version flags

## Acceptance Criteria
- [ ] `{{name}} --help` shows usage
- [ ] `{{name}} --version` shows version
- [ ] Each subcommand works correctly
"#,
    ),
    (
        "api-service",
        "api",
        "REST API with authentication",
        r#"# API Service Contract

## Requirements
- REST endpoints for {{resource}}
- JWT authentication
- Input validation

## Acceptance Criteria
- [ ] GET /{{resource}} returns list
- [ ] POST /{{resource}} creates item
- [ ] PUT /{{resource}}/:id updates item
- [ ] DELETE /{{resource}}/:id removes item
"#,
    ),
    (
        "library",
        "lib",
        "Reusable library with public API",
        r#"# Library Contract

## Requirements
- Public API: {{api_functions}}
- Documentation for all public items
- Unit tests with >80% coverage

## Acceptance Criteria
- [ ] All public functions documented
- [ ] Tests pass
- [ ] No clippy warnings
"#,
    ),
];

/// Get a template by name
pub fn get_template(name: &str) -> Option<WorkflowTemplate> {
    for (tmpl_name, category, description, content) in TEMPLATES {
        if *tmpl_name == name {
            // Extract variables from {{variable}} patterns
            let vars: Vec<String> = content
                .match_indices("{{")
                .filter_map(|(start, _)| {
                    let rest = &content[start + 2..];
                    rest.find("}}").map(|end| rest[..end].to_string())
                })
                .collect();

            return Some(WorkflowTemplate {
                name: tmpl_name.to_string(),
                description: description.to_string(),
                category: category.to_string(),
                tags: vec![category.to_string()],
                content: content.to_string(),
                variables: vars,
            });
        }
    }
    None
}

/// List all available templates
pub fn list_templates() -> Vec<WorkflowTemplate> {
    TEMPLATES
        .iter()
        .filter_map(|(name, _, _, _)| get_template(name))
        .collect()
}

/// Render a template with variable substitution
pub fn render_template(template: &WorkflowTemplate, vars: &HashMap<String, String>) -> String {
    let mut result = template.content.clone();
    for (key, value) in vars {
        result = result.replace(&format!("{{{{{}}}}}", key), value);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_template() {
        let tmpl = get_template("web-app").unwrap();
        assert_eq!(tmpl.name, "web-app");
        assert!(tmpl.variables.contains(&"resource".to_string()));
    }

    #[test]
    fn test_list_templates() {
        let templates = list_templates();
        assert!(templates.len() >= 4);
    }

    #[test]
    fn test_render_template() {
        let tmpl = get_template("cli-tool").unwrap();
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "mytool".to_string());
        vars.insert("subcommands".to_string(), "init, build, test".to_string());
        let rendered = render_template(&tmpl, &vars);
        assert!(rendered.contains("mytool"));
        assert!(rendered.contains("init, build, test"));
    }
}
