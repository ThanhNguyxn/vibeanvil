//! MCP Tools
//!
//! Defines all VibeAnvil tools exposed via MCP.

use serde_json::json;

use super::protocol::{CallToolParams, CallToolResult, ToolAnnotations, ToolDefinition};

/// Registry of all available MCP tools
pub struct ToolRegistry {
    tools: Vec<ToolDefinition>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: Self::build_tools(),
        }
    }

    /// Get all tool definitions
    pub fn list_tools(&self) -> &[ToolDefinition] {
        &self.tools
    }

    /// Build all VibeAnvil tools
    fn build_tools() -> Vec<ToolDefinition> {
        vec![
            // === Workflow Commands ===
            ToolDefinition {
                name: "vibeanvil_init".to_string(),
                title: Some("Initialize Project".to_string()),
                description: Some(
                    "Initialize a new VibeAnvil project in the current directory. \
                    Creates .vibeanvil/ folder with state.json and other workflow files."
                        .to_string(),
                ),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "force": {
                            "type": "boolean",
                            "description": "Force reinitialize even if already initialized",
                            "default": false
                        }
                    },
                    "required": []
                }),
                annotations: Some(ToolAnnotations {
                    title: Some("Initialize VibeAnvil".to_string()),
                    read_only_hint: Some(false),
                    destructive_hint: Some(false),
                    idempotent_hint: Some(true),
                    open_world_hint: Some(false),
                }),
            },
            ToolDefinition {
                name: "vibeanvil_status".to_string(),
                title: Some("Get Status".to_string()),
                description: Some(
                    "Get the current workflow status including state, provider, \
                    recent activity, and next steps."
                        .to_string(),
                ),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "verbose": {
                            "type": "boolean",
                            "description": "Show detailed status information",
                            "default": false
                        },
                        "json": {
                            "type": "boolean",
                            "description": "Output in JSON format",
                            "default": true
                        }
                    },
                    "required": []
                }),
                annotations: Some(ToolAnnotations {
                    title: Some("Check Status".to_string()),
                    read_only_hint: Some(true),
                    destructive_hint: Some(false),
                    idempotent_hint: Some(true),
                    open_world_hint: Some(false),
                }),
            },
            ToolDefinition {
                name: "vibeanvil_intake".to_string(),
                title: Some("Capture Requirements".to_string()),
                description: Some(
                    "Capture user requirements/ideas. This is the first step in the \
                    VibeAnvil workflow. Pass a message describing what you want to build."
                        .to_string(),
                ),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "message": {
                            "type": "string",
                            "description": "The requirement or idea to capture"
                        }
                    },
                    "required": ["message"]
                }),
                annotations: Some(ToolAnnotations {
                    title: Some("Capture Intake".to_string()),
                    read_only_hint: Some(false),
                    destructive_hint: Some(false),
                    idempotent_hint: Some(false),
                    open_world_hint: Some(false),
                }),
            },
            ToolDefinition {
                name: "vibeanvil_plan".to_string(),
                title: Some("Generate Plan".to_string()),
                description: Some(
                    "Generate an AI implementation plan based on the contract. \
                    Uses the configured provider to create a detailed plan."
                        .to_string(),
                ),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "provider": {
                            "type": "string",
                            "description": "Provider to use (e.g., 'claude', 'openai', 'human')",
                            "default": "human"
                        }
                    },
                    "required": []
                }),
                annotations: Some(ToolAnnotations {
                    title: Some("Generate Plan".to_string()),
                    read_only_hint: Some(false),
                    destructive_hint: Some(false),
                    idempotent_hint: Some(false),
                    open_world_hint: Some(true),
                }),
            },
            ToolDefinition {
                name: "vibeanvil_build".to_string(),
                title: Some("Execute Build".to_string()),
                description: Some(
                    "Execute the build process. Can run in manual, auto, or iterate mode. \
                    Manual mode prompts for confirmation, auto mode runs continuously."
                        .to_string(),
                ),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "mode": {
                            "type": "string",
                            "enum": ["manual", "auto", "iterate"],
                            "description": "Build mode: manual (step-by-step), auto (continuous), iterate (auto+watch)",
                            "default": "manual"
                        },
                        "provider": {
                            "type": "string",
                            "description": "Provider to use for build"
                        }
                    },
                    "required": []
                }),
                annotations: Some(ToolAnnotations {
                    title: Some("Build Project".to_string()),
                    read_only_hint: Some(false),
                    destructive_hint: Some(true),
                    idempotent_hint: Some(false),
                    open_world_hint: Some(true),
                }),
            },
            // === Task Management ===
            ToolDefinition {
                name: "vibeanvil_tasks".to_string(),
                title: Some("Manage Tasks".to_string()),
                description: Some(
                    "Break down the plan into actionable tasks. Shows task list with status. \
                    Use regenerate=true to create new tasks from the current plan."
                        .to_string(),
                ),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "regenerate": {
                            "type": "boolean",
                            "description": "Regenerate tasks from the plan",
                            "default": false
                        },
                        "done": {
                            "type": "string",
                            "description": "Mark a task as done by ID"
                        },
                        "provider": {
                            "type": "string",
                            "description": "Provider for task generation"
                        }
                    },
                    "required": []
                }),
                annotations: Some(ToolAnnotations {
                    title: Some("Task Management".to_string()),
                    read_only_hint: Some(false),
                    destructive_hint: Some(false),
                    idempotent_hint: Some(false),
                    open_world_hint: Some(false),
                }),
            },
            ToolDefinition {
                name: "vibeanvil_implement".to_string(),
                title: Some("Auto-Implement Tasks".to_string()),
                description: Some(
                    "Automatically implement tasks using AI. Can implement a specific task \
                    or all pending tasks. Use dry_run=true to preview without changes."
                        .to_string(),
                ),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "task": {
                            "type": "string",
                            "description": "Specific task ID to implement"
                        },
                        "all": {
                            "type": "boolean",
                            "description": "Implement all pending tasks",
                            "default": false
                        },
                        "dry_run": {
                            "type": "boolean",
                            "description": "Preview changes without applying",
                            "default": false
                        },
                        "provider": {
                            "type": "string",
                            "description": "Provider for implementation"
                        }
                    },
                    "required": []
                }),
                annotations: Some(ToolAnnotations {
                    title: Some("Auto Implement".to_string()),
                    read_only_hint: Some(false),
                    destructive_hint: Some(true),
                    idempotent_hint: Some(false),
                    open_world_hint: Some(true),
                }),
            },
            // === Analysis & Review ===
            ToolDefinition {
                name: "vibeanvil_analyze".to_string(),
                title: Some("Analyze Consistency".to_string()),
                description: Some(
                    "Cross-artifact consistency analysis. Checks that contract, plan, \
                    tasks, and code are aligned."
                        .to_string(),
                ),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "provider": {
                            "type": "string",
                            "description": "Provider for analysis"
                        }
                    },
                    "required": []
                }),
                annotations: Some(ToolAnnotations {
                    title: Some("Analyze Project".to_string()),
                    read_only_hint: Some(true),
                    destructive_hint: Some(false),
                    idempotent_hint: Some(true),
                    open_world_hint: Some(false),
                }),
            },
            ToolDefinition {
                name: "vibeanvil_review".to_string(),
                title: Some("Code Review".to_string()),
                description: Some(
                    "Run code review on recent changes. Can show pending review, \
                    approve changes, or request revisions."
                        .to_string(),
                ),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "action": {
                            "type": "string",
                            "enum": ["show", "approve", "reject"],
                            "description": "Review action",
                            "default": "show"
                        }
                    },
                    "required": []
                }),
                annotations: Some(ToolAnnotations {
                    title: Some("Code Review".to_string()),
                    read_only_hint: Some(false),
                    destructive_hint: Some(false),
                    idempotent_hint: Some(false),
                    open_world_hint: Some(false),
                }),
            },
            // === Codebase Understanding ===
            ToolDefinition {
                name: "vibeanvil_map".to_string(),
                title: Some("Codebase Map".to_string()),
                description: Some(
                    "Generate a repository map showing the codebase structure, \
                    important files, symbols, and dependencies."
                        .to_string(),
                ),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "max_tokens": {
                            "type": "integer",
                            "description": "Maximum tokens for the map output",
                            "default": 8000
                        }
                    },
                    "required": []
                }),
                annotations: Some(ToolAnnotations {
                    title: Some("Repository Map".to_string()),
                    read_only_hint: Some(true),
                    destructive_hint: Some(false),
                    idempotent_hint: Some(true),
                    open_world_hint: Some(false),
                }),
            },
            ToolDefinition {
                name: "vibeanvil_brain_query".to_string(),
                title: Some("Query Brain".to_string()),
                description: Some(
                    "Search the project's BrainPack knowledge base. Returns relevant \
                    code snippets, documentation, and patterns from harvested knowledge."
                        .to_string(),
                ),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "Search query"
                        },
                        "limit": {
                            "type": "integer",
                            "description": "Maximum results to return",
                            "default": 10
                        }
                    },
                    "required": ["query"]
                }),
                annotations: Some(ToolAnnotations {
                    title: Some("Search Brain".to_string()),
                    read_only_hint: Some(true),
                    destructive_hint: Some(false),
                    idempotent_hint: Some(true),
                    open_world_hint: Some(false),
                }),
            },
            ToolDefinition {
                name: "vibeanvil_harvest".to_string(),
                title: Some("Harvest Knowledge".to_string()),
                description: Some(
                    "Extract knowledge from the codebase into BrainPack. Analyzes code, \
                    documentation, patterns, and stores in searchable format."
                        .to_string(),
                ),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "preset": {
                            "type": "string",
                            "description": "Harvest preset (e.g., 'default', 'minimal', 'full')"
                        },
                        "path": {
                            "type": "string",
                            "description": "Path to harvest from",
                            "default": "."
                        }
                    },
                    "required": []
                }),
                annotations: Some(ToolAnnotations {
                    title: Some("Harvest Knowledge".to_string()),
                    read_only_hint: Some(false),
                    destructive_hint: Some(false),
                    idempotent_hint: Some(true),
                    open_world_hint: Some(false),
                }),
            },
            // === Code Execution ===
            ToolDefinition {
                name: "vibeanvil_run".to_string(),
                title: Some("Run Command".to_string()),
                description: Some(
                    "Execute a shell command and capture output. Useful for running \
                    tests, builds, or other development commands."
                        .to_string(),
                ),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "command": {
                            "type": "string",
                            "description": "The command to run"
                        },
                        "capture": {
                            "type": "boolean",
                            "description": "Capture output for sharing",
                            "default": true
                        }
                    },
                    "required": ["command"]
                }),
                annotations: Some(ToolAnnotations {
                    title: Some("Run Command".to_string()),
                    read_only_hint: Some(false),
                    destructive_hint: Some(true),
                    idempotent_hint: Some(false),
                    open_world_hint: Some(true),
                }),
            },
            ToolDefinition {
                name: "vibeanvil_test".to_string(),
                title: Some("Run Tests".to_string()),
                description: Some(
                    "Run project tests. Auto-detects test framework. \
                    Use fix=true to attempt auto-fixing failing tests."
                        .to_string(),
                ),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "cmd": {
                            "type": "string",
                            "description": "Custom test command"
                        },
                        "fix": {
                            "type": "boolean",
                            "description": "Attempt to auto-fix failing tests",
                            "default": false
                        }
                    },
                    "required": []
                }),
                annotations: Some(ToolAnnotations {
                    title: Some("Run Tests".to_string()),
                    read_only_hint: Some(false),
                    destructive_hint: Some(false),
                    idempotent_hint: Some(true),
                    open_world_hint: Some(false),
                }),
            },
            ToolDefinition {
                name: "vibeanvil_lint".to_string(),
                title: Some("Run Linter".to_string()),
                description: Some(
                    "Run code linting. Auto-detects linter for the project. \
                    Use fix=true to auto-fix linting issues."
                        .to_string(),
                ),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "cmd": {
                            "type": "string",
                            "description": "Custom lint command"
                        },
                        "fix": {
                            "type": "boolean",
                            "description": "Auto-fix linting issues",
                            "default": false
                        }
                    },
                    "required": []
                }),
                annotations: Some(ToolAnnotations {
                    title: Some("Run Linter".to_string()),
                    read_only_hint: Some(false),
                    destructive_hint: Some(false),
                    idempotent_hint: Some(false),
                    open_world_hint: Some(false),
                }),
            },
            // === Version Control ===
            ToolDefinition {
                name: "vibeanvil_snapshot".to_string(),
                title: Some("Create Snapshot".to_string()),
                description: Some(
                    "Create a snapshot (git commit) of current changes with optional message."
                        .to_string(),
                ),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "message": {
                            "type": "string",
                            "description": "Commit message (auto-generated if not provided)"
                        }
                    },
                    "required": []
                }),
                annotations: Some(ToolAnnotations {
                    title: Some("Create Snapshot".to_string()),
                    read_only_hint: Some(false),
                    destructive_hint: Some(false),
                    idempotent_hint: Some(false),
                    open_world_hint: Some(false),
                }),
            },
            ToolDefinition {
                name: "vibeanvil_undo".to_string(),
                title: Some("Undo Changes".to_string()),
                description: Some(
                    "Undo recent changes. Use dry_run=true to preview what would be undone."
                        .to_string(),
                ),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "dry_run": {
                            "type": "boolean",
                            "description": "Preview changes without undoing",
                            "default": true
                        }
                    },
                    "required": []
                }),
                annotations: Some(ToolAnnotations {
                    title: Some("Undo Changes".to_string()),
                    read_only_hint: Some(false),
                    destructive_hint: Some(true),
                    idempotent_hint: Some(false),
                    open_world_hint: Some(false),
                }),
            },
            // === Chat Modes ===
            ToolDefinition {
                name: "vibeanvil_chat".to_string(),
                title: Some("AI Chat".to_string()),
                description: Some(
                    "Chat with AI in different modes: 'ask' for questions, 'code' for implementation, \
                    'architect' for design, 'help' for VibeAnvil help."
                        .to_string(),
                ),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "mode": {
                            "type": "string",
                            "enum": ["ask", "code", "architect", "help"],
                            "description": "Chat mode",
                            "default": "ask"
                        },
                        "message": {
                            "type": "string",
                            "description": "Your message"
                        },
                        "provider": {
                            "type": "string",
                            "description": "Provider to use"
                        }
                    },
                    "required": ["message"]
                }),
                annotations: Some(ToolAnnotations {
                    title: Some("AI Chat".to_string()),
                    read_only_hint: Some(true),
                    destructive_hint: Some(false),
                    idempotent_hint: Some(true),
                    open_world_hint: Some(true),
                }),
            },
            // === Configuration ===
            ToolDefinition {
                name: "vibeanvil_providers".to_string(),
                title: Some("List Providers".to_string()),
                description: Some(
                    "List all available AI providers with their configuration status."
                        .to_string(),
                ),
                input_schema: json!({
                    "type": "object",
                    "properties": {},
                    "required": []
                }),
                annotations: Some(ToolAnnotations {
                    title: Some("List Providers".to_string()),
                    read_only_hint: Some(true),
                    destructive_hint: Some(false),
                    idempotent_hint: Some(true),
                    open_world_hint: Some(false),
                }),
            },
            ToolDefinition {
                name: "vibeanvil_doctor".to_string(),
                title: Some("Health Check".to_string()),
                description: Some(
                    "Run health check to verify VibeAnvil setup, providers, and dependencies."
                        .to_string(),
                ),
                input_schema: json!({
                    "type": "object",
                    "properties": {},
                    "required": []
                }),
                annotations: Some(ToolAnnotations {
                    title: Some("Doctor Check".to_string()),
                    read_only_hint: Some(true),
                    destructive_hint: Some(false),
                    idempotent_hint: Some(true),
                    open_world_hint: Some(false),
                }),
            },
        ]
    }

    /// Find a tool by name
    pub fn find_tool(&self, name: &str) -> Option<&ToolDefinition> {
        self.tools.iter().find(|t| t.name == name)
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Execute a VibeAnvil tool
pub async fn execute_tool(params: CallToolParams) -> CallToolResult {
    use std::process::Command;

    // Map MCP tool name to vibeanvil CLI command
    let (cmd, args) = match params.name.as_str() {
        "vibeanvil_init" => {
            let mut args = vec!["init".to_string()];
            if params
                .arguments
                .get("force")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
            {
                args.push("--force".to_string());
            }
            ("init", args)
        }
        "vibeanvil_status" => {
            let mut args = vec!["status".to_string()];
            args.push("--json".to_string()); // Always JSON for MCP
            if params
                .arguments
                .get("verbose")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
            {
                args.push("--verbose".to_string());
            }
            ("status", args)
        }
        "vibeanvil_intake" => {
            let message = params
                .arguments
                .get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let args = vec!["intake".to_string(), message.to_string()];
            ("intake", args)
        }
        "vibeanvil_plan" => {
            let mut args = vec!["plan".to_string()];
            if let Some(provider) = params.arguments.get("provider").and_then(|v| v.as_str()) {
                args.push("--provider".to_string());
                args.push(provider.to_string());
            }
            ("plan", args)
        }
        "vibeanvil_build" => {
            let mut args = vec!["build".to_string()];
            if let Some(mode) = params.arguments.get("mode").and_then(|v| v.as_str()) {
                args.push("--mode".to_string());
                args.push(mode.to_string());
            }
            if let Some(provider) = params.arguments.get("provider").and_then(|v| v.as_str()) {
                args.push("--provider".to_string());
                args.push(provider.to_string());
            }
            ("build", args)
        }
        "vibeanvil_tasks" => {
            let mut args = vec!["tasks".to_string()];
            if params
                .arguments
                .get("regenerate")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
            {
                args.push("--regenerate".to_string());
            }
            if let Some(done) = params.arguments.get("done").and_then(|v| v.as_str()) {
                args.push("--done".to_string());
                args.push(done.to_string());
            }
            if let Some(provider) = params.arguments.get("provider").and_then(|v| v.as_str()) {
                args.push("--provider".to_string());
                args.push(provider.to_string());
            }
            ("tasks", args)
        }
        "vibeanvil_implement" => {
            let mut args = vec!["implement".to_string()];
            if let Some(task) = params.arguments.get("task").and_then(|v| v.as_str()) {
                args.push("--task".to_string());
                args.push(task.to_string());
            }
            if params
                .arguments
                .get("all")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
            {
                args.push("--all".to_string());
            }
            if params
                .arguments
                .get("dry_run")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
            {
                args.push("--dry-run".to_string());
            }
            if let Some(provider) = params.arguments.get("provider").and_then(|v| v.as_str()) {
                args.push("--provider".to_string());
                args.push(provider.to_string());
            }
            ("implement", args)
        }
        "vibeanvil_analyze" => {
            let mut args = vec!["analyze".to_string()];
            if let Some(provider) = params.arguments.get("provider").and_then(|v| v.as_str()) {
                args.push("--provider".to_string());
                args.push(provider.to_string());
            }
            ("analyze", args)
        }
        "vibeanvil_review" => {
            let mut args = vec!["review".to_string()];
            if let Some(action) = params.arguments.get("action").and_then(|v| v.as_str()) {
                args.push(action.to_string());
            }
            ("review", args)
        }
        "vibeanvil_map" => {
            let mut args = vec!["map".to_string()];
            if let Some(max_tokens) = params.arguments.get("max_tokens").and_then(|v| v.as_i64()) {
                args.push("--max-tokens".to_string());
                args.push(max_tokens.to_string());
            }
            ("map", args)
        }
        "vibeanvil_brain_query" => {
            let query = params
                .arguments
                .get("query")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let mut args = vec!["brain".to_string(), "query".to_string(), query.to_string()];
            if let Some(limit) = params.arguments.get("limit").and_then(|v| v.as_i64()) {
                args.push("--limit".to_string());
                args.push(limit.to_string());
            }
            ("brain query", args)
        }
        "vibeanvil_harvest" => {
            let mut args = vec!["harvest".to_string()];
            if let Some(preset) = params.arguments.get("preset").and_then(|v| v.as_str()) {
                args.push("--preset".to_string());
                args.push(preset.to_string());
            }
            if let Some(path) = params.arguments.get("path").and_then(|v| v.as_str()) {
                args.push(path.to_string());
            }
            ("harvest", args)
        }
        "vibeanvil_run" => {
            let command = params
                .arguments
                .get("command")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let mut args = vec!["run".to_string(), command.to_string()];
            if params
                .arguments
                .get("capture")
                .and_then(|v| v.as_bool())
                .unwrap_or(true)
            {
                args.push("--capture".to_string());
            }
            ("run", args)
        }
        "vibeanvil_test" => {
            let mut args = vec!["test".to_string()];
            if let Some(cmd) = params.arguments.get("cmd").and_then(|v| v.as_str()) {
                args.push("--cmd".to_string());
                args.push(cmd.to_string());
            }
            if params
                .arguments
                .get("fix")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
            {
                args.push("--fix".to_string());
            }
            ("test", args)
        }
        "vibeanvil_lint" => {
            let mut args = vec!["lint".to_string()];
            if let Some(cmd) = params.arguments.get("cmd").and_then(|v| v.as_str()) {
                args.push("--cmd".to_string());
                args.push(cmd.to_string());
            }
            if params
                .arguments
                .get("fix")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
            {
                args.push("--fix".to_string());
            }
            ("lint", args)
        }
        "vibeanvil_snapshot" => {
            let mut args = vec!["snapshot".to_string()];
            if let Some(message) = params.arguments.get("message").and_then(|v| v.as_str()) {
                args.push("--message".to_string());
                args.push(message.to_string());
            }
            ("snapshot", args)
        }
        "vibeanvil_undo" => {
            let mut args = vec!["undo".to_string()];
            if params
                .arguments
                .get("dry_run")
                .and_then(|v| v.as_bool())
                .unwrap_or(true)
            {
                args.push("--dry-run".to_string());
            }
            ("undo", args)
        }
        "vibeanvil_chat" => {
            let mode = params
                .arguments
                .get("mode")
                .and_then(|v| v.as_str())
                .unwrap_or("ask");
            let message = params
                .arguments
                .get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let mut args = vec![
                "chat".to_string(),
                "--mode".to_string(),
                mode.to_string(),
                message.to_string(),
            ];
            if let Some(provider) = params.arguments.get("provider").and_then(|v| v.as_str()) {
                args.push("--provider".to_string());
                args.push(provider.to_string());
            }
            ("chat", args)
        }
        "vibeanvil_providers" => {
            let args = vec!["providers".to_string()];
            ("providers", args)
        }
        "vibeanvil_doctor" => {
            let args = vec!["doctor".to_string()];
            ("doctor", args)
        }
        _ => {
            return CallToolResult::error(&format!("Unknown tool: {}", params.name));
        }
    };

    // Execute vibeanvil command
    let output = match Command::new("vibeanvil")
        .args(&args[1..]) // Skip first arg which is just the command name for logging
        .arg(&args[0])
        .output()
    {
        Ok(output) => output,
        Err(e) => {
            // Try with full path or current directory
            match std::env::current_exe()
                .ok()
                .and_then(|exe| Command::new(exe).args(&args).output().ok())
            {
                Some(output) => output,
                None => {
                    return CallToolResult::error(&format!(
                        "Failed to execute vibeanvil {}: {}",
                        cmd, e
                    ));
                }
            }
        }
    };

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        CallToolResult::success(&stdout)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let error_msg = if stderr.is_empty() {
            stdout.to_string()
        } else {
            format!("{}\n{}", stderr, stdout)
        };
        CallToolResult::error(&error_msg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_registry_creation() {
        let registry = ToolRegistry::new();
        assert!(!registry.list_tools().is_empty());
    }

    #[test]
    fn test_find_tool() {
        let registry = ToolRegistry::new();
        let tool = registry.find_tool("vibeanvil_status");
        assert!(tool.is_some());
        assert_eq!(tool.unwrap().name, "vibeanvil_status");
    }

    #[test]
    fn test_all_tools_have_descriptions() {
        let registry = ToolRegistry::new();
        for tool in registry.list_tools() {
            assert!(
                tool.description.is_some(),
                "Tool {} missing description",
                tool.name
            );
        }
    }

    #[test]
    fn test_all_tools_have_valid_schema() {
        let registry = ToolRegistry::new();
        for tool in registry.list_tools() {
            assert!(
                tool.input_schema.is_object(),
                "Tool {} has invalid schema",
                tool.name
            );
        }
    }

    #[test]
    fn test_tool_count() {
        let registry = ToolRegistry::new();
        // Should have at least 15 tools
        assert!(registry.list_tools().len() >= 15);
    }
}
