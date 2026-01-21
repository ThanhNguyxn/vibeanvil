//! MCP Resources Implementation
//!
//! Exposes VibeAnvil project artifacts as MCP resources.
//! Resources allow AI assistants to read project files like contract, plan, state.

use std::path::PathBuf;
use tokio::fs;
use tracing::{debug, warn};

use super::protocol::*;
use crate::workspace;

/// Resource URI prefix for VibeAnvil
pub const RESOURCE_URI_PREFIX: &str = "vibeanvil://";

/// Resource type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceType {
    Contract,
    Plan,
    State,
    Constitution,
    Blueprint,
    Intake,
    RepoMap,
    Tasks,
}

impl ResourceType {
    /// Get the URI for this resource type
    pub fn uri(&self) -> String {
        format!("{}{}", RESOURCE_URI_PREFIX, self.name())
    }

    /// Get the name of this resource type
    pub fn name(&self) -> &'static str {
        match self {
            ResourceType::Contract => "contract",
            ResourceType::Plan => "plan",
            ResourceType::State => "state",
            ResourceType::Constitution => "constitution",
            ResourceType::Blueprint => "blueprint",
            ResourceType::Intake => "intake",
            ResourceType::RepoMap => "repomap",
            ResourceType::Tasks => "tasks",
        }
    }

    /// Get the description for this resource type
    pub fn description(&self) -> &'static str {
        match self {
            ResourceType::Contract => {
                "Project contract defining requirements and acceptance criteria"
            }
            ResourceType::Plan => "Implementation plan with tasks and milestones",
            ResourceType::State => "Current workflow state (JSON)",
            ResourceType::Constitution => "Project AI constitution and guidelines",
            ResourceType::Blueprint => "Technical blueprint and architecture",
            ResourceType::Intake => "Captured user requirements",
            ResourceType::RepoMap => "Repository structure and file map",
            ResourceType::Tasks => "Task breakdown and progress",
        }
    }

    /// Get the MIME type for this resource type
    pub fn mime_type(&self) -> &'static str {
        match self {
            ResourceType::State => "application/json",
            _ => "text/markdown",
        }
    }

    /// Get the file path for this resource type
    pub fn file_path(&self) -> PathBuf {
        let anvil_dir = workspace::workspace_path();
        match self {
            ResourceType::Contract => anvil_dir.join("contract.md"),
            ResourceType::Plan => anvil_dir.join("plan.md"),
            ResourceType::State => anvil_dir.join("state.json"),
            ResourceType::Constitution => anvil_dir.join("constitution.md"),
            ResourceType::Blueprint => anvil_dir.join("blueprint.md"),
            ResourceType::Intake => anvil_dir.join("intake.md"),
            ResourceType::RepoMap => anvil_dir.join("repomap.md"),
            ResourceType::Tasks => anvil_dir.join("tasks.md"),
        }
    }

    /// Parse resource type from URI
    pub fn from_uri(uri: &str) -> Option<Self> {
        let name = uri.strip_prefix(RESOURCE_URI_PREFIX)?;
        Self::from_name(name)
    }

    /// Parse resource type from name
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "contract" => Some(ResourceType::Contract),
            "plan" => Some(ResourceType::Plan),
            "state" => Some(ResourceType::State),
            "constitution" => Some(ResourceType::Constitution),
            "blueprint" => Some(ResourceType::Blueprint),
            "intake" => Some(ResourceType::Intake),
            "repomap" => Some(ResourceType::RepoMap),
            "tasks" => Some(ResourceType::Tasks),
            _ => None,
        }
    }

    /// Get all resource types
    pub fn all() -> &'static [ResourceType] {
        &[
            ResourceType::Contract,
            ResourceType::Plan,
            ResourceType::State,
            ResourceType::Constitution,
            ResourceType::Blueprint,
            ResourceType::Intake,
            ResourceType::RepoMap,
            ResourceType::Tasks,
        ]
    }
}

/// Resource registry
pub struct ResourceRegistry;

