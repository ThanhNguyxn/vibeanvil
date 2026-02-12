# 🔧 Commands Reference

Complete reference for all VibeAnvil CLI commands.

---

## 📋 Table of Contents

- [Core Workflow Commands](#-core-workflow-commands)
- [Brain Commands](#-brain-commands)
- [Utility Commands](#-utility-commands)
- [Prompt Commands](#-prompt-commands)
- [Provider Commands](#-provider-commands)
- [Global Options](#-global-options)

---

## 🔄 Core Workflow Commands

### `init` - Initialize Workspace

```bash
vibeanvil init [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `-f, --force` | Force re-initialization |

**Examples:**
```bash
# Normal initialization
vibeanvil init

# Force re-init (resets state)
vibeanvil init --force
```

---

### `intake` - Capture Requirements

```bash
vibeanvil intake [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `-m, --message <MSG>` | Requirement message |

**Examples:**
```bash
# With inline message
vibeanvil intake -m "Build a REST API with JWT auth"

# Interactive mode
vibeanvil intake
```

---

### `blueprint` - Generate Blueprint

```bash
vibeanvil blueprint [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `-a, --auto` | Auto-generate from intake |

**Examples:**
```bash
# Manual creation
vibeanvil blueprint

# Auto-generate
vibeanvil blueprint --auto
```

---

### `contract` - Manage Contract

```bash
vibeanvil contract <ACTION>
```

| Action | Description |
|--------|-------------|
| `create` | Create new contract |
| `validate` | Validate contract |
| `lock` | Lock contract (permanent!) |
| `status` | Show contract status |

**Examples:**
```bash
# Create → Validate → Lock flow
vibeanvil contract create
vibeanvil contract validate
vibeanvil contract lock

# Check status
vibeanvil contract status
```

---

### `plan` - Create Implementation Plan

```bash
vibeanvil plan [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `-p, --provider <NAME>` | AI provider (default: claude-code) |

> **Note:** `plan` automatically scans your codebase ("Smart Context") to provide the AI with a map of your project's structure.

**Examples:**
```bash
# Default provider
vibeanvil plan

# Specify provider
vibeanvil plan --provider claude-code
```

---

### `build` - Execute Build

```bash
vibeanvil build <MODE> [OPTIONS]
```

| Mode | Description |
|------|-------------|
| `manual` | Step-by-step manual build |
| `auto` | Single-shot AI build |
| `iterate` | Test/lint/fix loop |

| Option | Description |
|--------|-------------|
| `-p, --provider <NAME>` | AI provider |
| `--max <N>` | Max iterations (iterate mode) |
| `--strict` | Fail on first error |
| `--timeout <SECS>` | Per-iteration timeout |
| `--no-test` | Skip tests |
| `--no-lint` | Skip linting |
| `--evidence` | Capture evidence |
| `--watch` | Watch for file changes |
| `--resume` | Resume last session |

**Examples:**
```bash
# Manual build
vibeanvil build manual start
vibeanvil build manual evidence
vibeanvil build manual complete
# (Triggers Interactive Auto-Commit: Confirm/Edit/Cancel generated message)

# Auto build with evidence
vibeanvil build auto --evidence

# Iterate with max 5 attempts
vibeanvil build iterate --max 5 --evidence

# Strict mode
vibeanvil build iterate --max 3 --strict
```

---

### `review` - Review Build

```bash
vibeanvil review <ACTION>
```

| Action | Description |
|--------|-------------|
| `start` | Start review |
| `pass` | Pass review |
| `fail` | Fail review |
| `status` | Check review status |

**Examples:**
```bash
vibeanvil review start
# ... review the code ...
vibeanvil review pass  # or: vibeanvil review fail
```

---

### `snapshot` - Create Snapshot

```bash
vibeanvil snapshot [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `-m, --message <MSG>` | Snapshot message |

**Examples:**
```bash
vibeanvil snapshot -m "Before major refactor"
```

---

### `ship` - Ship Project

```bash
vibeanvil ship [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `-t, --tag <TAG>` | Version tag |
| `-m, --message <MSG>` | Ship message |

**Examples:**
```bash
vibeanvil ship --tag v1.0.0 -m "Initial release"
```

---

## 🧠 Brain Commands

### `harvest` - Harvest GitHub Repos

```bash
vibeanvil harvest [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `-q, --query <Q>` | Search query (repeatable) |
| `-t, --topic <T>` | Topic filter (repeatable) |
| `-l, --language <L>` | Language filter |
| `--max-repos <N>` | Max repos (default: 20) |
| `--min-stars <N>` | Min stars (default: 10) |
| `--updated-within-days <N>` | Freshness (default: 365) |
| `--download <METHOD>` | tarball or git |
| `--ignore-glob <GLOB>` | Ignore patterns |
| `--allow-glob <GLOB>` | Allow patterns |

**Examples:**
```bash
# Search by topic
vibeanvil harvest -t rust -t cli --max-repos 10

# Search by query
vibeanvil harvest -q "machine learning" -l python --min-stars 100

# Multiple queries
vibeanvil harvest -q "react hooks" -q "state management" --max-repos 30
```

---

### `brain` - Manage BrainPack

```bash
vibeanvil brain <COMMAND>
```

#### `brain ensure`
Install Core BrainPack (curated templates). Safe to run repeatedly.

```bash
vibeanvil brain ensure
```

Output:
```
🧠 Core BrainPack Setup
═══════════════════════════════════════

  ✅ Core BrainPack installed successfully!
  📦 Imported entries successfully

💡 Quick starts:
  • vibeanvil brain search 'web contract'
  • vibeanvil brain search 'acceptance criteria'
```

#### `brain stats`
```bash
vibeanvil brain stats [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `--json` | JSON output (machine-readable) |

Output:
```
🧠 BrainPack Statistics

  Sources:        42
  Records:        15,234
  Chunks:         8,721
  JSONL size:     12.5 MB
  SQLite size:    45.2 MB
```

#### `brain search`
```bash
vibeanvil brain search <QUERY> [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `-n, --limit <N>` | Max results (default: 10) |
| `-t, --record-type <TYPE>` | Filter by type (function, class, doc) |
| `-l, --language <LANG>` | Filter by language (rust, python, js) |

**Examples:**
```bash
# Basic search
vibeanvil brain search "authentication middleware"

# Filter by language
vibeanvil brain search "error" -l rust

# Filter by type
vibeanvil brain search "parse" -t function

# Combined filters
vibeanvil brain search "async" -l rust -t function -n 5
```

#### `brain export`
```bash
vibeanvil brain export <jsonl|md> [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `-o, --output <PATH>` | Output file |
| `--include-source-ids` | Include source IDs (default: excluded for privacy) |
| `--limit <N>` | Limit entries for md export (default 50, 0=no limit) |

**Examples:**
```bash
# Export to JSONL
vibeanvil brain export jsonl -o brain.jsonl

# Export to Markdown (preview, default 50 entries)
vibeanvil brain export md -o brain.md

# Export more entries
vibeanvil brain export md --limit 200

# Export all entries (no limit)
vibeanvil brain export md --limit 0
```

#### `brain pack`
```bash
vibeanvil brain pack [OPTIONS]
```

Pack the current codebase into a single AI-friendly file. Works with any AI coding assistant (Copilot, Cursor, Claude, ChatGPT, etc.).

| Option | Description |
|--------|-------------|
| `-o, --output <FILE>` | Output file (default: `context.xml`) |
| `-f, --format <FMT>` | Format: `xml` (default) or `markdown` |

**Examples:**
```bash
# Pack to XML (best for Claude/Anthropic)
vibeanvil brain pack

# Pack to Markdown
vibeanvil brain pack --format md -o context.md

# Custom output
vibeanvil brain pack -o my_project.xml
```

---

## 🛠️ Utility Commands

### `doctor` - Check System Health

Check system and workspace health. Runs diagnostics on git, rust, and workspace state.

```bash
vibeanvil doctor
```

---

### `wizard` - Interactive Wizard

Interactive wizard menu for common workflows.

```bash
vibeanvil wizard
```

---

### `status` - Show Status

```bash
vibeanvil status [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `-v, --verbose` | Verbose output |
| `--json` | JSON output (machine-readable) |

---

### `log` - View Audit Log

```bash
vibeanvil log [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `-n, --lines <N>` | Lines to show (default: 20) |
| `--json` | JSON output |

---

### `update` - Check for Updates

```bash
vibeanvil update
```

---

### `upgrade` - Self-Update

```bash
vibeanvil upgrade
```

---

### `undo` - Undo Last AI Change

Revert the last commit (typically an AI-made change).

```bash
vibeanvil undo [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `--dry-run` | Show what would be undone without undoing |

**Examples:**
```bash
# Preview what would be undone
vibeanvil undo --dry-run

# Actually undo the last commit
vibeanvil undo
```

> **Note:** Changes are kept staged after undo. Run `git diff --cached` to review them.

---

## 🧩 Prompt Commands

### `prompt` - Print Prompt Templates

Print built-in prompt templates for different roles and workflows.

```bash
vibeanvil prompt <KIND>
```

| Kind | Description |
|------|-------------|
| `install` | Print the LLM paste-in installer prompt |
| `architect` | Print the Architect vibecode prompt |
| `developer` | Print the Developer vibecode prompt |
| `qa` | Print the QA vibecode prompt |
| `plan` | Print the Plan vibecode prompt |
| `review` | Print the Review vibecode prompt |
| `commit` | Print the Commit vibecode prompt |
| `debug` | Print the debugging investigation prompt |
| `xray` | Print the codebase analysis prompt |
| `vision` | Print the project vision prompt |
| `security` | Print the security audit prompt |
| `migrate` | Print the migration planning prompt |
| `refactor` | Print the safe refactoring prompt |

**Examples:**
```bash
# Print installer prompt for VibeAnvil
vibeanvil prompt install

# Print security audit prompt
vibeanvil prompt security
```

---

## 🔌 Provider Commands

### `providers` - List AI Providers

```bash
vibeanvil providers
```

Lists all supported AI providers and checks their availability.

**Output:**
```
Available Providers:
  ✅ Available human
  ❌ Not available claude-code
  ✅ Available command
  ✅ Available patch
```

---

## ⚙️ Global Options

| Option | Description |
|--------|-------------|
| `-h, --help` | Print help |
| `-V, --version` | Print version |

---

## 💖 Support VibeAnvil

If you find VibeAnvil useful, consider supporting the project:

[![GitHub Sponsors](https://img.shields.io/badge/Sponsor-❤️-ea4aaa?style=for-the-badge&logo=github)](https://github.com/sponsors/ThanhNguyxn)
[![Buy Me A Coffee](https://img.shields.io/badge/Buy%20Me%20A%20Coffee-☕-ffdd00?style=for-the-badge&logo=buy-me-a-coffee)](https://buymeacoffee.com/thanhnguyxn)

---

Made with ❤️ by the VibeAnvil team
