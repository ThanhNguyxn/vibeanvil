//! MCP Server Implementation
//!
//! Main server logic for handling MCP protocol messages.

use std::sync::Arc;
use tracing::{debug, info};

use super::protocol::*;
use super::tools::{ToolRegistry, execute_tool};
use super::transport::StdioTransport;

/// VibeAnvil MCP Server
pub struct McpServer {
    tool_registry: Arc<ToolRegistry>,
    initialized: bool,
    client_info: Option<ClientInfo>,
}

impl McpServer {
    pub fn new() -> Self {
        Self {
            tool_registry: Arc::new(ToolRegistry::new()),
            initialized: false,
            client_info: None,
        }
    }

    /// Run the MCP server with STDIO transport
    pub async fn run(&mut self) -> anyhow::Result<()> {
        info!("Starting VibeAnvil MCP Server v{}", env!("CARGO_PKG_VERSION"));
        
        let transport = StdioTransport::new();
        let tool_registry = Arc::clone(&self.tool_registry);
        let mut initialized = false;

        transport.run(move |request| {
            let tool_registry = Arc::clone(&tool_registry);
            let _initialized_ref = initialized;
            
            async move {
                let response = handle_request(request, &tool_registry, &mut initialized).await;
                response
            }
        }).await
    }

    /// Get server info
    pub fn server_info() -> ServerInfo {
        ServerInfo {
            name: "vibeanvil".to_string(),
            version: Some(env!("CARGO_PKG_VERSION").to_string()),
        }
    }

    /// Get server capabilities
    pub fn capabilities() -> ServerCapabilities {
        ServerCapabilities {
            tools: Some(ToolsCapability { list_changed: true }),
            resources: None, // Can add resource support later
            prompts: None,   // Can add prompt templates later
            logging: Some(LoggingCapability {}),
        }
    }

    /// Get server instructions
    pub fn instructions() -> String {
        r#"VibeAnvil is a contract-first vibe coding CLI with evidence, audit, and repo-brain harvesting.

## Workflow
1. `vibeanvil_init` - Initialize a new project
2. `vibeanvil_intake` - Capture user requirements
3. `vibeanvil_plan` - Generate implementation plan
4. `vibeanvil_tasks` - Break down into actionable tasks
5. `vibeanvil_implement` - Auto-implement tasks with AI
6. `vibeanvil_review` - Review and approve changes
7. `vibeanvil_snapshot` - Commit changes

## Key Features
- Contract-first development with evidence tracking
- Multi-provider support (Claude, OpenAI, Ollama, etc.)
- BrainPack knowledge base for project context
- Repository mapping for codebase understanding

Use `vibeanvil_status` to see current workflow state.
Use `vibeanvil_brain_query` to search project knowledge.
Use `vibeanvil_map` to understand codebase structure."#.to_string()
    }
}

impl Default for McpServer {
    fn default() -> Self {
        Self::new()
    }
}

/// Handle a JSON-RPC request
async fn handle_request(
    request: JsonRpcRequest,
    tool_registry: &ToolRegistry,
    initialized: &mut bool,
) -> Option<JsonRpcResponse> {
    debug!("Handling method: {}", request.method);

    // Notifications don't have IDs and don't return responses
    if request.id.is_none() {
        handle_notification(&request.method, request.params.as_ref());
        return None;
    }

    let response = match request.method.as_str() {
        // Lifecycle methods
        "initialize" => handle_initialize(request.params, initialized),
        "initialized" => {
            // This is a notification, should not have an id
            *initialized = true;
            return None;
        }
        "ping" => handle_ping(),

        // Tool methods
        "tools/list" => {
            if !*initialized {
                JsonRpcResponse::error(
                    request.id.clone(),
                    JsonRpcError::internal_error("Server not initialized"),
                )
            } else {
                handle_tools_list(tool_registry)
            }
        }
        "tools/call" => {
            if !*initialized {
                JsonRpcResponse::error(
                    request.id.clone(),
                    JsonRpcError::internal_error("Server not initialized"),
                )
            } else {
                handle_tools_call(request.params).await
            }
        }

        // Resource methods (placeholder for future)
        "resources/list" => handle_resources_list(),
        "resources/read" => handle_resources_read(request.params),

        // Prompt methods (placeholder for future)
        "prompts/list" => handle_prompts_list(),
        "prompts/get" => handle_prompts_get(request.params),

        // Logging methods
        "logging/setLevel" => handle_logging_set_level(request.params),

        // Unknown method
        _ => JsonRpcResponse::error(
            request.id.clone(),
            JsonRpcError::method_not_found(&request.method),
        ),
    };

    Some(JsonRpcResponse {
        id: request.id,
        ..response
    })
}

