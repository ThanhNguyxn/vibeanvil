//! MCP STDIO Transport
//!
//! Handles reading JSON-RPC messages from stdin and writing to stdout.

use std::io::{self, BufRead, Write};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

use tracing::{debug, error, trace};

use super::protocol::{JsonRpcRequest, JsonRpcResponse, JsonRpcNotification};

/// STDIO Transport for MCP
pub struct StdioTransport {
    _marker: (),
}

impl StdioTransport {
    pub fn new() -> Self {
        Self { _marker: () }
    }

    /// Run the transport, processing messages from stdin and sending to stdout
    pub async fn run<F, Fut>(
        &self,
        mut handler: F,
    ) -> anyhow::Result<()>
    where
        F: FnMut(JsonRpcRequest) -> Fut,
        Fut: std::future::Future<Output = Option<JsonRpcResponse>>,
    {
        let stdin = tokio::io::stdin();
        let mut stdout = tokio::io::stdout();
        let reader = BufReader::new(stdin);
        let mut lines = reader.lines();

        debug!("MCP STDIO transport started, waiting for messages...");

        while let Some(line) = lines.next_line().await? {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            trace!("Received: {}", line);

            // Parse the JSON-RPC request
            match serde_json::from_str::<JsonRpcRequest>(line) {
                Ok(request) => {
                    // Handle the request
                    if let Some(response) = handler(request).await {
                        let response_json = serde_json::to_string(&response)?;
                        trace!("Sending: {}", response_json);
                        stdout.write_all(response_json.as_bytes()).await?;
                        stdout.write_all(b"\n").await?;
                        stdout.flush().await?;
                    }
                }
                Err(e) => {
                    error!("Failed to parse JSON-RPC request: {}", e);
                    let error_response = JsonRpcResponse::error(
                        None,
                        super::protocol::JsonRpcError::parse_error(),
                    );
                    let response_json = serde_json::to_string(&error_response)?;
                    stdout.write_all(response_json.as_bytes()).await?;
                    stdout.write_all(b"\n").await?;
                    stdout.flush().await?;
                }
            }
        }

        debug!("MCP STDIO transport shutting down");
        Ok(())
    }

    /// Send a notification to stdout
    pub async fn send_notification(notification: JsonRpcNotification) -> anyhow::Result<()> {
        let mut stdout = tokio::io::stdout();
        let json = serde_json::to_string(&notification)?;
        trace!("Sending notification: {}", json);
        stdout.write_all(json.as_bytes()).await?;
        stdout.write_all(b"\n").await?;
        stdout.flush().await?;
        Ok(())
    }
}

impl Default for StdioTransport {
    fn default() -> Self {
        Self::new()
    }
}

/// Synchronous STDIO Transport for simpler use cases
pub struct SyncStdioTransport {
    _marker: (),
}

impl SyncStdioTransport {
    pub fn new() -> Self {
        Self { _marker: () }
    }

    /// Run the transport synchronously
    pub fn run<F>(&self, mut handler: F) -> anyhow::Result<()>
    where
        F: FnMut(JsonRpcRequest) -> Option<JsonRpcResponse>,
    {
        let stdin = io::stdin();
        let mut stdout = io::stdout();
        let reader = stdin.lock();

        debug!("MCP Sync STDIO transport started");

        for line in reader.lines() {
            let line = line?;
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            trace!("Received: {}", line);

            match serde_json::from_str::<JsonRpcRequest>(line) {
                Ok(request) => {
                    if let Some(response) = handler(request) {
                        let response_json = serde_json::to_string(&response)?;
                        trace!("Sending: {}", response_json);
                        writeln!(stdout, "{}", response_json)?;
                        stdout.flush()?;
                    }
                }
                Err(e) => {
                    error!("Failed to parse JSON-RPC request: {}", e);
                    let error_response = JsonRpcResponse::error(
                        None,
                        super::protocol::JsonRpcError::parse_error(),
                    );
                    let response_json = serde_json::to_string(&error_response)?;
                    writeln!(stdout, "{}", response_json)?;
                    stdout.flush()?;
                }
            }
        }

        Ok(())
    }
}

impl Default for SyncStdioTransport {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stdio_transport_creation() {
        let transport = StdioTransport::new();
        assert!(true); // Just verify it compiles
    }

    #[test]
    fn test_sync_stdio_transport_creation() {
        let transport = SyncStdioTransport::new();
        assert!(true);
    }
}
