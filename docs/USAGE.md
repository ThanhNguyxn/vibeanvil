# 📖 Usage Guide

Comprehensive guide to using VibeAnvil commands with intent, prerequisites, and outputs.

---

## 📋 Table of Contents

- [Workflow Overview](#-workflow-overview)
- [Command Reference](#-command-reference)
  - [init](#init)
  - [intake](#intake)
  - [blueprint](#blueprint)
  - [contract](#contract)
  - [plan](#plan)
  - [build](#build)
  - [review](#review)
  - [snapshot](#snapshot)
  - [ship](#ship)
  - [status](#status)
  - [log](#log)
  - [brain](#brain)
  - [harvest](#harvest)
  - [update/upgrade](#updateupgrade)

---

## 🔄 Workflow Overview

| Step | Command | State |
|------|---------|-------|
| 1 | `init` | Init |
| 2 | `intake` | IntakeCaptured |
| 3 | `blueprint` | BlueprintGenerated |
| 4 | `contract lock` | ContractLocked |
| 5 | `plan` | Planned |
| 6 | `build` | Built |
| 7 | `review pass` | Reviewed |
| 8 | `snapshot` | SnapshotCreated |
| 9 | `ship` | Shipped |

The state machine enforces this order. You cannot skip states or go backwards.

---

## 📚 Command Reference

### `init` - Initialize Workspace

**Intent:** Create the `.vibeanvil/` directory structure and initial state.

**Prerequisites:** None (first command to run)

**Command:**
```bash
vibeanvil init [--force]
```

**Options:**
| Flag | Description |
|------|-------------|
| `-f, --force` | Reinitialize existing workspace (resets state!) |

**Outputs:**
- `.vibeanvil/` directory with subdirectories
- `.vibeanvil/state.json` - initial state
- `.vibeanvil/.gitignore` - excludes sensitive files

**Next Step:** `vibeanvil brain ensure` (recommended) or `vibeanvil intake`

**Example:**
```bash
cd my-project
vibeanvil init
# ✅ Workspace initialized successfully!
```

---

### `intake` - Capture Requirements

**Intent:** Document project requirements and user stories.

**Prerequisites:** Workspace initialized (run `init` first)

**Command:**
```bash
vibeanvil intake [-m <message>]
```

**Options:**
| Flag | Description |
|------|-------------|
| `-m, --message` | Requirement message (inline) |

**Outputs:**
- Updates `state.json` to `IntakeCaptured`
- Logs to `audit.jsonl`

**Next Step:** `vibeanvil blueprint`

**Examples:**
```bash
# Inline message
vibeanvil intake -m "Build a REST API with user authentication"

# Interactive (if no message)
vibeanvil intake
```

---

### `blueprint` - Generate Blueprint

**Intent:** Create a technical blueprint from requirements.

**Prerequisites:** Requirements captured (`intake` completed)

**Command:**
```bash
vibeanvil blueprint [--auto]
```

**Options:**
| Flag | Description |
|------|-------------|
| `-a, --auto` | Auto-generate from intake |

**Outputs:**
- `.vibeanvil/blueprints/blueprint.md` - blueprint document
- Updates `state.json` to `BlueprintGenerated`

**Next Step:** `vibeanvil contract create`

**Examples:**
```bash
# Auto-generate
vibeanvil blueprint --auto

# Manual/interactive
vibeanvil blueprint
```

---

### `contract` - Manage Contract

**Intent:** Create, validate, and lock the project contract.

**Prerequisites:** Blueprint generated

**Command:**
```bash
vibeanvil contract <action>
```

**Actions:**
| Action | Description |
|--------|-------------|
| `create` | Create contract from blueprint |
| `validate` | Validate contract structure |
| `lock` | Lock contract (permanent!) |
| `status` | Show contract status |

**Outputs:**
- `create`: `.vibeanvil/contracts/contract.json`
- `lock`: `.vibeanvil/contracts/contract.lock`
- Updates state accordingly

**Next Step:** After `lock` → `vibeanvil plan`

**Examples:**
```bash
# Full flow
vibeanvil contract create
vibeanvil contract validate
vibeanvil contract lock
# 🔒 Contract LOCKED = License to Build!

# Check status
vibeanvil contract status
```

> ⚠️ **Warning:** `contract lock` is permanent. The contract cannot be modified after locking.

---

### `plan` - Create Implementation Plan

**Intent:** Generate an implementation plan using AI provider.

**Prerequisites:** Contract locked

**Command:**
```bash
vibeanvil plan [--provider <name>]
```

**Options:**
| Flag | Description |
|------|-------------|
| `-p, --provider` | AI provider (default: `claude-code`) |

**Outputs:**
- Implementation plan (provider-dependent)
- Updates `state.json` to `Planned`

**Next Step:** `vibeanvil build`

**Example:**
```bash
vibeanvil plan --provider claude-code
```

---

### `build` - Execute Build

**Intent:** Execute the build using manual, auto, or iterate mode.

**Prerequisites:** Plan created

**Command:**
```bash
vibeanvil build <mode> [options] [action]
```

**Modes:**
| Mode | Description |
|------|-------------|
| `manual` | Step-by-step manual build with evidence capture |
| `auto` | Single-shot AI-driven build |
| `iterate` | Automated test/lint/fix loop |

**Options:**
| Flag | Description |
|------|-------------|
| `-p, --provider` | AI provider (default: `claude-code`) |
| `--max <N>` | Max iterations for iterate mode (default: 5) |
| `--strict` | Fail on first error |
| `--timeout <secs>` | Per-iteration timeout (default: 300) |
| `--no-test` | Skip test execution |
| `--no-lint` | Skip lint execution |
| `--evidence` | Capture evidence (diffs, logs) |

**Manual Actions:**
| Action | Description |
|--------|-------------|
| `start` | Start manual build session |
| `evidence` | Capture evidence mid-build |
| `complete` | Complete the manual build |

**Outputs:**
- Session created in `.vibeanvil/sessions/<id>/`
- Evidence in `.vibeanvil/sessions/<id>/evidence/`
- Updates `state.json` to `Built`

**Next Step:** `vibeanvil review start`

**Examples:**
```bash
# Manual build
vibeanvil build manual start
# ... do your work ...
vibeanvil build manual evidence
# ... continue working ...
vibeanvil build manual complete

# Auto build with evidence
vibeanvil build auto --evidence

# Iterate with strict mode
vibeanvil build iterate --max 5 --strict --evidence
```

---

### `review` - Review Build

**Intent:** Review the completed build and pass/fail it.

**Prerequisites:** Build completed

**Command:**
```bash
vibeanvil review <action>
```

**Actions:**
| Action | Description |
|--------|-------------|
| `start` | Start review process |
| `pass` | Pass the review |
| `fail` | Fail the review |
| `status` | Check review status |

**Outputs:**
- Updates `state.json` to `Reviewed` (on pass)

**Next Step:** `vibeanvil snapshot` or `vibeanvil ship`

**Example:**
```bash
vibeanvil review start
# ... review the code ...
vibeanvil review pass
```

---

### `snapshot` - Create Snapshot

**Intent:** Create a point-in-time snapshot of the project.

**Prerequisites:** Review passed

**Command:**
```bash
vibeanvil snapshot [-m <message>]
```

**Options:**
| Flag | Description |
|------|-------------|
| `-m, --message` | Snapshot description |

**Outputs:**
- Updates `state.json` to `SnapshotCreated`
- Logs snapshot event

**Next Step:** `vibeanvil ship`

**Example:**
```bash
vibeanvil snapshot -m "Pre-release snapshot"
```

---

### `ship` - Ship Project

**Intent:** Mark the project as shipped with a version tag.

**Prerequisites:** Snapshot created

**Command:**
```bash
vibeanvil ship [--tag <tag>] [-m <message>]
```

**Options:**
| Flag | Description |
|------|-------------|
| `-t, --tag` | Version tag (e.g., `v1.0.0`) |
| `-m, --message` | Ship message |

**Outputs:**
- Updates `state.json` to `Shipped`
- Final audit log entry

**Next Step:** Workflow complete! 🎉

**Example:**
```bash
vibeanvil ship --tag v1.0.0 -m "Initial release"
```

---

### `status` - Show Status

**Intent:** Display current workflow state and progress.

**Prerequisites:** Workspace initialized

**Command:**
```bash
vibeanvil status [--verbose]
```

**Options:**
| Flag | Description |
|------|-------------|
| `-v, --verbose` | Show detailed status with history |

**Outputs:** Console display of current state, progress, and next steps.

**Example:**
```bash
vibeanvil status
# Shows: Current state, workflow progress, next step hint
```

---

### `log` - View Audit Log

**Intent:** View the JSONL audit trail of all commands and events.

**Prerequisites:** Workspace initialized

**Command:**
```bash
vibeanvil log [-n <lines>] [--json]
```

**Options:**
| Flag | Description |
|------|-------------|
| `-n, --lines` | Number of lines (default: 20) |
| `--json` | Output raw JSON |

**Outputs:** Formatted or JSON output of recent audit events.

**Example:**
```bash
vibeanvil log -n 50
vibeanvil log --json
```

---

### `brain` - Manage BrainPack

**Intent:** Manage the searchable knowledge base.

**Prerequisites:** None (works globally)

**Subcommands:**

#### `brain ensure`
Install/verify Core BrainPack (50+ templates).

```bash
vibeanvil brain ensure
```

**Outputs:**
- Imports embedded Core BrainPack if not present
- Idempotent - safe to run repeatedly

#### `brain stats`
Show BrainPack statistics.

```bash
vibeanvil brain stats
```

**Outputs:** Sources, records, chunks, file sizes, breakdowns.

#### `brain search`
Search the knowledge base.

```bash
vibeanvil brain search "<query>" [-n <limit>]
```

**Options:**
| Flag | Description |
|------|-------------|
| `-n, --limit` | Max results (default: 10) |

**Example:**
```bash
vibeanvil brain search "acceptance criteria" -n 20
```

#### `brain export`
Export BrainPack to file.

```bash
vibeanvil brain export <format> [-o <path>] [--include-source-ids]
```

**Formats:** `jsonl`, `md`

**Example:**
```bash
vibeanvil brain export jsonl -o backup.jsonl
```

---

### `harvest` - Harvest GitHub Repos

**Intent:** Populate BrainPack by harvesting GitHub repositories.

**Prerequisites:** None (optional: set `GITHUB_TOKEN` for higher rate limits)

**Command:**
```bash
vibeanvil harvest [options]
```

**Key Options:**
| Flag | Description |
|------|-------------|
| `-q, --query` | Search query (repeatable) |
| `-t, --topic` | Topic filter (repeatable) |
| `-l, --language` | Language filter |
| `--max-repos` | Max repos (default: 20) |
| `--min-stars` | Min stars (default: 10) |
| `--updated-within-days` | Freshness (default: 365) |
| `--ignore-glob` | Ignore patterns (repeatable) |
| `--allow-glob` | Allow patterns (repeatable) |

**Outputs:**
- Records added to BrainPack SQLite
- JSONL appended to brainpack.jsonl

**Example:**
```bash
vibeanvil harvest -t rust -t cli --min-stars 100 --max-repos 10
```

See [DATA_SOURCES.md](DATA_SOURCES.md) for harvest presets and strategies.

---

### `update`/`upgrade` - Self-Update

**Intent:** Check for and install updates.

**Commands:**
```bash
vibeanvil update   # Check for updates
vibeanvil upgrade  # Download and install latest
```

**Outputs:**
- `update`: Shows current vs latest version
- `upgrade`: Downloads and replaces binary

**Example:**
```bash
vibeanvil update
# New version available: v0.3.0
vibeanvil upgrade
# ✅ Upgraded to v0.3.0!
```

---

## 🔧 Troubleshooting

### Common Issues

| Issue | Solution |
|-------|----------|
| "Workspace not initialized" | Run `vibeanvil init` |
| "Wrong state" error | Run `vibeanvil status` to see current state |
| "Contract already locked" | Cannot unlock - start new workspace if needed |
| No search results | Run `vibeanvil brain ensure` to install Core BrainPack |

### Debug Commands

```bash
# Check current state
vibeanvil status -v

# View recent activity
vibeanvil log -n 50

# Check BrainPack health
vibeanvil brain stats
```

---

Made with ❤️ by the VibeAnvil team
