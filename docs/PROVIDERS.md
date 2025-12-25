# AI Providers

VibeAnvil is **AI-agnostic** — you can use it with any AI coding assistant. This document describes each available provider and when to use it.

## Quick Reference

| Provider | Best For | Requirements |
|----------|----------|--------------|
| `human` | GitHub Copilot, Cursor, VS Code Chat | None (always available) |
| `claude-code` | Claude Code CLI automation | `@anthropic-ai/claude-code` installed |
| `command` | Any CLI agent (Aider, etc.) | Command available + env vars set |
| `patch` | Diff-based workflows, API LLMs | `git` installed |
| `mock` | Testing | None |

## Which Provider Should I Use?

| If you use... | Choose | Why |
|---------------|--------|-----|
| **GitHub Copilot** | `human` | Copilot works in your IDE; copy the generated prompt, apply suggestions manually |
| **Cursor / Windsurf** | `human` | AI-assisted editing in IDE; paste prompt, let AI suggest, you approve |
| **Claude CLI** | `claude-code` | Direct automation; Claude executes plan and reports back |
| **Aider / Shell Agent** | `command` | Runs any CLI command; agent handles the implementation |
| **API + Custom Script** | `patch` | Generate diffs via API, validate and apply with git |
| **Testing VibeAnvil** | `mock` | No external dependencies; useful for CI/CD testing |

> **Tip:** Not sure? Start with `human`. It works everywhere and teaches you the workflow.

## List Available Providers

```bash
vibeanvil providers
```

---

## Human Provider

**Best for:** GitHub Copilot, Cursor, VS Code Chat, or any IDE-based AI assistant.

### How It Works

1. Generates a prompt file in your session directory
2. You copy the prompt into your IDE assistant
3. Apply the AI's suggestions manually
4. Run evidence capture to complete the build

### Usage

```bash
# Start an iterate build with human provider
vibeanvil build iterate --provider human

# The command will print:
# 1. Path to the generated prompt file
# 2. Instructions for using your IDE assistant
# 3. Commands to run after applying changes
```

### Workflow

```bash
# 1. Lock your contract
vibeanvil contract lock

# 2. Create a plan
vibeanvil plan

# 3. Start iterate build with human provider
vibeanvil build iterate --provider human

# 4. Open the generated prompt file (shown in output)
# 5. Paste content into Copilot/Cursor/VS Code Chat
# 6. Apply the AI's suggestions in your IDE

# 7. Capture evidence and complete
vibeanvil build manual evidence
vibeanvil build manual complete
```

---

## Claude Code Provider

**Best for:** Fully automated AI coding using Claude Code CLI.

### Requirements

```bash
npm install -g @anthropic-ai/claude-code
```

### Usage

```bash
vibeanvil build iterate --provider claude-code
vibeanvil build auto --provider claude-code
```

---

## Command Provider

**Best for:** Any command-line AI agent (Aider, custom scripts, etc.).

### Configuration

Set environment variables:

| Variable | Required | Description |
|----------|----------|-------------|
| `VIBEANVIL_PROVIDER_COMMAND` | Yes | Command name (e.g., `aider`) |
| `VIBEANVIL_PROVIDER_ARGS` | No | Extra arguments (space-separated) |
| `VIBEANVIL_PROVIDER_MODE` | No | How to pass prompt: `stdin` (default), `arg`, or `file` |
| `VIBEANVIL_PROVIDER_TIMEOUT` | No | Timeout in seconds (default: 300) |

### Examples

**Using Aider:**
```bash
export VIBEANVIL_PROVIDER_COMMAND=aider
export VIBEANVIL_PROVIDER_ARGS="--yes --message"
export VIBEANVIL_PROVIDER_MODE=arg

vibeanvil build iterate --provider command
```

**Using a custom script:**
```bash
export VIBEANVIL_PROVIDER_COMMAND=./my-ai-script.sh
export VIBEANVIL_PROVIDER_MODE=stdin

vibeanvil build iterate --provider command
```

### Modes

- **stdin**: Prompt is written to the command's stdin (default)
- **arg**: Prompt is passed as the final command argument
- **file**: Prompt is written to a temp file, path passed as argument

---

## Patch Provider

**Best for:** Diff-based workflows where AI outputs unified diffs.

### Requirements

- `git` must be installed

### Two-Step Workflow

**Step 1: Generate prompt**
```bash
vibeanvil build iterate --provider patch
# This creates a prompt file instructing the AI to output a unified diff
```

**Step 2: Apply the diff**
```bash
# Save the AI's diff output to a file
# Set the path and re-run
export VIBEANVIL_PATCH_FILE=changes.patch
vibeanvil build iterate --provider patch
```

### Safety

The patch provider includes safety checks:
- ❌ Rejects absolute paths
- ❌ Rejects path traversal (`../`)
- ❌ Rejects modifications outside repo root
- ✅ Validates diff before applying

---

## Manual Mode (No Provider)

You can always use manual mode without any AI provider:

```bash
vibeanvil build manual start
# ... make your changes manually ...
vibeanvil build manual evidence
vibeanvil build manual complete
```

---

## Choosing a Provider

| Your Setup | Recommended Provider |
|------------|---------------------|
| VS Code + GitHub Copilot | `human` |
| Cursor | `human` |
| Claude Code CLI | `claude-code` |
| Aider or CLI agents | `command` |
| API-based LLM (OpenAI, etc.) | `patch` |
| No AI / Manual coding | Manual mode |

---

## Environment Variables Summary

| Variable | Provider | Description |
|----------|----------|-------------|
| `VIBEANVIL_PROVIDER_COMMAND` | command | External command name |
| `VIBEANVIL_PROVIDER_ARGS` | command | Extra arguments |
| `VIBEANVIL_PROVIDER_MODE` | command | `stdin`, `arg`, or `file` |
| `VIBEANVIL_PROVIDER_TIMEOUT` | command | Timeout in seconds |
| `VIBEANVIL_PATCH_FILE` | patch | Path to unified diff file |
