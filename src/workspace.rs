//! Workspace management for .vibeanvil directory

use anyhow::{Context, Result};
use std::path::PathBuf;
use tokio::fs;

use crate::state::StateData;

/// Workspace directory name
pub const WORKSPACE_DIR: &str = ".vibeanvil";

/// Get the workspace path from current directory
pub fn workspace_path() -> PathBuf {
    PathBuf::from(WORKSPACE_DIR)
}

/// Check if workspace exists
pub async fn workspace_exists() -> bool {
    fs::metadata(workspace_path()).await.is_ok()
}

/// Initialize the workspace directory structure
pub async fn init_workspace(force: bool) -> Result<()> {
    let ws = workspace_path();

    if ws.exists() && !force {
        anyhow::bail!("Workspace already exists. Use --force to reinitialize.");
    }

    // Create directory structure
    let dirs = [
        ws.clone(),
        ws.join("logs"),
        ws.join("sessions"),
        ws.join("contracts"),
        ws.join("blueprints"),
        ws.join("cache"),
    ];

    for dir in &dirs {
        fs::create_dir_all(dir)
            .await
            .with_context(|| format!("Failed to create directory: {}", dir.display()))?;
    }

    // Initialize state.json
    let state = StateData::default();
    save_state(&state).await?;

    // Create .gitignore for sensitive files
    let gitignore_content = r#"# VibeAnvil workspace
logs/
sessions/*/evidence/
cache/
*.lock
"#;
    fs::write(ws.join(".gitignore"), gitignore_content).await?;

    Ok(())
}

/// Get path to state.json
pub fn state_path() -> PathBuf {
    workspace_path().join("state.json")
}

/// Get path to logs directory
pub fn logs_path() -> PathBuf {
    workspace_path().join("logs")
}

/// Get path to audit log
pub fn audit_log_path() -> PathBuf {
    logs_path().join("audit.jsonl")
}

/// Get path to sessions directory
pub fn sessions_path() -> PathBuf {
    workspace_path().join("sessions")
}

/// Get path to contracts directory
pub fn contracts_path() -> PathBuf {
    workspace_path().join("contracts")
}

/// Get path to a specific session
pub fn session_path(session_id: &str) -> PathBuf {
    sessions_path().join(session_id)
}

/// Get path to evidence for a session
pub fn evidence_path(session_id: &str) -> PathBuf {
    session_path(session_id).join("evidence")
}

/// Load state from state.json
pub async fn load_state() -> Result<StateData> {
    let path = state_path();

    if !path.exists() {
        anyhow::bail!("Workspace not initialized. Run 'vibeanvil init' first.");
    }

    let content = fs::read_to_string(&path)
        .await
        .with_context(|| format!("Failed to read state file: {}", path.display()))?;

    let state: StateData =
        serde_json::from_str(&content).with_context(|| "Failed to parse state.json")?;

    Ok(state)
}

/// Save state to state.json
pub async fn save_state(state: &StateData) -> Result<()> {
    let path = state_path();
    let content = serde_json::to_string_pretty(state)?;

    fs::write(&path, content)
        .await
        .with_context(|| format!("Failed to write state file: {}", path.display()))?;

    Ok(())
}

/// Ensure session directory exists
pub async fn ensure_session(session_id: &str) -> Result<PathBuf> {
    let path = session_path(session_id);
    fs::create_dir_all(&path).await?;
    fs::create_dir_all(evidence_path(session_id)).await?;
    Ok(path)
}

/// Get the cache directory (user-level)
pub fn cache_dir() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("vibeanvil")
}

/// Get the brainpack directory
pub fn brainpack_dir() -> PathBuf {
    cache_dir().join("brainpack")
}

/// Ensure cache directory exists
pub async fn ensure_cache() -> Result<PathBuf> {
    let path = brainpack_dir();
    fs::create_dir_all(&path).await?;
    Ok(path)
}
