//! SLSA Provenance Module
//! Generates SLSA-style provenance metadata for builds.
//! See: https://slsa.dev/provenance/v1

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// SLSA Provenance metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provenance {
    /// Schema version
    #[serde(rename = "_type")]
    pub schema_type: String,
    /// Predicate type (SLSA Provenance)
    pub predicate_type: String,
    /// Build definition
    pub build_definition: BuildDefinition,
    /// Run details
    pub run_details: RunDetails,
}

/// Build definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildDefinition {
    /// Build type
    pub build_type: String,
    /// External parameters (e.g., contract hash)
    pub external_parameters: HashMap<String, String>,
    /// Internal parameters
    pub internal_parameters: HashMap<String, String>,
    /// Resolved dependencies
    pub resolved_dependencies: Vec<Dependency>,
}

/// Dependency reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    /// URI of the dependency
    pub uri: String,
    /// Digest (SHA256)
    pub digest: HashMap<String, String>,
}

/// Run details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunDetails {
    /// Builder info
    pub builder: Builder,
    /// Build metadata
    pub metadata: BuildMetadata,
}

/// Builder information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Builder {
    /// Builder ID (e.g., vibeanvil@version)
    pub id: String,
    /// Builder version
    pub version: Option<HashMap<String, String>>,
}

/// Build metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildMetadata {
    /// Invocation ID (session ID)
    pub invocation_id: String,
    /// Start time
    pub started_on: DateTime<Utc>,
    /// End time
    pub finished_on: Option<DateTime<Utc>>,
}

impl Provenance {
    /// Create new provenance for a build
    pub fn new(contract_hash: &str, session_id: &str) -> Self {
        let mut external_params = HashMap::new();
        external_params.insert("contract_hash".to_string(), contract_hash.to_string());

        let mut internal_params = HashMap::new();
        internal_params.insert(
            "vibeanvil_version".to_string(),
            env!("CARGO_PKG_VERSION").to_string(),
        );

        Self {
            schema_type: "https://in-toto.io/Statement/v1".to_string(),
            predicate_type: "https://slsa.dev/provenance/v1".to_string(),
            build_definition: BuildDefinition {
                build_type: "https://vibeanvil.dev/build/v1".to_string(),
                external_parameters: external_params,
                internal_parameters: internal_params,
                resolved_dependencies: vec![],
            },
            run_details: RunDetails {
                builder: Builder {
                    id: format!("vibeanvil@{}", env!("CARGO_PKG_VERSION")),
                    version: None,
                },
                metadata: BuildMetadata {
                    invocation_id: session_id.to_string(),
                    started_on: Utc::now(),
                    finished_on: None,
                },
            },
        }
    }

    /// Add a dependency
    pub fn add_dependency(&mut self, uri: &str, sha256: &str) {
        let mut digest = HashMap::new();
        digest.insert("sha256".to_string(), sha256.to_string());
        self.build_definition
            .resolved_dependencies
            .push(Dependency {
                uri: uri.to_string(),
                digest,
            });
    }

    /// Mark build as finished
    pub fn finish(&mut self) {
        self.run_details.metadata.finished_on = Some(Utc::now());
    }

    /// Serialize to JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_provenance() {
        let prov = Provenance::new("abc123", "session-456");
        assert_eq!(prov.predicate_type, "https://slsa.dev/provenance/v1");
        assert!(prov
            .build_definition
            .external_parameters
            .contains_key("contract_hash"));
    }

    #[test]
    fn test_add_dependency() {
        let mut prov = Provenance::new("abc123", "session-456");
        prov.add_dependency("file:///src/main.rs", "sha256hash");
        assert_eq!(prov.build_definition.resolved_dependencies.len(), 1);
    }

    #[test]
    fn test_to_json() {
        let prov = Provenance::new("abc123", "session-456");
        let json = prov.to_json().unwrap();
        assert!(json.contains("slsa.dev/provenance"));
    }
}
