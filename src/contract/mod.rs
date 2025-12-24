//! Contract validation, locking, and management

pub mod schema;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use tokio::fs;

use crate::workspace;

/// Contract status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ContractStatus {
    Draft,
    Locked,
}

/// A contract definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contract {
    /// Contract schema version
    pub schema_version: String,
    /// Contract status
    pub status: ContractStatus,
    /// Project name
    pub project_name: String,
    /// Project description
    pub description: String,
    /// Goals/objectives
    pub goals: Vec<String>,
    /// Requirements
    pub requirements: Vec<Requirement>,
    /// Acceptance criteria
    pub acceptance_criteria: Vec<String>,
    /// Constraints
    pub constraints: Vec<String>,
    /// Out of scope items
    pub out_of_scope: Vec<String>,
    /// Timestamp of creation
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Timestamp of last update
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// A requirement entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Requirement {
    /// Requirement ID
    pub id: String,
    /// Requirement description
    pub description: String,
    /// Priority (must, should, could)
    pub priority: Priority,
}

/// Requirement priority
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    Must,
    Should,
    Could,
}

/// Contract lock information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractLock {
    /// SHA-256 hash of the contract
    pub hash: String,
    /// Timestamp when locked
    pub locked_at: chrono::DateTime<chrono::Utc>,
    /// Tool version that created the lock
    pub tool_version: String,
    /// Schema version of the contract
    pub schema_version: String,
}

impl Default for Contract {
    fn default() -> Self {
        let now = chrono::Utc::now();
        Self {
            schema_version: "1.0.0".to_string(),
            status: ContractStatus::Draft,
            project_name: String::new(),
            description: String::new(),
            goals: vec![],
            requirements: vec![],
            acceptance_criteria: vec![],
            constraints: vec![],
            out_of_scope: vec![],
            created_at: now,
            updated_at: now,
        }
    }
}

impl Contract {
    /// Create a new contract with project name
    pub fn new(project_name: &str) -> Self {
        Self {
            project_name: project_name.to_string(),
            ..Default::default()
        }
    }

    /// Add a requirement
    pub fn add_requirement(&mut self, id: &str, description: &str, priority: Priority) {
        self.requirements.push(Requirement {
            id: id.to_string(),
            description: description.to_string(),
            priority,
        });
        self.updated_at = chrono::Utc::now();
    }

    /// Add a goal
    pub fn add_goal(&mut self, goal: &str) {
        self.goals.push(goal.to_string());
        self.updated_at = chrono::Utc::now();
    }

    /// Add acceptance criteria
    pub fn add_acceptance_criterion(&mut self, criterion: &str) {
        self.acceptance_criteria.push(criterion.to_string());
        self.updated_at = chrono::Utc::now();
    }

    /// Validate the contract
    pub fn validate(&self) -> ContractValidation {
        let mut errors = vec![];
        let mut warnings = vec![];

        if self.project_name.is_empty() {
            errors.push("Project name is required".to_string());
        }

        if self.description.is_empty() {
            errors.push("Description is required".to_string());
        }

        if self.goals.is_empty() {
            errors.push("At least one goal is required".to_string());
        }

        if self.requirements.is_empty() {
            warnings.push("No requirements defined".to_string());
        }

        if self.acceptance_criteria.is_empty() {
            warnings.push("No acceptance criteria defined".to_string());
        }

        ContractValidation {
            valid: errors.is_empty(),
            errors,
            warnings,
        }
    }

    /// Generate canonical JSON for hashing
    pub fn canonical_json(&self) -> Result<String> {
        // Sort keys and format consistently
        serde_json::to_string(self).context("Failed to serialize contract to JSON")
    }

    /// Generate SHA-256 hash for locking
    pub fn generate_hash(&self, tool_version: &str) -> Result<String> {
        let canonical = self.canonical_json()?;
        let mut hasher = Sha256::new();
        hasher.update(canonical.as_bytes());
        hasher.update(tool_version.as_bytes());
        hasher.update(self.schema_version.as_bytes());
        Ok(hex::encode(hasher.finalize()))
    }

    /// Lock the contract
    pub fn lock(&mut self, tool_version: &str) -> Result<ContractLock> {
        let validation = self.validate();
        if !validation.valid {
            anyhow::bail!("Cannot lock invalid contract: {:?}", validation.errors);
        }

        let hash = self.generate_hash(tool_version)?;
        self.status = ContractStatus::Locked;
        self.updated_at = chrono::Utc::now();

        Ok(ContractLock {
            hash,
            locked_at: chrono::Utc::now(),
            tool_version: tool_version.to_string(),
            schema_version: self.schema_version.clone(),
        })
    }

    /// Check if contract is locked
    pub fn is_locked(&self) -> bool {
        self.status == ContractStatus::Locked
    }
}

/// Contract validation result
#[derive(Debug, Clone)]
pub struct ContractValidation {
    /// Whether the contract is valid
    pub valid: bool,
    /// Validation errors
    pub errors: Vec<String>,
    /// Validation warnings
    pub warnings: Vec<String>,
}

/// Get path to contract.json
pub fn contract_path() -> PathBuf {
    workspace::contracts_path().join("contract.json")
}

/// Get path to contract.lock
pub fn contract_lock_path() -> PathBuf {
    workspace::workspace_path().join("contract.lock")
}

/// Load contract from file
pub async fn load_contract() -> Result<Contract> {
    let path = contract_path();
    if !path.exists() {
        anyhow::bail!("No contract found. Create one with 'vibeanvil contract create'");
    }

    let content = fs::read_to_string(&path)
        .await
        .context("Failed to read contract file")?;

    serde_json::from_str(&content).context("Failed to parse contract.json")
}

/// Save contract to file
pub async fn save_contract(contract: &Contract) -> Result<()> {
    let path = contract_path();
    fs::create_dir_all(path.parent().unwrap()).await?;

    let content = serde_json::to_string_pretty(contract)?;
    fs::write(&path, content)
        .await
        .context("Failed to write contract.json")
}

/// Save contract lock file
pub async fn save_lock(lock: &ContractLock) -> Result<()> {
    let path = contract_lock_path();
    let content = serde_json::to_string_pretty(lock)?;
    fs::write(&path, content)
        .await
        .context("Failed to write contract.lock")
}

/// Load contract lock file
pub async fn load_lock() -> Result<ContractLock> {
    let path = contract_lock_path();
    if !path.exists() {
        anyhow::bail!("Contract is not locked");
    }

    let content = fs::read_to_string(&path)
        .await
        .context("Failed to read contract.lock")?;

    serde_json::from_str(&content).context("Failed to parse contract.lock")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contract_validation() {
        let mut contract = Contract::new("Test Project");
        let validation = contract.validate();
        assert!(!validation.valid);

        contract.description = "A test project".to_string();
        contract.add_goal("Complete the test");
        let validation = contract.validate();
        assert!(validation.valid);
    }

    #[test]
    fn test_contract_hash() {
        let mut contract = Contract::new("Test");
        contract.description = "Test description".to_string();
        contract.add_goal("Goal 1");

        let hash1 = contract.generate_hash("1.0.0").unwrap();
        let hash2 = contract.generate_hash("1.0.0").unwrap();
        assert_eq!(hash1, hash2);

        let hash3 = contract.generate_hash("1.0.1").unwrap();
        assert_ne!(hash1, hash3);
    }
}
