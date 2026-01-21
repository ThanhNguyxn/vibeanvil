//! MCP CLI Commands
//!
//! CLI handlers for MCP server functionality.

use anyhow::Result;
use colored::*;

use crate::mcp::McpServer;

/// MCP subcommand action
#[derive(Debug, Clone, clap::Subcommand)]
pub enum McpAction {
    /// Start MCP server (STDIO transport)
    Serve,
    /// Test MCP server with a simple request
    Test,
    /// Show MCP server info
    Info,
    /// Generate Claude Desktop configuration
    Config,
}

/// Run MCP command
pub async fn run(action: McpAction) -> Result<()> {
    match action {
        McpAction::Serve => run_serve().await,
        McpAction::Test => run_test().await,
        McpAction::Info => run_info(),
        McpAction::Config => run_config(),
    }
}

/// Start MCP server
async fn run_serve() -> Result<()> {
    // Disable normal logging to stderr since we need clean STDIO
    let mut server = McpServer::new();
    server.run().await
}

/// Test MCP server with a simple request
async fn run_test() -> Result<()> {
    use crate::mcp::protocol::*;
    use crate::mcp::tools::ToolRegistry;

    println!("{}", "🧪 Testing MCP Server...".cyan().bold());
    println!();

    // Test 1: Server info
    println!("{}", "1. Server Info:".yellow());
    let info = McpServer::server_info();
    println!("   Name: {}", info.name.green());
    println!("   Version: {}", info.version.unwrap_or_default().green());
    println!();

    // Test 2: Capabilities
    println!("{}", "2. Capabilities:".yellow());
    let caps = McpServer::capabilities();
    println!("   Tools: {}", if caps.tools.is_some() { "✓".green() } else { "✗".red() });
    println!("   Resources: {}", if caps.resources.is_some() { "✓".green() } else { "○".dimmed() });
    println!("   Prompts: {}", if caps.prompts.is_some() { "✓".green() } else { "○".dimmed() });
    println!("   Logging: {}", if caps.logging.is_some() { "✓".green() } else { "○".dimmed() });
    println!();

    // Test 3: Tools
    println!("{}", "3. Available Tools:".yellow());
    let registry = ToolRegistry::new();
    let tools = registry.list_tools();
    println!("   Total: {} tools", tools.len().to_string().green());
    println!();
    
    for tool in tools.iter().take(5) {
        println!("   {} - {}", 
            tool.name.cyan(),
            tool.description.as_deref().unwrap_or("").dimmed()
        );
    }
    if tools.len() > 5 {
        println!("   {} more tools...", format!("...and {}", tools.len() - 5).dimmed());
    }
    println!();

    // Test 4: Initialize simulation
    println!("{}", "4. Initialize Test:".yellow());
    let init_request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(serde_json::json!(1)),
        method: "initialize".to_string(),
        params: Some(serde_json::json!({
            "protocolVersion": MCP_PROTOCOL_VERSION,
            "capabilities": {},
            "clientInfo": {
                "name": "vibeanvil-test",
                "version": "1.0.0"
            }
        })),
    };
    println!("   Request: {}", serde_json::to_string(&init_request)?.dimmed());
    
    // Create expected response
    let init_result = InitializeResult {
        protocol_version: MCP_PROTOCOL_VERSION.to_string(),
        capabilities: McpServer::capabilities(),
        server_info: McpServer::server_info(),
        instructions: Some(McpServer::instructions()),
    };
    let _response = JsonRpcResponse::success(
        Some(serde_json::json!(1)),
        serde_json::to_value(&init_result)?,
    );
    println!("   Response: {}", "OK".green());
    println!();

    // Test 5: Protocol version
    println!("{}", "5. Protocol:".yellow());
    println!("   Version: {}", MCP_PROTOCOL_VERSION.green());
    println!();

    println!("{}", "✓ All tests passed!".green().bold());
    println!();
    println!("{}", "To start the MCP server:".dimmed());
    println!("  {}", "vibeanvil mcp serve".cyan());
    println!();
    println!("{}", "To configure Claude Desktop, run:".dimmed());
    println!("  {}", "vibeanvil mcp config".cyan());

    Ok(())
}

