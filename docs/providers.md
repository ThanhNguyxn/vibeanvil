# AI Providers

VibeAnvil is **AI-agnostic** — you can use it with any AI coding assistant. This document describes each available provider and when to use it.

## Quick Reference

| Provider | Best For | Requirements |
|----------|----------|--------------|
| **IDE AI (with CLI)** | | |
| `cursor` | Cursor AI automation | Cursor CLI installed |
| `cline` | Cline autonomous agent | `npm install -g cline` |
| `continue` | Continue dev headless | `npm i -g @continuedev/cli` |
| `cody` | Sourcegraph Cody | `npm i -g @sourcegraph/cody` |
| **IDE AI (prompt-based)** | | |
| `copilot` | GitHub Copilot | None (generates prompt) |
| `zed` | Zed AI assistant | None (generates prompt) |
| `windsurf` | Codeium Windsurf | None (generates prompt) |
| `trae` | ByteDance Trae | None (generates prompt) |
| `jetbrains` | JetBrains AI | None (generates prompt) |
| `supermaven` | Supermaven | None (generates prompt) |
| `gemini-assist` | Google Gemini Code Assist | None (generates prompt) |
| **Terminal Agents** | | |
| `claude-code` | Claude Code CLI | `@anthropic-ai/claude-code` |
| `aider` | AI pair programming | `pip install aider-chat` |
| `opencode` | Terminal TUI | `go install crush` |
| `goose` | Block's AI agent | `brew install --cask block-goose` |
| **Cloud AI** | | |
| `kiro` | AWS Kiro (Amazon Q) | `kiro-cli` installed |
| **Self-Hosted AI** | | |
| `tabby` | Tabby ML server | Tabby server running |
| `tabby/<model>` | Tabby with model | e.g., `tabby/StarCoder-7B` |
| `ollama` | Local LLM (default) | Ollama installed + running |
| `ollama/<model>` | Any Ollama model | e.g., `ollama/llama3.2` |
| **Model Shortcuts** | | |
| `llama`, `codellama`, `deepseek`... | Popular Ollama models | Ollama + model pulled |
| **Generic** | | |
| `human` | Generic IDE prompt | None (always available) |
| `command` | Any CLI agent | Command + env vars |
| `patch` | Diff-based workflows | `git` installed |
| `mock` | Testing | None |

## Which Provider Should I Use?

| If you use... | Choose | Why |
|---------------|--------|-----|
| **GitHub Copilot** | `copilot` | Optimized prompt for Copilot Chat |
| **Cursor** | `cursor` | CLI automation with headless mode |
| **Zed** | `zed` | Prompt for Zed's built-in AI |
| **Windsurf** | `windsurf` | Prompt for Cascade mode |
| **Trae** | `trae` | Prompt for SOLO autonomous mode |
| **JetBrains IDE** | `jetbrains` | IntelliJ, PyCharm, WebStorm AI |
| **VS Code + Cline** | `cline` | Full autonomous agent |
| **Continue** | `continue` | Headless automation |
| **Sourcegraph Cody** | `cody` | Codebase context awareness |
| **Supermaven** | `supermaven` | Fast 1M token context |
| **Gemini Code Assist** | `gemini-assist` | Google's AI assistant |
| **Claude CLI** | `claude-code` | Direct CLI automation |
| **Aider** | `aider` | Best OSS terminal agent |
| **Goose** | `goose` | Block's autonomous agent |
| **AWS Kiro** | `kiro` | Enterprise AWS-backed |
| **Self-hosted Tabby** | `tabby` | Zero cloud dependency |
| **Local Models** | `ollama` | Privacy-first, zero API cost |
| **Other CLI Agent** | `command` | Runs any CLI command |
| **API + Custom Script** | `patch` | Generate and apply diffs |

> **Tip:** Not sure? Start with `copilot` for GitHub Copilot or `human` for a generic prompt.

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
# 5. Paste content into your IDE assistant
# 6. Apply the AI's suggestions in your IDE

