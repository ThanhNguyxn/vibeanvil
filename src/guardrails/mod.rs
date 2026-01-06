//! Guardrails module - Change Gate with A/B/C risk classification
//!
//! Provides tool-enforced gates for code changes based on risk level:
//! - Level A: Safe/cosmetic (docs, comments) - auto-apply
//! - Level B: Logic changes (refactors, tests) - require approval
//! - Level C: High-impact (public API, deps, security) - require impact analysis

pub mod capsule;
pub mod classifier;
pub mod gate;

use serde::{Deserialize, Serialize};

/// Risk level for a change
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum RiskLevel {
    /// Safe/cosmetic changes (docs, comments, typos)
    A,
    /// Logic changes (refactors, renames, tests)
    B,
    /// High-impact changes (public API, deps, security)
    C,
}

impl RiskLevel {
    /// Check if this level requires explicit approval
    pub fn requires_approval(&self, auto_approve_a: bool) -> bool {
        match self {
            RiskLevel::A => !auto_approve_a,
            RiskLevel::B => true,
            RiskLevel::C => true,
        }
    }

    /// Check if this level requires impact analysis
    pub fn requires_impact_analysis(&self) -> bool {
        matches!(self, RiskLevel::C)
    }

    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            RiskLevel::A => "A (Safe)",
            RiskLevel::B => "B (Logic)",
            RiskLevel::C => "C (High-Impact)",
        }
    }
}

impl std::fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// Guardrails configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardrailsConfig {
    /// Whether guardrails are enabled (default: false)
    #[serde(default)]
    pub enabled: bool,

    /// Guardrails mode
    #[serde(default)]
    pub mode: GuardrailsMode,

    /// Auto-approve Level A changes in normal mode
    #[serde(default = "default_true")]
    pub auto_approve_level_a: bool,

    /// Require impact analysis for Level C
    #[serde(default = "default_true")]
    pub require_impact_for_c: bool,

    /// Require alternatives for Level C
    #[serde(default)]
    pub require_alternatives_for_c: bool,
}

fn default_true() -> bool {
    true
}

impl Default for GuardrailsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            mode: GuardrailsMode::Normal,
            auto_approve_level_a: true,
            require_impact_for_c: true,
            require_alternatives_for_c: false,
        }
    }
}

/// Guardrails operating mode
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GuardrailsMode {
    /// Guardrails disabled
    Off,
    /// Normal mode: A can auto-apply, B/C require approval
    #[default]
    Normal,
    /// Strict mode: All levels require approval + complete metadata
    Strict,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_risk_level_approval() {
        assert!(!RiskLevel::A.requires_approval(true));
        assert!(RiskLevel::A.requires_approval(false));
        assert!(RiskLevel::B.requires_approval(true));
        assert!(RiskLevel::C.requires_approval(true));
    }

    #[test]
    fn test_risk_level_impact() {
        assert!(!RiskLevel::A.requires_impact_analysis());
        assert!(!RiskLevel::B.requires_impact_analysis());
        assert!(RiskLevel::C.requires_impact_analysis());
    }

    #[test]
    fn test_config_defaults() {
        let config = GuardrailsConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.mode, GuardrailsMode::Normal);
        assert!(config.auto_approve_level_a);
    }
}