/// Handle notification (no response)
fn handle_notification(method: &str, _params: Option<&serde_json::Value>) {
    match method {
        "notifications/initialized" => {
            info!("Client initialized");
        }
        "notifications/cancelled" => {
            debug!("Request cancelled");
        }
        _ => {
            debug!("Unknown notification: {}", method);
        }
    }
}

/// Handle initialize request
fn handle_initialize(
    params: Option<serde_json::Value>,
    initialized: &mut bool,
) -> JsonRpcResponse {
    let init_params: InitializeParams = match params {
        Some(p) => match serde_json::from_value(p) {
            Ok(params) => params,
            Err(e) => {
                return JsonRpcResponse::error(
                    None,
                    JsonRpcError::invalid_params(&e.to_string()),
                );
            }
        },
        None => {
            return JsonRpcResponse::error(
                None,
                JsonRpcError::invalid_params("Missing params"),
            );
        }
    };

    info!(
        "Initializing with client: {} v{}",
        init_params.client_info.name,
        init_params.client_info.version.as_deref().unwrap_or("unknown")
    );

    *initialized = true;

    let result = InitializeResult {
        protocol_version: MCP_PROTOCOL_VERSION.to_string(),
        capabilities: McpServer::capabilities(),
        server_info: McpServer::server_info(),
        instructions: Some(McpServer::instructions()),
    };

    JsonRpcResponse::success(None, serde_json::to_value(result).unwrap())
}

/// Handle ping request
fn handle_ping() -> JsonRpcResponse {
    JsonRpcResponse::success(None, serde_json::json!({}))
}

/// Handle tools/list request
fn handle_tools_list(tool_registry: &ToolRegistry) -> JsonRpcResponse {
    let result = ToolsListResult {
        tools: tool_registry.list_tools().to_vec(),
        next_cursor: None,
    };
    JsonRpcResponse::success(None, serde_json::to_value(result).unwrap())
}

/// Handle tools/call request
async fn handle_tools_call(params: Option<serde_json::Value>) -> JsonRpcResponse {
    let call_params: CallToolParams = match params {
        Some(p) => match serde_json::from_value(p) {
            Ok(params) => params,
            Err(e) => {
                return JsonRpcResponse::error(
                    None,
                    JsonRpcError::invalid_params(&e.to_string()),
                );
            }
        },
        None => {
            return JsonRpcResponse::error(
                None,
                JsonRpcError::invalid_params("Missing params"),
            );
        }
    };

    info!("Calling tool: {}", call_params.name);
    
    let result = execute_tool(call_params).await;
    JsonRpcResponse::success(None, serde_json::to_value(result).unwrap())
}

/// Handle resources/list request (placeholder)
fn handle_resources_list() -> JsonRpcResponse {
    let result = ResourcesListResult {
        resources: vec![
            ResourceDefinition {
                uri: "vibeanvil://contract".to_string(),
                name: "Contract".to_string(),
                description: Some("Current project contract".to_string()),
                mime_type: Some("text/markdown".to_string()),
            },
            ResourceDefinition {
                uri: "vibeanvil://plan".to_string(),
                name: "Plan".to_string(),
                description: Some("Current implementation plan".to_string()),
                mime_type: Some("text/markdown".to_string()),
            },
            ResourceDefinition {
                uri: "vibeanvil://state".to_string(),
                name: "State".to_string(),
                description: Some("Current workflow state".to_string()),
                mime_type: Some("application/json".to_string()),
            },
        ],
        next_cursor: None,
    };
    JsonRpcResponse::success(None, serde_json::to_value(result).unwrap())
}