# 7. Capture evidence and complete
vibeanvil build manual evidence
vibeanvil build manual complete
```

---

## GitHub Copilot Provider

**Best for:** GitHub Copilot Chat in VS Code, JetBrains, or Neovim.

### Usage

```bash
vibeanvil build iterate --provider copilot
```

### Workflow

1. Command generates `copilot_prompt.md` in your session directory
2. Open GitHub Copilot Chat in your IDE
3. Paste the prompt content
4. Apply suggested changes
5. Run `vibeanvil build manual evidence`

---

## Zed AI Provider

**Best for:** Zed editor's built-in AI assistant.

### Usage

```bash
vibeanvil build iterate --provider zed
```

### Workflow

1. Command generates `zed_prompt.md`
2. Open Zed, press `Cmd+Enter` (Mac) or `Ctrl+Enter` (Linux)
3. Paste the prompt, use `/file` to add context
4. Apply changes

---

## Windsurf Provider

**Best for:** Codeium's Windsurf IDE with Cascade mode.

### Usage

```bash
vibeanvil build iterate --provider windsurf
```

### Features

- **Cascade** (`Cmd+I`): Autonomous coding mode
- **Flow** (`Cmd+L`): Inline editing
- Use `@file`, `@folder`, `@docs` for context

---

## Trae Provider

**Best for:** ByteDance's Trae IDE with SOLO mode.

### Usage

```bash
vibeanvil build iterate --provider trae
```

### Features

- **SOLO mode**: Autonomous planning, coding, debugging
- **Builder mode**: Step-by-step guidance
- **Chat mode**: Quick questions

---

## JetBrains AI Provider

**Best for:** JetBrains IDEs (IntelliJ, PyCharm, WebStorm, etc.)

### Usage

```bash
vibeanvil build iterate --provider jetbrains
# Aliases: intellij, idea
```

### Features

- Press `Alt+Enter` on code for AI suggestions
- AI Chat panel for conversations
- Commands: `/explain`, `/refactor`, `/tests`

---

## Supermaven Provider

**Best for:** Supermaven's fast AI completions with 1M token context.

### Usage

```bash
vibeanvil build iterate --provider supermaven
```

### Features

- 1M token context window
- Fast inline completions
- Works with VS Code, JetBrains, Neovim

---

## Gemini Code Assist Provider

**Best for:** Google's Gemini Code Assist in VS Code/JetBrains.

### Usage

```bash
vibeanvil build iterate --provider gemini-assist
# Alias: gemini-code
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

## Aider Provider

**Best for:** AI pair programming with automatic git integration.

### Requirements

```bash
pip install aider-chat
# or
pipx install aider-chat
```

### Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `ANTHROPIC_API_KEY` | - | For Claude models (recommended) |
| `OPENAI_API_KEY` | - | For OpenAI models |
| `AIDER_MODEL` | claude-3-sonnet | Model to use |
| `AIDER_AUTO_COMMITS` | true | Enable git auto-commits |
| `AIDER_EXTRA_ARGS` | - | Additional CLI arguments |

### Usage

```bash
vibeanvil build iterate --provider aider
```

### Features

- **RepoMap**: Automatic codebase understanding
- **Auto-commit**: Git commits with AI-generated messages
- **Watch mode**: Monitor files for AI instruction comments
- **Voice input**: Speak your coding requests
- **100+ language support**

### Local Model Support

```bash
# Use with Ollama
export AIDER_MODEL=ollama/llama3.2

# Use with local endpoint
export AIDER_MODEL=openai/local-model
export OPENAI_API_BASE=http://localhost:8080/v1
```

---

## Ollama Provider

**Best for:** Local LLM inference with zero API cost.

### Requirements

```bash
# Install Ollama from https://ollama.com
# macOS:
brew install ollama

# Start the server
ollama serve

# Pull any model you want to use
ollama pull llama3.2
ollama pull codellama
ollama pull deepseek-coder-v2
```

### Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `OLLAMA_HOST` | http://localhost:11434 | Ollama server URL |
| `OLLAMA_MODEL` | llama3.2 | Default model |
| `OLLAMA_CONTEXT_SIZE` | 8192 | Context window size |
| `OLLAMA_TEMPERATURE` | 0.7 | Sampling temperature |

### Usage

```bash
# Default model (uses OLLAMA_MODEL or llama3.2)
vibeanvil build iterate --provider ollama

# Use ANY Ollama model with dynamic syntax
vibeanvil build iterate --provider ollama/codellama
vibeanvil build iterate --provider ollama/deepseek-coder-v2
vibeanvil build iterate --provider ollama/qwen2.5-coder:14b
vibeanvil build iterate --provider ollama/llama3.2:70b

# Or use convenient shortcuts (no version hardcoding!)
vibeanvil build iterate --provider codellama
vibeanvil build iterate --provider deepseek
vibeanvil build iterate --provider qwen
```

### Model Shortcuts

For convenience, these shortcuts map to popular models:

| Shortcut | Model | Best For |
|----------|-------|----------|
| `llama` | llama3.2 | General purpose |
| `codellama` | codellama | Code generation |
| `deepseek` | deepseek-coder-v2 | Code + reasoning |
| `qwen` | qwen2.5-coder | Code generation |
| `mistral` | mistral | General purpose |
| `mixtral` | mixtral | Large context |
| `phi` | phi3 | Fast & small |
| `gemma` | gemma2 | Google's model |
| `starcoder` | starcoder2 | Code completion |
| `codegemma` | codegemma | Code-focused Gemma |
| `wizardcoder` | wizardcoder | Coding assistant |
| `codestral` | codestral | Mistral's code model |
| `granite-code` | granite-code | IBM's code model |
| `yi-coder` | yi-coder | 01.AI's code model |
| `stable-code` | stable-code | Stability AI code |