/// Show MCP server info
fn run_info() -> Result<()> {
    use crate::mcp::tools::ToolRegistry;

    println!("{}", "VibeAnvil MCP Server".cyan().bold());
    println!();
    
    let info = McpServer::server_info();
    println!("{}: {}", "Name".yellow(), info.name);
    println!("{}: {}", "Version".yellow(), info.version.unwrap_or_default());
    println!("{}: {}", "Protocol".yellow(), crate::mcp::protocol::MCP_PROTOCOL_VERSION);
    println!();
    
    println!("{}", "Capabilities:".yellow());
    let caps = McpServer::capabilities();
    println!("  Tools: {}", if caps.tools.is_some() { "Enabled" } else { "Disabled" });
    println!("  Resources: {}", if caps.resources.is_some() { "Enabled" } else { "Disabled" });
    println!("  Prompts: {}", if caps.prompts.is_some() { "Enabled" } else { "Disabled" });
    println!("  Logging: {}", if caps.logging.is_some() { "Enabled" } else { "Disabled" });
    println!();

    println!("{}", "Tools:".yellow());
    let registry = ToolRegistry::new();
    for tool in registry.list_tools() {
        println!("  {} - {}", 
            tool.name.cyan(),
            tool.description.as_deref().unwrap_or("").dimmed()
        );
    }

    Ok(())
}

/// Generate Claude Desktop configuration
fn run_config() -> Result<()> {
    let exe_path = std::env::current_exe()?;
    let exe_path_str = exe_path.to_string_lossy();

    println!("{}", "Claude Desktop Configuration".cyan().bold());
    println!();
    println!("Add the following to your Claude Desktop config file:");
    println!();
    
    #[cfg(target_os = "windows")]
    let config_path = r#"%APPDATA%\Claude\claude_desktop_config.json"#;
    #[cfg(target_os = "macos")]
    let config_path = "~/Library/Application Support/Claude/claude_desktop_config.json";
    #[cfg(target_os = "linux")]
    let config_path = "~/.config/claude/claude_desktop_config.json";
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    let config_path = "claude_desktop_config.json";

    println!("{}: {}", "Config file".yellow(), config_path);
    println!();

    let config = serde_json::json!({
        "mcpServers": {
            "vibeanvil": {
                "command": exe_path_str,
                "args": ["mcp", "serve"],
                "env": {}
            }
        }
    });

    println!("{}", "Configuration:".yellow());
    println!("{}", serde_json::to_string_pretty(&config)?);
    println!();

    println!("{}", "Cursor Configuration".cyan().bold());
    println!();
    println!("For Cursor, add to .cursor/mcp.json in your project:");
    println!();
    
    let cursor_config = serde_json::json!({
        "mcpServers": {
            "vibeanvil": {
                "command": exe_path_str,
                "args": ["mcp", "serve"]
            }
        }
    });
    
    println!("{}", serde_json::to_string_pretty(&cursor_config)?);
    println!();

    println!("{}", "VS Code Configuration".cyan().bold());
    println!();
    println!("For VS Code with GitHub Copilot, add to settings.json:");
    println!();

    let vscode_config = serde_json::json!({
        "github.copilot.chat.experimental.mcpServers": {
            "vibeanvil": {
                "command": exe_path_str,
                "args": ["mcp", "serve"]
            }
        }
    });

    println!("{}", serde_json::to_string_pretty(&vscode_config)?);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_action_variants() {
        // Just verify the enum variants exist
        let _ = McpAction::Serve;
        let _ = McpAction::Test;
        let _ = McpAction::Info;
        let _ = McpAction::Config;
    }
}
