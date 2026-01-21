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
//! ## Capabilities
//!
//! - **Tools** - 20 VibeAnvil commands exposed as MCP tools
//! - **Resources** - Project artifacts (contract, plan, state) as readable resources
//! - **Prompts** - Workflow prompt templates (plan, review, implement, etc.)
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
//! ## Resources
//!
//! - `vibeanvil://contract` - Project contract
//! - `vibeanvil://plan` - Implementation plan
//! - `vibeanvil://state` - Workflow state (JSON)
//! - `vibeanvil://constitution` - AI guidelines
//!
//! ## Prompts
//!
//! - `plan` - Generate implementation plan
//! - `review` - Code review
//! - `architect` - Architecture design
//! - `developer` - Implementation
//! - `commit` - Commit message
//!
//! ## Usage
//!
//! Start MCP server: `vibeanvil mcp serve`
//! Test with inspector: `vibeanvil mcp test`

pub mod http_transport;
pub mod prompts;
pub mod protocol;
pub mod resources;
pub mod server;
pub mod tools;
pub mod transport;

pub use server::McpServer;
