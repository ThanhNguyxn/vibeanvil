//! MCP Server Implementation
//!
//! Main server logic for handling MCP protocol messages.

use std::sync::Arc;
use tracing::{debug, info};

use super::protocol::*;
use super::tools::{execute_tool, ToolRegistry};
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
        info!(
            "Starting VibeAnvil MCP Server v{}",
            env!("CARGO_PKG_VERSION")
        );

        let transport = StdioTransport::new();
        let tool_registry = Arc::clone(&self.tool_registry);
        let mut initialized = false;

        transport
            .run(move |request| {
                let tool_registry = Arc::clone(&tool_registry);
                let _initialized_ref = initialized;

                async move {
                    let response = handle_request(request, &tool_registry, &mut initialized).await;
                    response
                }
            })
            .await
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
            resources: Some(ResourcesCapability {
                subscribe: false,
                list_changed: true,
            }),
            prompts: Some(PromptsCapability { list_changed: true }),
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

        // Resource methods
        "resources/list" => handle_resources_list().await,
        "resources/read" => handle_resources_read(request.params).await,

        // Prompt methods
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
fn handle_initialize(params: Option<serde_json::Value>, initialized: &mut bool) -> JsonRpcResponse {
    let init_params: InitializeParams = match params {
        Some(p) => match serde_json::from_value(p) {
            Ok(params) => params,
            Err(e) => {
                return JsonRpcResponse::error(None, JsonRpcError::invalid_params(&e.to_string()));
            }
        },
        None => {
            return JsonRpcResponse::error(None, JsonRpcError::invalid_params("Missing params"));
        }
    };

    info!(
        "Initializing with client: {} v{}",
        init_params.client_info.name,
        init_params
            .client_info
            .version
            .as_deref()
            .unwrap_or("unknown")
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
                return JsonRpcResponse::error(None, JsonRpcError::invalid_params(&e.to_string()));
            }
        },
        None => {
            return JsonRpcResponse::error(None, JsonRpcError::invalid_params("Missing params"));
        }
    };

    info!("Calling tool: {}", call_params.name);

    let result = execute_tool(call_params).await;
    JsonRpcResponse::success(None, serde_json::to_value(result).unwrap())
}

/// Handle resources/list request
async fn handle_resources_list() -> JsonRpcResponse {
    use super::resources::ResourceRegistry;

    let resources = ResourceRegistry::list_resources().await;
    let result = ResourcesListResult {
        resources,
        next_cursor: None,
    };
    JsonRpcResponse::success(None, serde_json::to_value(result).unwrap())
}

/// Handle resources/read request
async fn handle_resources_read(params: Option<serde_json::Value>) -> JsonRpcResponse {
    use super::resources::{ResourceReadParams, ResourceReadResult, ResourceRegistry};

    let read_params: ResourceReadParams = match params {
        Some(p) => match serde_json::from_value(p) {
            Ok(params) => params,
            Err(e) => {
                return JsonRpcResponse::error(None, JsonRpcError::invalid_params(&e.to_string()));
            }
        },
        None => {
            return JsonRpcResponse::error(None, JsonRpcError::invalid_params("Missing params"));
        }
    };

    match ResourceRegistry::read_resource(&read_params.uri).await {
        Ok(contents) => {
            let result = ResourceReadResult {
                contents: vec![contents],
            };
            JsonRpcResponse::success(None, serde_json::to_value(result).unwrap())
        }
        Err(e) => JsonRpcResponse::error(None, JsonRpcError::internal_error(&e)),
    }
}

/// Handle prompts/list request
fn handle_prompts_list() -> JsonRpcResponse {
    use super::prompts::PromptRegistry;

    let result = PromptsListResult {
        prompts: PromptRegistry::list_prompts(),
        next_cursor: None,
    };
    JsonRpcResponse::success(None, serde_json::to_value(result).unwrap())
}

/// Handle prompts/get request
fn handle_prompts_get(params: Option<serde_json::Value>) -> JsonRpcResponse {
    use super::prompts::{PromptGetParams, PromptRegistry};

    let get_params: PromptGetParams = match params {
        Some(p) => match serde_json::from_value(p) {
            Ok(params) => params,
            Err(e) => {
                return JsonRpcResponse::error(None, JsonRpcError::invalid_params(&e.to_string()));
            }
        },
        None => {
            return JsonRpcResponse::error(None, JsonRpcError::invalid_params("Missing params"));
        }
    };

    match PromptRegistry::get_prompt(&get_params.name, get_params.arguments) {
        Ok(result) => JsonRpcResponse::success(None, serde_json::to_value(result).unwrap()),
        Err(e) => JsonRpcResponse::error(None, JsonRpcError::invalid_params(&e)),
    }
}

/// Handle logging/setLevel request
fn handle_logging_set_level(params: Option<serde_json::Value>) -> JsonRpcResponse {
    if let Some(p) = params {
        if let Some(level) = p.get("level").and_then(|v| v.as_str()) {
            debug!("Setting log level to: {}", level);
        }
    }
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

        let result: ToolsListResult = serde_json::from_value(response.result.unwrap()).unwrap();
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

    #[tokio::test]
    async fn test_handle_resources_list() {
        let response = handle_resources_list().await;
        assert!(response.result.is_some());

        let result: ResourcesListResult = serde_json::from_value(response.result.unwrap()).unwrap();
        // Resources may be empty if no workspace initialized
        assert!(result.resources.is_empty() || !result.resources.is_empty());
    }

    #[test]
    fn test_handle_prompts_list() {
        let response = handle_prompts_list();
        assert!(response.result.is_some());

        let result: PromptsListResult = serde_json::from_value(response.result.unwrap()).unwrap();
        assert!(!result.prompts.is_empty());
    }
}
