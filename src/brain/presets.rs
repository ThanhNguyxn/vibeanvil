//! Harvest presets loader
//!
//! Loads preset configurations from brainpacks/presets.yaml

use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;

/// Embedded presets YAML
const PRESETS_YAML: &str = include_str!("../../brainpacks/presets.yaml");

/// Preset defaults
#[derive(Debug, Clone, Deserialize, Default)]
pub struct PresetDefaults {
    #[serde(default = "default_min_stars")]
    pub min_stars: u32,
    #[serde(default = "default_updated_within_days")]
    pub updated_within_days: u32,
    #[serde(default = "default_max_repos")]
    pub max_repos: usize,
    #[serde(default)]
    pub download: String,
    #[serde(default)]
    pub ignore_globs: Vec<String>,
}

fn default_min_stars() -> u32 { 50 }
fn default_updated_within_days() -> u32 { 365 }
fn default_max_repos() -> usize { 20 }

/// A single preset configuration
#[derive(Debug, Clone, Deserialize)]
pub struct Preset {
    pub name: String,
    pub purpose: String,
    #[serde(default)]
    pub signals: Vec<String>,
    #[serde(default)]
    pub queries: Vec<String>,
    #[serde(default)]
    pub filters: PresetFilters,
    #[serde(default)]
    pub allow_globs: Vec<String>,
}

/// Subset of filters that can be overridden per-preset
#[derive(Debug, Clone, Deserialize, Default)]
pub struct PresetFilters {
    pub min_stars: Option<u32>,
    pub updated_within_days: Option<u32>,
    #[serde(default)]
    pub languages: Vec<String>,
}

/// Root structure of presets.yaml
#[derive(Debug, Clone, Deserialize)]
pub struct PresetsFile {
    #[serde(default)]
    pub defaults: PresetDefaults,
    #[serde(default)]
    pub presets: HashMap<String, Preset>,
}

impl PresetsFile {
    /// Load presets from embedded YAML
    pub fn load() -> Result<Self> {
        serde_yaml::from_str(PRESETS_YAML).context("Failed to parse presets.yaml")
    }

    /// Get a preset by name
    pub fn get(&self, name: &str) -> Option<&Preset> {
        self.presets.get(name)
    }

    /// List all preset names with their descriptions
    pub fn list(&self) -> Vec<(&str, &str, &str)> {
        let mut items: Vec<_> = self.presets
            .iter()
            .map(|(key, preset)| (key.as_str(), preset.name.as_str(), preset.purpose.as_str()))
            .collect();
        items.sort_by(|a, b| a.0.cmp(b.0));
        items
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_presets() {
        let presets = PresetsFile::load().expect("Failed to load presets");
        assert!(!presets.presets.is_empty());
        assert!(presets.get("cli_framework_patterns").is_some());
    }

    #[test]
    fn test_list_presets() {
        let presets = PresetsFile::load().expect("Failed to load presets");
        let list = presets.list();
        assert!(!list.is_empty());
    }
}
