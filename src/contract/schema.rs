//! Contract JSON schema definition

use serde::{Deserialize, Serialize};

/// JSON Schema for contract validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractSchema {
    #[serde(rename = "$schema")]
    pub schema: String,
    #[serde(rename = "$id")]
    pub id: String,
    pub title: String,
    pub description: String,
    #[serde(rename = "type")]
    pub schema_type: String,
    pub required: Vec<String>,
    pub properties: SchemaProperties,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaProperties {
    pub schema_version: SchemaProperty,
    pub status: SchemaProperty,
    pub project_name: SchemaProperty,
    pub description: SchemaProperty,
    pub goals: SchemaArrayProperty,
    pub requirements: SchemaArrayProperty,
    pub acceptance_criteria: SchemaArrayProperty,
    pub constraints: SchemaArrayProperty,
    pub out_of_scope: SchemaArrayProperty,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaProperty {
    #[serde(rename = "type")]
    pub property_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "enum", skip_serializing_if = "Option::is_none")]
    pub enum_values: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaArrayProperty {
    #[serde(rename = "type")]
    pub property_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<SchemaItems>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaItems {
    #[serde(rename = "type")]
    pub item_type: Option<String>,
    #[serde(rename = "$ref")]
    pub reference: Option<String>,
}

impl Default for ContractSchema {
    fn default() -> Self {
        Self {
            schema: "https://json-schema.org/draft/2020-12/schema".to_string(),
            id: "https://vibeanvil.dev/schemas/contract.json".to_string(),
            title: "VibeAnvil Contract".to_string(),
            description: "Contract schema for vibeanvil workflow".to_string(),
            schema_type: "object".to_string(),
            required: vec![
                "schema_version".to_string(),
                "status".to_string(),
                "project_name".to_string(),
                "description".to_string(),
                "goals".to_string(),
            ],
            properties: SchemaProperties {
                schema_version: SchemaProperty {
                    property_type: "string".to_string(),
                    description: Some("Schema version".to_string()),
                    enum_values: None,
                },
                status: SchemaProperty {
                    property_type: "string".to_string(),
                    description: Some("Contract status".to_string()),
                    enum_values: Some(vec!["DRAFT".to_string(), "LOCKED".to_string()]),
                },
                project_name: SchemaProperty {
                    property_type: "string".to_string(),
                    description: Some("Project name".to_string()),
                    enum_values: None,
                },
                description: SchemaProperty {
                    property_type: "string".to_string(),
                    description: Some("Project description".to_string()),
                    enum_values: None,
                },
                goals: SchemaArrayProperty {
                    property_type: "array".to_string(),
                    description: Some("Project goals".to_string()),
                    items: Some(SchemaItems {
                        item_type: Some("string".to_string()),
                        reference: None,
                    }),
                },
                requirements: SchemaArrayProperty {
                    property_type: "array".to_string(),
                    description: Some("Project requirements".to_string()),
                    items: Some(SchemaItems {
                        item_type: None,
                        reference: Some("#/$defs/Requirement".to_string()),
                    }),
                },
                acceptance_criteria: SchemaArrayProperty {
                    property_type: "array".to_string(),
                    description: Some("Acceptance criteria".to_string()),
                    items: Some(SchemaItems {
                        item_type: Some("string".to_string()),
                        reference: None,
                    }),
                },
                constraints: SchemaArrayProperty {
                    property_type: "array".to_string(),
                    description: Some("Project constraints".to_string()),
                    items: Some(SchemaItems {
                        item_type: Some("string".to_string()),
                        reference: None,
                    }),
                },
                out_of_scope: SchemaArrayProperty {
                    property_type: "array".to_string(),
                    description: Some("Out of scope items".to_string()),
                    items: Some(SchemaItems {
                        item_type: Some("string".to_string()),
                        reference: None,
                    }),
                },
            },
        }
    }
}

/// Generate the JSON schema as a string
pub fn generate_schema() -> String {
    let schema = ContractSchema::default();
    serde_json::to_string_pretty(&schema).unwrap_or_default()
}