impl ResourceRegistry {
    /// List all available resources
    pub async fn list_resources() -> Vec<ResourceDefinition> {
        let mut resources = Vec::new();

        for resource_type in ResourceType::all() {
            let path = resource_type.file_path();

            // Only include resources that exist
            if path.exists() {
                resources.push(ResourceDefinition {
                    uri: resource_type.uri(),
                    name: resource_type.name().to_string(),
                    description: Some(resource_type.description().to_string()),
                    mime_type: Some(resource_type.mime_type().to_string()),
                });
            }
        }

        // Always include state (even if not exists, we can generate it)
        if !resources.iter().any(|r| r.uri == ResourceType::State.uri()) {
            resources.push(ResourceDefinition {
                uri: ResourceType::State.uri(),
                name: ResourceType::State.name().to_string(),
                description: Some(ResourceType::State.description().to_string()),
                mime_type: Some(ResourceType::State.mime_type().to_string()),
            });
        }

        resources
    }

    /// Read a resource by URI
    pub async fn read_resource(uri: &str) -> Result<ResourceContents, String> {
        let resource_type =
            ResourceType::from_uri(uri).ok_or_else(|| format!("Unknown resource URI: {}", uri))?;

        let path = resource_type.file_path();
        debug!("Reading resource: {} from {:?}", uri, path);

        // Special handling for state - always try to read from workspace
        if resource_type == ResourceType::State {
            return Self::read_state_resource().await;
        }

        // Read file content
        if !path.exists() {
            return Err(format!("Resource not found: {}", uri));
        }

        let content = fs::read_to_string(&path)
            .await
            .map_err(|e| format!("Failed to read resource: {}", e))?;

        Ok(ResourceContents {
            uri: uri.to_string(),
            mime_type: Some(resource_type.mime_type().to_string()),
            text: Some(content),
            blob: None,
        })
    }

    /// Read state resource
    async fn read_state_resource() -> Result<ResourceContents, String> {
        match crate::workspace::load_state().await {
            Ok(state) => {
                let json = serde_json::to_string_pretty(&state)
                    .map_err(|e| format!("Failed to serialize state: {}", e))?;
                Ok(ResourceContents {
                    uri: ResourceType::State.uri(),
                    mime_type: Some("application/json".to_string()),
                    text: Some(json),
                    blob: None,
                })
            }
            Err(e) => {
                warn!("Failed to load state: {}", e);
                // Return default state
                let default_state = crate::state::StateData::default();
                let json = serde_json::to_string_pretty(&default_state)
                    .map_err(|e| format!("Failed to serialize default state: {}", e))?;
                Ok(ResourceContents {
                    uri: ResourceType::State.uri(),
                    mime_type: Some("application/json".to_string()),
                    text: Some(json),
                    blob: None,
                })
            }
        }
    }
}

/// Resource contents returned from read
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceContents {
    pub uri: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blob: Option<String>,
}

/// Resource read result
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ResourceReadResult {
    pub contents: Vec<ResourceContents>,
}

/// Resource read params
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ResourceReadParams {
    pub uri: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_type_from_uri() {
        assert_eq!(
            ResourceType::from_uri("vibeanvil://contract"),
            Some(ResourceType::Contract)
        );
        assert_eq!(
            ResourceType::from_uri("vibeanvil://plan"),
            Some(ResourceType::Plan)
        );
        assert_eq!(
            ResourceType::from_uri("vibeanvil://state"),
            Some(ResourceType::State)
        );
        assert_eq!(ResourceType::from_uri("invalid://foo"), None);
    }

    #[test]
    fn test_resource_type_uri() {
        assert_eq!(ResourceType::Contract.uri(), "vibeanvil://contract");
        assert_eq!(ResourceType::Plan.uri(), "vibeanvil://plan");
        assert_eq!(ResourceType::State.uri(), "vibeanvil://state");
    }

    #[test]
    fn test_resource_type_mime_type() {
        assert_eq!(ResourceType::Contract.mime_type(), "text/markdown");
        assert_eq!(ResourceType::State.mime_type(), "application/json");
    }

    #[test]
    fn test_all_resource_types() {
        let all = ResourceType::all();
        assert!(all.len() >= 8);
        assert!(all.contains(&ResourceType::Contract));
        assert!(all.contains(&ResourceType::Plan));
        assert!(all.contains(&ResourceType::State));
    }
}