> **Flexible versioning:** Shortcuts use base model names without hardcoded versions.
> Ollama will use the default tag (usually `:latest`). For specific sizes/versions:
> ```bash
> # Use any version you want
> vibeanvil build iterate --provider ollama/llama3.3:70b
> vibeanvil build iterate --provider ollama/qwen3:32b
> vibeanvil build iterate --provider ollama/deepseek-r1:8b
> 
> # Future models work automatically!
> vibeanvil build iterate --provider ollama/llama4
> vibeanvil build iterate --provider ollama/gpt-5-local  # hypothetical
> ```

### Recommended Models for Coding

**Best coding models (as of 2024-2025):**

```bash
# Top tier - best quality
ollama pull deepseek-coder-v2:16b
ollama pull qwen2.5-coder:14b
ollama pull codellama:34b

# Mid tier - good balance
ollama pull deepseek-coder-v2
ollama pull qwen2.5-coder:7b
ollama pull codegemma:7b

# Lightweight - fast
ollama pull deepseek-coder:1.3b
ollama pull qwen2.5-coder:1.5b
ollama pull phi3
```

### Features

- **100+ models** available
- **OpenAI-compatible API**
- **Zero cost** per token
- **Full privacy** - data never leaves your machine
- **Offline capable**

---

## OpenCode/Crush Provider

**Best for:** Terminal AI with beautiful TUI and auto-compact.

> Note: OpenCode has been archived. Use Crush (successor).

### Requirements

```bash
go install github.com/ryboe/crush@latest
```

### Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `ANTHROPIC_API_KEY` | - | For Claude models |
| `OPENAI_API_KEY` | - | For OpenAI models |
| `OPENCODE_MODEL` | claude-sonnet-4 | Model to use |
| `OPENCODE_EXTRA_ARGS` | - | Additional CLI arguments |

### Usage

```bash
vibeanvil build iterate --provider opencode
# or
vibeanvil build iterate --provider crush
```

### Features

- **TUI** with Bubble Tea
- **Auto-compact** at 95% context limit
- **MCP integration** for tools
- **LSP support** for code intelligence
- **Session persistence**

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
| `VIBEANVIL_PROVIDER_TIMEOUT_SECS` | No | Timeout in seconds (default: 600). Legacy: `VIBEANVIL_PROVIDER_TIMEOUT` also works |

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
| `VIBEANVIL_PROVIDER_TIMEOUT_SECS` | command | Timeout in seconds (default: 600) |
| `VIBEANVIL_PATCH_FILE` | patch | Path to unified diff file |
| `VIBEANVIL_PATCH_MAX_FILES` | patch | Max files in patch (default: 50) |
| `VIBEANVIL_PATCH_MAX_ADDED_LINES` | patch | Max total added lines (default: 5000) |
| `VIBEANVIL_PATCH_MAX_FILE_ADDED_LINES` | patch | Max lines per file (default: 2000) |
| `VIBEANVIL_PATCH_MAX_BYTES` | patch | Max patch size in bytes (default: 2MB) |

---

## Security & Safety

VibeAnvil implements several safety measures to protect your codebase and prevent accidents.

### Timeout Protection

The `command` provider has a built-in timeout to prevent hung processes:

```bash
# Default timeout: 600 seconds (10 minutes)
# To increase for long-running agents:
export VIBEANVIL_PROVIDER_TIMEOUT_SECS=1800  # 30 minutes
```

If a command times out:
- The process is terminated automatically
- An error message shows how to increase the limit
- No partial changes are applied

### Patch Safety Limits

The `patch` provider validates all diffs before applying:

