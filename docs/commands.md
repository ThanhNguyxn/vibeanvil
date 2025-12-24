# 🔧 Commands Reference

Complete reference for all VibeAnvil CLI commands.

---

## 📋 Table of Contents

- [Core Workflow Commands](#-core-workflow-commands)
- [Brain Commands](#-brain-commands)
- [Utility Commands](#-utility-commands)
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

**Examples:**
```bash
# Manual build
vibeanvil build manual start
vibeanvil build manual evidence
vibeanvil build manual complete

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

#### `brain stats`
```bash
vibeanvil brain stats
```

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

**Examples:**
```bash
vibeanvil brain search "authentication middleware"
vibeanvil brain search "error handling" -n 20
```

#### `brain export`
```bash
vibeanvil brain export [FORMAT] [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `-o, --output <PATH>` | Output file |
| `--include-source-ids` | Include source IDs |

**Examples:**
```bash
# Export to JSONL
vibeanvil brain export jsonl -o brain.jsonl

# Export to Markdown
vibeanvil brain export md -o brain.md
```

---

## 🛠️ Utility Commands

### `status` - Show Status

```bash
vibeanvil status [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `-v, --verbose` | Verbose output |

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
