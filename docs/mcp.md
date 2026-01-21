# MCP Server Integration

VibeAnvil includes a built-in MCP (Model Context Protocol) server that allows AI tools like Claude Desktop, Cursor, VS Code + GitHub Copilot, and others to interact directly with your VibeAnvil workflow.

## What is MCP?

Model Context Protocol (MCP) is an open standard that enables AI assistants to securely connect to external tools and data sources. It uses JSON-RPC 2.0 over STDIO for communication.

## Quick Start

### 1. Test the MCP Server

```bash
vibeanvil mcp test
```

This verifies the server is working and shows available tools.

### 2. Get Configuration

```bash
vibeanvil mcp config
```

This generates configuration snippets for all supported AI tools.

### 3. Start the Server

```bash
vibeanvil mcp serve
```

This starts the MCP server (usually called by the AI tool automatically).

## Tool Integration

### Claude Desktop

Add to `%APPDATA%\Claude\claude_desktop_config.json` (Windows) or `~/Library/Application Support/Claude/claude_desktop_config.json` (macOS):

```json
{
  "mcpServers": {
    "vibeanvil": {
      "command": "vibeanvil",
      "args": ["mcp", "serve"],
      "env": {}
    }
  }
}
```

### Cursor

Add to `.cursor/mcp.json` in your project:

```json
{
  "mcpServers": {
    "vibeanvil": {
      "command": "vibeanvil",
      "args": ["mcp", "serve"]
    }
  }
}
```

### VS Code + GitHub Copilot

Add to VS Code `settings.json`:

```json
{
  "github.copilot.chat.experimental.mcpServers": {
    "vibeanvil": {
      "command": "vibeanvil",
      "args": ["mcp", "serve"]
    }
  }
}
```

## Available Tools

The MCP server exposes 20 tools covering the full VibeAnvil workflow:

### Workflow Management

| Tool | Description |
|------|-------------|
| `vibeanvil_init` | Initialize a new project |
| `vibeanvil_status` | Get current workflow status |
| `vibeanvil_intake` | Capture requirements |
| `vibeanvil_plan` | Generate implementation plan |
| `vibeanvil_build` | Execute build process |

### Task Management

| Tool | Description |
|------|-------------|
| `vibeanvil_tasks` | Manage actionable tasks |
| `vibeanvil_implement` | Auto-implement tasks with AI |
| `vibeanvil_analyze` | Check cross-artifact consistency |
| `vibeanvil_review` | Review code changes |

### Codebase Understanding

| Tool | Description |
|------|-------------|
| `vibeanvil_map` | Generate repository structure map |
| `vibeanvil_brain_query` | Search project knowledge base |
| `vibeanvil_harvest` | Extract knowledge to BrainPack |

### Code Execution

| Tool | Description |
|------|-------------|
| `vibeanvil_run` | Execute shell commands |
| `vibeanvil_test` | Run tests with auto-fix option |
| `vibeanvil_lint` | Run linter with auto-fix |

### Version Control

| Tool | Description |
|------|-------------|
| `vibeanvil_snapshot` | Create git commit |
| `vibeanvil_undo` | Undo recent changes |

### Chat & Configuration

| Tool | Description |
|------|-------------|
| `vibeanvil_chat` | Chat in different modes (ask/code/architect/help) |
| `vibeanvil_providers` | List available AI providers |
| `vibeanvil_doctor` | Run health check |

## Example Usage

Once configured, you can ask your AI assistant to:

- "Initialize a new VibeAnvil project" → uses `vibeanvil_init`
- "What's the current workflow status?" → uses `vibeanvil_status`
- "Capture this requirement: Build a REST API" → uses `vibeanvil_intake`
- "Generate tasks from the plan" → uses `vibeanvil_tasks`
- "Search the codebase for authentication patterns" → uses `vibeanvil_brain_query`
- "Create a repository map" → uses `vibeanvil_map`
- "Run the tests" → uses `vibeanvil_test`

## Protocol Details

- **Protocol Version**: 2025-06-18
- **Transport**: STDIO (stdin/stdout)
- **Message Format**: JSON-RPC 2.0
- **Capabilities**: Tools, Resources, Prompts, Logging

## MCP Resources

The MCP server exposes project artifacts as readable resources:

| Resource URI | Description |
|--------------|-------------|
| `vibeanvil://contract` | Project contract defining requirements |
| `vibeanvil://plan` | Implementation plan with tasks |
| `vibeanvil://state` | Current workflow state (JSON) |
| `vibeanvil://constitution` | AI guidelines and rules |
| `vibeanvil://blueprint` | Technical architecture |
| `vibeanvil://intake` | Captured user requirements |
| `vibeanvil://repomap` | Repository structure map |
| `vibeanvil://tasks` | Task breakdown and progress |

AI assistants can read these resources to get context about your project.

## MCP Prompts

The MCP server provides workflow prompt templates:

| Prompt | Arguments | Description |
|--------|-----------|-------------|
| `plan` | `feature`, `context` | Generate implementation plan |
| `review` | `files`, `diff` | Code review |
| `architect` | `requirements`, `constraints` | Architecture design |
| `developer` | `task`, `files` | Implementation |
| `qa` | `feature`, `type` | Test planning |
| `commit` | `changes`, `type` | Git commit message |
| `intake` | `request` | Capture requirements |
| `clarify` | `requirements`, `questions` | Ask clarifying questions |
| `implement` | `task`, `plan` | AI implementation |
| `debug` | `issue`, `error` | Debug and fix issues |

Example: AI can use the `plan` prompt with your feature description to generate a structured implementation plan.

## Troubleshooting

### Server Not Starting

1. Ensure `vibeanvil` is in your PATH
2. Run `vibeanvil mcp test` to verify functionality
3. Check that the configuration paths are correct

### Tools Not Appearing

1. Restart your AI tool after adding configuration
2. Verify the config JSON syntax is valid
3. Check the AI tool's MCP logs for errors

### Commands Failing

1. Ensure you're in a VibeAnvil-initialized directory
2. Run `vibeanvil doctor` to check dependencies
3. Check that required providers are configured

## Security

The MCP server:
- Only executes VibeAnvil commands
- Respects project-level guardrails
- Does not expose filesystem directly
- Logs all operations to audit trail

## Development

To run the MCP server in development:

```bash
cargo run -- mcp serve
```

To test with the MCP Inspector:

```bash
# Install MCP Inspector
npm install -g @modelcontextprotocol/inspector

# Run with inspector
mcp-inspector vibeanvil mcp serve
```
