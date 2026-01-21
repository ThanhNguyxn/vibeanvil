//! # MCP Server Module
//!
//! Model Context Protocol (MCP) server implementation for VibeAnvil.
//! Enables integration with Claude Desktop, Cursor, VS Code, and other MCP-compatible AI tools.
//!
//! ## Architecture
//!
//! MCP uses JSON-RPC 2.0 over STDIO transport to communicate between AI tools and VibeAnvil.
//! The server exposes VibeAnvil commands as MCP tools that can be called by AI assistants.
//!
//! ## Supported Tools
//!
//! - `intake` - Capture user requirements
//! - `plan` - Generate AI implementation plan
//! - `build` - Execute build process
//! - `status` - Get current workflow status
//! - `tasks` - Manage task breakdown
//! - `harvest` - Extract repo knowledge to BrainPack
//! - `brain_query` - Search project knowledge base
//! - `repomap` - Get codebase structure
//! - `analyze` - Cross-artifact consistency check
//!
//! ## Usage
//!
//! Start MCP server: `vibeanvil mcp serve`
//! Test with inspector: `vibeanvil mcp test`

pub mod protocol;
pub mod server;
pub mod tools;
pub mod transport;

pub use server::McpServer;
