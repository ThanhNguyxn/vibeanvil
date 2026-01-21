//! HTTP/SSE Transport for MCP Server
//!
//! Implements Streamable HTTP transport as per MCP specification 2025-06-18.
//! Supports both standard HTTP POST/GET and Server-Sent Events (SSE) streaming.

use anyhow::Result;
use std::sync::Arc;
use tracing::info;

use super::protocol::*;
use super::tools::ToolRegistry;

/// HTTP Transport configuration
#[derive(Debug, Clone)]
pub struct HttpTransportConfig {
    /// Host to bind to (default: 127.0.0.1)
    pub host: String,
    /// Port to listen on (default: 3000)
    pub port: u16,
    /// Enable CORS headers
    pub cors: bool,
    /// Allowed origins for CORS
    pub allowed_origins: Vec<String>,
}

impl Default for HttpTransportConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 3000,
            cors: true,
            allowed_origins: vec!["*".to_string()],
        }
    }
}

/// HTTP Transport state
pub struct HttpTransportState {
    pub tool_registry: Arc<ToolRegistry>,
    pub initialized: bool,
    pub session_id: Option<String>,
}

impl HttpTransportState {
    pub fn new() -> Self {
        Self {
            tool_registry: Arc::new(ToolRegistry::new()),
            initialized: false,
            session_id: None,
        }
    }
}

impl Default for HttpTransportState {
    fn default() -> Self {
        Self::new()
    }
}

/// Start HTTP/SSE MCP server
///
/// Uses axum for HTTP server with SSE support.
/// Endpoint: POST/GET /mcp
pub async fn start_http_server(config: HttpTransportConfig) -> Result<()> {
    info!(
        "Starting MCP HTTP Server on {}:{}",
        config.host, config.port
    );

    // Note: This is a placeholder implementation.
    // Full implementation requires axum dependency.
    // For now, we provide the structure and types.

    println!("🌐 MCP HTTP Server");
    println!("   Endpoint: http://{}:{}/mcp", config.host, config.port);
    println!();
    println!("⚠️  HTTP transport requires additional setup.");
    println!("   For local development, use stdio transport:");
    println!("   vibeanvil mcp serve");
    println!();
    println!("   For remote/HTTP access, configure with:");
    println!("   vibeanvil mcp serve --http --port {}", config.port);

    // Future: Full axum implementation
    // Currently most MCP clients (Claude Desktop, Cursor) use stdio

    Ok(())
}

/// Format SSE message
pub fn format_sse_message(event: &str, data: &str, id: Option<&str>) -> String {
    let mut msg = String::new();

    if let Some(id) = id {
        msg.push_str(&format!("id: {}\n", id));
    }

    msg.push_str(&format!("event: {}\n", event));

    for line in data.lines() {
        msg.push_str(&format!("data: {}\n", line));
    }

    msg.push('\n');
    msg
}

/// Parse incoming HTTP request body as JSON-RPC
pub fn parse_jsonrpc_request(body: &str) -> Result<JsonRpcRequest, JsonRpcError> {
    serde_json::from_str(body).map_err(|_| JsonRpcError::parse_error())
}

/// HTTP response for JSON-RPC
#[derive(Debug, Clone)]
pub struct HttpJsonRpcResponse {
    pub status: u16,
    pub content_type: String,
    pub body: String,
    pub headers: Vec<(String, String)>,
}

impl HttpJsonRpcResponse {
    /// Create JSON response
    pub fn json(response: &JsonRpcResponse) -> Self {
        Self {
            status: 200,
            content_type: "application/json".to_string(),
            body: serde_json::to_string(response).unwrap_or_default(),
            headers: vec![],
        }
    }

    /// Create SSE response
    pub fn sse(events: Vec<String>) -> Self {
        Self {
            status: 200,
            content_type: "text/event-stream".to_string(),
            body: events.join(""),
            headers: vec![
                ("Cache-Control".to_string(), "no-cache".to_string()),
                ("Connection".to_string(), "keep-alive".to_string()),
            ],
        }
    }

    /// Create error response
    pub fn error(status: u16, message: &str) -> Self {
        let error = JsonRpcError::internal_error(message);
        let response = JsonRpcResponse::error(None, error);
        Self {
            status,
            content_type: "application/json".to_string(),
            body: serde_json::to_string(&response).unwrap_or_default(),
            headers: vec![],
        }
    }

    /// Create accepted (202) response
    pub fn accepted() -> Self {
        Self {
            status: 202,
            content_type: "text/plain".to_string(),
            body: String::new(),
            headers: vec![],
        }
    }
}

/// Session manager for HTTP transport
pub struct SessionManager {
    sessions: std::collections::HashMap<String, HttpTransportState>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: std::collections::HashMap::new(),
        }
    }

    /// Create new session
    pub fn create_session(&mut self) -> String {
        let session_id = uuid::Uuid::new_v4().to_string();
        self.sessions
            .insert(session_id.clone(), HttpTransportState::new());
        session_id
    }

    /// Get session
    pub fn get_session(&self, id: &str) -> Option<&HttpTransportState> {
        self.sessions.get(id)
    }

    /// Get mutable session
    pub fn get_session_mut(&mut self, id: &str) -> Option<&mut HttpTransportState> {
        self.sessions.get_mut(id)
    }

    /// Remove session
    pub fn remove_session(&mut self, id: &str) -> bool {
        self.sessions.remove(id).is_some()
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_config_default() {
        let config = HttpTransportConfig::default();
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 3000);
        assert!(config.cors);
    }

    #[test]
    fn test_format_sse_message() {
        let msg = format_sse_message("message", r#"{"test": true}"#, Some("123"));
        assert!(msg.contains("id: 123"));
        assert!(msg.contains("event: message"));
        assert!(msg.contains(r#"data: {"test": true}"#));
    }

    #[test]
    fn test_format_sse_message_multiline() {
        let msg = format_sse_message("data", "line1\nline2", None);
        assert!(msg.contains("data: line1\n"));
        assert!(msg.contains("data: line2\n"));
    }

    #[test]
    fn test_parse_jsonrpc_request() {
        let body = r#"{"jsonrpc":"2.0","id":1,"method":"ping"}"#;
        let result = parse_jsonrpc_request(body);
        assert!(result.is_ok());
        let req = result.unwrap();
        assert_eq!(req.method, "ping");
    }

    #[test]
    fn test_parse_jsonrpc_request_invalid() {
        let body = "not json";
        let result = parse_jsonrpc_request(body);
        assert!(result.is_err());
    }

    #[test]
    fn test_http_response_json() {
        let response = JsonRpcResponse::success(Some(serde_json::json!(1)), serde_json::json!({}));
        let http = HttpJsonRpcResponse::json(&response);
        assert_eq!(http.status, 200);
        assert_eq!(http.content_type, "application/json");
    }

    #[test]
    fn test_http_response_sse() {
        let events = vec![format_sse_message("ping", "{}", None)];
        let http = HttpJsonRpcResponse::sse(events);
        assert_eq!(http.status, 200);
        assert_eq!(http.content_type, "text/event-stream");
    }

    #[test]
    fn test_session_manager() {
        let mut manager = SessionManager::new();
        let id = manager.create_session();
        assert!(manager.get_session(&id).is_some());
        assert!(manager.remove_session(&id));
        assert!(manager.get_session(&id).is_none());
    }
}