**Path Safety:**
- ❌ Absolute paths (Unix `/etc/` or Windows `C:\`)
- ❌ Path traversal (`../`)
- ❌ Modifications to `.git/` directory
- ✅ Only relative paths within your repo

**Size Limits (override with env vars if needed):**

| Limit | Default | Env Var |
|-------|---------|---------|
| Max files | 50 | `VIBEANVIL_PATCH_MAX_FILES` |
| Max total added lines | 5,000 | `VIBEANVIL_PATCH_MAX_ADDED_LINES` |
| Max lines per file | 2,000 | `VIBEANVIL_PATCH_MAX_FILE_ADDED_LINES` |
| Max patch size | 2MB | `VIBEANVIL_PATCH_MAX_BYTES` |

**Binary Protection:**
- Binary patches (`GIT binary patch`) are rejected
- Very long lines (>20KB) are rejected as likely binary/minified content

### Secret Redaction

All provider outputs are automatically scanned for secrets before display or storage:

**Redacted patterns:**
- GitHub tokens: `ghp_`, `github_pat_`, `gho_`, `ghu_`, `ghr_`
- OpenAI/Anthropic keys: `sk-`, `sk-ant-`
- AWS keys: `AKIA...`
- Bearer tokens and Authorization headers
- Generic `api_key`, `secret`, `password` patterns

**Example:** `ghp_abc123...` becomes `ghp_***REDACTED***`

### Generated Prompt Safety

Both `human` and `patch` providers include safety instructions in generated prompts:
- Do not paste secrets
- Do not modify files outside the repo
- Run tests after changes

---

## New Providers (IDE AI Assistants)

### Cursor Provider

**Best for:** Automating Cursor AI editor.

#### Requirements

Download Cursor from https://cursor.com. CLI is included.

#### Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `CURSOR_MODEL` | - | Model to use |
| `CURSOR_EXTRA_ARGS` | - | Additional CLI arguments |
| `CURSOR_TIMEOUT_SECS` | 600 | Timeout in seconds |

#### Usage

```bash
vibeanvil build iterate --provider cursor
```

---

### Cline Provider

**Best for:** Autonomous AI coding with file operations and browser automation.

#### Requirements

```bash
npm install -g cline
```

#### Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `ANTHROPIC_API_KEY` | - | For Claude models |
| `OPENAI_API_KEY` | - | For OpenAI models |
| `CLINE_MODEL` | - | Model to use |
| `CLINE_EXTRA_ARGS` | - | Additional CLI arguments |

#### Usage

```bash
vibeanvil build iterate --provider cline
```

---

### Continue Provider

**Best for:** Continue with headless automation mode.

#### Requirements

```bash
npm i -g @continuedev/cli
```

#### Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `CONTINUE_MODEL` | - | Model to use |
| `CONTINUE_EXTRA_ARGS` | - | Additional CLI arguments |

#### Usage

```bash
vibeanvil build iterate --provider continue
# or alias
vibeanvil build iterate --provider cn
```

---

### Cody Provider

**Best for:** Sourcegraph's AI with excellent codebase context.

#### Requirements

```bash
npm install -g @sourcegraph/cody
cody auth login --web
```

#### Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `SRC_ENDPOINT` | sourcegraph.com | Sourcegraph endpoint |
| `SRC_ACCESS_TOKEN` | - | Access token |
| `CODY_MODEL` | - | Model to use |
| `CODY_CONTEXT_REPO` | - | Repository for context |

#### Usage

```bash
vibeanvil build iterate --provider cody
```

---

## New Providers (Terminal Agents)

### Goose Provider

**Best for:** Block's autonomous AI agent with MCP support.

#### Requirements

```bash
# macOS
brew install --cask block-goose

# Other platforms: https://github.com/block/goose
```

#### Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `GOOSE_MODEL` | - | Model to use |
| `GOOSE_PROVIDER` | - | AI provider (openai, anthropic, etc.) |
| `GOOSE_EXTRA_ARGS` | - | Additional CLI arguments |

#### Usage

```bash
vibeanvil build iterate --provider goose
```

---

## New Providers (Cloud AI)

### Kiro Provider (AWS)

**Best for:** AWS-backed enterprise AI assistant.

#### Requirements

```bash
# macOS/Linux
curl -fsSL https://cli.kiro.dev/install | bash

# Login
kiro-cli login
```

#### Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `KIRO_EXTRA_ARGS` | - | Additional CLI arguments |

#### Usage

```bash
vibeanvil build iterate --provider kiro
# or alias
vibeanvil build iterate --provider amazon-q
```

---

## New Providers (Self-Hosted)

### Tabby Provider

**Best for:** Self-hosted AI on consumer GPUs.

#### Requirements

```bash
# Docker with GPU
docker run -it --gpus all -p 8080:8080 -v $HOME/.tabby:/data \
  tabbyml/tabby serve --model StarCoder-1B --device cuda
```

#### Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `TABBY_HOST` | http://localhost:8080 | Tabby server URL |
| `TABBY_MODEL` | - | Chat model |
| `TABBY_API_KEY` | - | API key (if auth enabled) |

#### Usage

```bash
# Default model
vibeanvil build iterate --provider tabby

# Specific model
vibeanvil build iterate --provider tabby/StarCoder-7B
```

---

## IDE-Only Tools (Use Human Provider)

Some AI tools don't have CLI/API access and must be used with the `human` provider:

| Tool | Type | Why Human Provider? |
|------|------|---------------------|
| **Windsurf** | Codeium IDE | IDE-only, no CLI |
| **Trae** | ByteDance IDE | IDE-only, no CLI |
| **Zed AI** | Zed Editor | Built-in, no external API |
| **Supermaven** | Extensions | VS Code/JetBrains only |
| **Gemini Code Assist** | Extensions | IDE extensions only |

For these tools:
```bash
# Generate prompt, paste into your IDE
vibeanvil build iterate --provider human
```