/// Handle resources/read request (placeholder)
fn handle_resources_read(_params: Option<serde_json::Value>) -> JsonRpcResponse {
    // TODO: Implement actual resource reading
    JsonRpcResponse::error(
        None,
        JsonRpcError::internal_error("Resource reading not yet implemented"),
    )
}

/// Handle prompts/list request (placeholder)
fn handle_prompts_list() -> JsonRpcResponse {
    let result = PromptsListResult {
        prompts: vec![
            PromptDefinition {
                name: "plan".to_string(),
                description: Some("Generate an implementation plan for a feature".to_string()),
                arguments: vec![
                    PromptArgument {
                        name: "feature".to_string(),
                        description: Some("The feature to plan".to_string()),
                        required: true,
                    },
                ],
            },
            PromptDefinition {
                name: "review".to_string(),
                description: Some("Review code changes".to_string()),
                arguments: vec![
                    PromptArgument {
                        name: "files".to_string(),
                        description: Some("Files to review".to_string()),
                        required: false,
                    },
                ],
            },
        ],
        next_cursor: None,
    };
    JsonRpcResponse::success(None, serde_json::to_value(result).unwrap())
}

/// Handle prompts/get request (placeholder)
fn handle_prompts_get(_params: Option<serde_json::Value>) -> JsonRpcResponse {
    // TODO: Implement actual prompt retrieval
    JsonRpcResponse::error(
        None,
        JsonRpcError::internal_error("Prompt retrieval not yet implemented"),
    )
}

/// Handle logging/setLevel request
fn handle_logging_set_level(_params: Option<serde_json::Value>) -> JsonRpcResponse {
    // TODO: Implement logging level changes
    JsonRpcResponse::success(None, serde_json::json!({}))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_creation() {
        let server = McpServer::new();
        assert!(!server.initialized);
    }

    #[test]
    fn test_server_info() {
        let info = McpServer::server_info();
        assert_eq!(info.name, "vibeanvil");
        assert!(info.version.is_some());
    }

    #[test]
    fn test_server_capabilities() {
        let caps = McpServer::capabilities();
        assert!(caps.tools.is_some());
        assert!(caps.logging.is_some());
    }

    #[test]
    fn test_server_instructions() {
        let instructions = McpServer::instructions();
        assert!(instructions.contains("VibeAnvil"));
        assert!(instructions.contains("Workflow"));
    }

    #[tokio::test]
    async fn test_handle_ping() {
        let response = handle_ping();
        assert!(response.result.is_some());
        assert!(response.error.is_none());
    }

    #[test]
    fn test_handle_tools_list() {
        let registry = ToolRegistry::new();
        let response = handle_tools_list(&registry);
        assert!(response.result.is_some());
        
        let result: ToolsListResult = 
            serde_json::from_value(response.result.unwrap()).unwrap();
        assert!(!result.tools.is_empty());
    }

    #[test]
    fn test_handle_initialize() {
        let params = serde_json::json!({
            "protocolVersion": "2025-06-18",
            "capabilities": {},
            "clientInfo": {
                "name": "test-client",
                "version": "1.0.0"
            }
        });
        
        let mut initialized = false;
        let response = handle_initialize(Some(params), &mut initialized);
        
        assert!(response.result.is_some());
        assert!(initialized);
    }

    #[test]
    fn test_handle_resources_list() {
        let response = handle_resources_list();
        assert!(response.result.is_some());
        
        let result: ResourcesListResult = 
            serde_json::from_value(response.result.unwrap()).unwrap();
        assert!(!result.resources.is_empty());
    }

    #[test]
    fn test_handle_prompts_list() {
        let response = handle_prompts_list();
        assert!(response.result.is_some());
        
        let result: PromptsListResult = 
            serde_json::from_value(response.result.unwrap()).unwrap();
        assert!(!result.prompts.is_empty());
    }
}
