//! State machine for the vibeanvil workflow

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::fmt;

/// All possible workflow states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum State {
    /// Initial state - workspace just created
    #[default]
    Init,
    /// Requirements/intake has been captured
    IntakeCaptured,
    /// Blueprint has been drafted
    BlueprintDrafted,
    /// Contract has been drafted (not yet locked)
    ContractDrafted,
    /// Contract is locked and immutable
    ContractLocked,
    /// Implementation plan has been created
    PlanCreated,
    /// Build is currently in progress
    BuildInProgress,
    /// Build has completed
    BuildDone,
    /// Review has passed
    ReviewPassed,
    /// Review has failed
    ReviewFailed,
    /// Project has been shipped
    Shipped,
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            State::Init => write!(f, "INIT"),
            State::IntakeCaptured => write!(f, "INTAKE_CAPTURED"),
            State::BlueprintDrafted => write!(f, "BLUEPRINT_DRAFTED"),
            State::ContractDrafted => write!(f, "CONTRACT_DRAFTED"),
            State::ContractLocked => write!(f, "CONTRACT_LOCKED"),
            State::PlanCreated => write!(f, "PLAN_CREATED"),
            State::BuildInProgress => write!(f, "BUILD_IN_PROGRESS"),
            State::BuildDone => write!(f, "BUILD_DONE"),
            State::ReviewPassed => write!(f, "REVIEW_PASSED"),
            State::ReviewFailed => write!(f, "REVIEW_FAILED"),
            State::Shipped => write!(f, "SHIPPED"),
        }
    }
}

impl State {
    /// Get the ordinal value of this state (for ordering)
    pub fn ordinal(&self) -> u8 {
        match self {
            State::Init => 0,
            State::IntakeCaptured => 1,
            State::BlueprintDrafted => 2,
            State::ContractDrafted => 3,
            State::ContractLocked => 4,
            State::PlanCreated => 5,
            State::BuildInProgress => 6,
            State::BuildDone => 7,
            State::ReviewPassed => 8,
            State::ReviewFailed => 8, // Same level as ReviewPassed
            State::Shipped => 9,
        }
    }

    /// Check if this state is at or past the target state
    pub fn is_at_least(&self, target: State) -> bool {
        self.ordinal() >= target.ordinal()
    }

    /// Get all valid transitions from this state
    pub fn valid_transitions(&self) -> Vec<State> {
        match self {
            State::Init => vec![State::IntakeCaptured],
            State::IntakeCaptured => vec![State::BlueprintDrafted],
            State::BlueprintDrafted => vec![State::ContractDrafted],
            State::ContractDrafted => vec![State::ContractLocked],
            State::ContractLocked => vec![State::PlanCreated],
            State::PlanCreated => vec![State::BuildInProgress],
            State::BuildInProgress => vec![State::BuildDone],
            State::BuildDone => vec![State::ReviewPassed, State::ReviewFailed],
            State::ReviewPassed => vec![State::Shipped],
            State::ReviewFailed => vec![State::BuildInProgress], // Can retry build
            State::Shipped => vec![],                            // Terminal state
        }
    }

    /// Check if transition to target state is valid
    pub fn can_transition_to(&self, target: State) -> bool {
        self.valid_transitions().contains(&target)
    }
}

/// State history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateHistoryEntry {
    /// Previous state
    pub from_state: State,
    /// New state
    pub to_state: State,
    /// Timestamp of transition
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Action that triggered the transition
    pub action: String,
    /// Session ID active during transition
    pub session_id: String,
}

/// Complete state data persisted to state.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateData {
    /// Tool version
    pub tool_version: String,
    /// Schema version for state data
    pub schema_version: String,
    /// Current state
    pub current_state: State,
    /// Session ID of last action
    pub current_session_id: Option<String>,
    /// Spec hash if contract is locked
    pub spec_hash: Option<String>,
    /// State transition history
    pub history: Vec<StateHistoryEntry>,
    /// Timestamp of last update
    pub updated_at: chrono::DateTime<chrono::Utc>,
    /// Timestamp of creation
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl Default for StateData {
    fn default() -> Self {
        let now = chrono::Utc::now();
        Self {
            tool_version: env!("CARGO_PKG_VERSION").to_string(),
            schema_version: "1.0.0".to_string(),
            current_state: State::Init,
            current_session_id: None,
            spec_hash: None,
            history: vec![],
            updated_at: now,
            created_at: now,
        }
    }
}

impl StateData {
    /// Attempt to transition to a new state
    pub fn transition_to(
        &mut self,
        new_state: State,
        action: &str,
        session_id: &str,
    ) -> Result<()> {
        if !self.current_state.can_transition_to(new_state) {
            return Err(anyhow!(
                "Invalid transition: {} → {}. Allowed: {:?}",
                self.current_state,
                new_state,
                self.current_state.valid_transitions()
            ));
        }

        let entry = StateHistoryEntry {
            from_state: self.current_state,
            to_state: new_state,
            timestamp: chrono::Utc::now(),
            action: action.to_string(),
            session_id: session_id.to_string(),
        };

        self.history.push(entry);
        self.current_state = new_state;
        self.current_session_id = Some(session_id.to_string());
        self.updated_at = chrono::Utc::now();

        Ok(())
    }

    /// Get the last N history entries
    pub fn recent_history(&self, n: usize) -> &[StateHistoryEntry] {
        let start = self.history.len().saturating_sub(n);
        &self.history[start..]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_transitions() {
        assert!(State::Init.can_transition_to(State::IntakeCaptured));
        assert!(!State::Init.can_transition_to(State::Shipped));
    }

    #[test]
    fn test_state_ordering() {
        assert!(State::ContractLocked.is_at_least(State::Init));
        assert!(!State::Init.is_at_least(State::ContractLocked));
    }

    #[test]
    fn test_transition() {
        let mut state = StateData::default();
        assert!(state
            .transition_to(State::IntakeCaptured, "intake", "session-1")
            .is_ok());
        assert_eq!(state.current_state, State::IntakeCaptured);
        assert_eq!(state.history.len(), 1);
    }

    #[test]
    fn test_invalid_transition() {
        let mut state = StateData::default();
        assert!(state
            .transition_to(State::Shipped, "skip", "session-1")
            .is_err());
    }
}
