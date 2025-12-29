<p align="center">
  <img src="https://img.shields.io/badge/🔨-VibeAnvil-8B5CF6?style=for-the-badge" alt="VibeAnvil"/>
</p>

<h1 align="center">VibeAnvil</h1>

<p align="center">
  <strong>Contract-first vibe coding with evidence, audit, and repo-brain harvesting</strong>
</p>

<p align="center">
  <a href="https://github.com/ThanhNguyxn/vibeanvil/actions"><img src="https://img.shields.io/github/actions/workflow/status/ThanhNguyxn/vibeanvil/ci.yml?style=flat-square&logo=github&label=CI" alt="CI"></a>
  <a href="https://github.com/ThanhNguyxn/vibeanvil/releases"><img src="https://img.shields.io/github/v/release/ThanhNguyxn/vibeanvil?style=flat-square&logo=github&label=Release" alt="Release"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-MIT-yellow?style=flat-square" alt="License: MIT"></a>
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/Rust-1.75+-orange?style=flat-square&logo=rust" alt="Rust"></a>
  <img src="https://img.shields.io/badge/Platform-Windows%20|%20macOS%20|%20Linux-blue?style=flat-square" alt="Platform">
</p>

<p align="center">
  <a href="https://github.com/sponsors/ThanhNguyxn"><img src="https://img.shields.io/badge/Sponsor-❤️-ea4aaa?style=for-the-badge&logo=github" alt="GitHub Sponsors"></a>
  <a href="https://buymeacoffee.com/thanhnguyxn"><img src="https://img.shields.io/badge/Buy%20Me%20A%20Coffee-☕-ffdd00?style=for-the-badge&logo=buy-me-a-coffee" alt="Buy Me A Coffee"></a>
</p>

<p align="center">
  <a href="#-features">Features</a> •
  <a href="#-installation">Installation</a> •
  <a href="#-workflow">Workflow</a> •
  <a href="#-commands">Commands</a> •
  <a href="#-brainpack">BrainPack</a> •
  <a href="docs/">📚 Docs</a> •
  <a href="#-contributing">Contributing</a>
</p>

---

## 🌟 Overview

**VibeAnvil** is a production-grade CLI that enforces a **contract-first** development workflow with full evidence capture and audit trails. Built with Rust, it ships as a single cross-platform binary with no runtime dependencies.

### ✨ Why VibeAnvil?

| Feature | Description |
|---------|-------------|
| 🔒 **Contract-First** | Enforced state machine from intake to ship |
| 📋 **Evidence & Audit** | JSONL audit trail with secret redaction |
| 🔄 **Build Modes** | Manual, auto, iterate (test/lint/fix loop) |
| 🧠 **BrainPack** | Dynamic repo harvesting into searchable knowledge base |
| 🔌 **AI-Agnostic** | Works with Copilot, Cursor, Claude Code, any CLI agent |
| 🌊 **Vibe Coding** | Smart Context (Repo Map) & Interactive Auto-Commit |
| 📦 **Context Pack** | Export codebase as single AI-friendly file (XML/MD) |
| 🔐 **Privacy-First** | Anonymized source IDs, no external URLs stored |
| 🛡️ **Security Hardened** | Path traversal protection, filename sanitization |
| ↩️ **Undo Command** | Instantly revert last AI change with `vibeanvil undo` |

---

## ⚡ 60-Second Quickstart

```bash
# 1. Install
curl -fsSL https://raw.githubusercontent.com/ThanhNguyxn/vibeanvil/main/install.sh | bash

# 2. Initialize workspace
vibeanvil init

# 3. Install Core BrainPack (curated templates included!)
vibeanvil brain ensure

# 4. Search for guidance
vibeanvil brain search "acceptance criteria"
vibeanvil brain search "web contract"

# 5. Start your workflow
vibeanvil intake -m "Build a REST API for user management"
vibeanvil blueprint --auto
vibeanvil contract create
vibeanvil contract lock   # 🔒 "Contract LOCKED = License to Build"
```

> 💡 **Core BrainPack ships with VibeAnvil!** No need to run `harvest` first.
> Use `brain search` immediately to find templates for contracts, plans, and best practices.

> 🔄 **Core BrainPack is embedded** at build time. After upgrading VibeAnvil, run:
> ```bash
> vibeanvil brain ensure --refresh-core
> ```
> This refreshes your local DB with the new embedded content.

📚 **Detailed guides:** [Getting Started](docs/getting-started.md) | [Workflow](docs/workflow.md) | [Commands](docs/commands.md) | [Providers](docs/PROVIDERS.md)

---

## 🚀 Features

### 🔐 Contract Locking
```
"Contract LOCKED = License to Build"
```
- SHA-256 hash of contract content
- Immutable once locked
- Validation required before build permission

### 📊 State Machine
```
INIT → INTAKE → BLUEPRINT → CONTRACT_DRAFT → CONTRACT_LOCKED
                                     ↓
                               PLAN_CREATED
                                     ↓
                             BUILD_IN_PROGRESS → BUILD_DONE
                                     ↓            ↓
                            REVIEW_FAILED ← → REVIEW_PASSED
                                                   ↓
                                               SHIPPED
```

### 🛡️ Evidence Capture
- Automatic git diff capture
- Build/test/lint log collection
- Secret redaction (API keys, tokens, passwords)
- Session-based organization

### 🧠 BrainPack Harvesting
- Dynamic GitHub search (user-driven queries)
- Signal detection (state machine, contract patterns, iterate loops)
- SQLite FTS5 full-text search
- Privacy-first: anonymized source IDs

---

## 📦 Installation

### 🐧 Linux / 🍎 macOS

```bash
curl -fsSL https://raw.githubusercontent.com/ThanhNguyxn/vibeanvil/main/install.sh | bash
```

### 🪟 Windows (PowerShell)

```powershell
irm https://raw.githubusercontent.com/ThanhNguyxn/vibeanvil/main/install.ps1 | iex
```

### 🔧 Build from Source

```bash
# Clone repository
git clone https://github.com/ThanhNguyxn/vibeanvil.git
cd vibeanvil

# Build release binary
cargo build --release

# Run
./target/release/vibeanvil --version
```

### 📋 Requirements

- **Rust**: 1.75+ (for building from source)
- **Git**: For evidence capture
- **GITHUB_TOKEN** (optional): For higher API rate limits

### 🔄 Update / Upgrade

```bash
# Check for updates
vibeanvil update

# Download and install latest version
vibeanvil upgrade
```

### 🗑️ Uninstall

**Windows:**
```powershell
irm https://raw.githubusercontent.com/ThanhNguyxn/vibeanvil/main/uninstall.ps1 | iex
```

**Linux/macOS:**
```bash
curl -fsSL https://raw.githubusercontent.com/ThanhNguyxn/vibeanvil/main/uninstall.sh | bash
```

---

## 🔄 Workflow

### Full Workflow Example

```bash
# 1️⃣ Initialize workspace
vibeanvil init

# 2️⃣ Capture requirements
vibeanvil intake --message "Build a CLI for task management"

# 3️⃣ Generate blueprint
vibeanvil blueprint --auto

# 4️⃣ Create and lock contract
vibeanvil contract create
# ✏️ Edit .vibeanvil/contracts/contract.json
vibeanvil contract validate
vibeanvil contract lock    # 🔒 "Contract LOCKED = License to Build"

# 5️⃣ Create implementation plan
vibeanvil plan

# 6️⃣ Build with your preferred AI provider
# Choose: human (Copilot/Cursor), claude-code, command (Aider), or patch
vibeanvil build iterate --provider human --max 5

# 7️⃣ Review and ship
vibeanvil review pass
vibeanvil snapshot --message "v1.0.0 ready"
vibeanvil ship --tag v1.0.0
```

> 📚 **See [docs/PROVIDERS.md](docs/PROVIDERS.md) for detailed provider setup**

---

## 📖 Commands

### 🛠️ Core Commands

| Command | Description | Example |
|---------|-------------|---------|
| `init` | Initialize workspace | `vibeanvil init` |
| `intake` | Capture requirements | `vibeanvil intake -m "Build X"` |
| `blueprint` | Generate blueprint | `vibeanvil blueprint --auto` |
| `contract` | Manage contract | `vibeanvil contract create` |
| `plan` | Create implementation plan | `vibeanvil plan` |
| `build` | Execute build | `vibeanvil build iterate` |
| `review` | Review changes | `vibeanvil review pass` |
| `snapshot` | Create snapshot | `vibeanvil snapshot -m "v1"` |
| `ship` | Mark as shipped | `vibeanvil ship --tag v1.0` |

### 🧠 BrainPack Commands

| Command | Description | Example |
|---------|-------------|---------|
| `harvest` | Harvest repositories | `vibeanvil harvest --query "cli"` |
| `brain stats` | View statistics | `vibeanvil brain stats` |
| `brain search` | Search brain | `vibeanvil brain search "pattern"` |
| `brain export` | Export data | `vibeanvil brain export md` |

### 📊 Utility Commands

| Command | Description | Example |
|---------|-------------|---------|
| `status` | Show workflow status | `vibeanvil status -v` |
| `log` | View audit log | `vibeanvil log -n 50` |
| `providers` | List AI providers | `vibeanvil providers` |

---

## 🔨 Build Modes

### 📝 Manual Mode
```bash
vibeanvil build manual start
# ... make your changes ...
vibeanvil build manual evidence
vibeanvil build manual complete
```

### 🤖 Auto Mode
```bash
vibeanvil build auto --provider claude-code
```

### 🔄 Iterate Mode
```bash
vibeanvil build iterate \
  --max 10 \           # Maximum iterations
  --strict \           # Fail on first error
  --timeout 300 \      # Timeout per iteration (seconds)
  --evidence           # Capture evidence
```

---

## 🤖 AI Providers

VibeAnvil works with **any AI coding assistant**. See [docs/PROVIDERS.md](docs/PROVIDERS.md) for details.

```bash
# List available providers
vibeanvil providers
```

### Using with GitHub Copilot / Cursor
```bash
# Use the human provider - generates a prompt file
vibeanvil build iterate --provider human

# 1. Open the generated prompt file (path shown in output)
# 2. Paste into Copilot/Cursor/VS Code Chat
# 3. Apply the AI's suggestions
# 4. Complete the build:
vibeanvil build manual evidence
vibeanvil build manual complete
```

### Using with CLI Agents (Aider, etc.)
```bash
# Configure your CLI agent
export VIBEANVIL_PROVIDER_COMMAND=aider
export VIBEANVIL_PROVIDER_ARGS="--yes --message"
export VIBEANVIL_PROVIDER_MODE=arg

# Run with command provider
vibeanvil build iterate --provider command
```

---

## 🧠 BrainPack

### 🔍 Harvest Repositories

```bash
vibeanvil harvest \
  --query "contract-first workflow cli" \
  --query "state machine rust" \
  --topic "cli" \
  --language rust \
  --max-repos 20 \
  --min-stars 50 \
  --updated-within-days 180
```

### 📊 View Statistics

```bash
vibeanvil brain stats
```

Output:
```
🧠 BrainPack Statistics

  Sources:        42
  Records:        1,234
  Chunks:         5,678
  JSONL size:     2,456,789 bytes
  SQLite size:    1,234,567 bytes

  By content type:
    code         890
    doc          234
    config       110
```

### 🔎 Search

```bash
vibeanvil brain search "state machine transition"
```

### 📤 Export

```bash
# JSONL format (privacy-clean)
vibeanvil brain export jsonl

# Markdown format
vibeanvil brain export md

# Include anonymized source IDs
vibeanvil brain export jsonl --include-source-ids
```

---

## 📁 Project Structure

### 📂 Workspace Layout

```
.vibeanvil/
├── 📄 state.json           # Current workflow state
├── 📄 intake.md            # Captured requirements
├── 📄 plan.md              # Implementation plan
├── 📁 contracts/
│   └── 📄 contract.json    # Project contract
├── 🔒 contract.lock        # Locked contract hash
├── 📁 blueprints/
│   └── 📄 blueprint.md     # Project blueprint
├── 📁 sessions/
│   └── 📁 <session-id>/
│       └── 📁 evidence/    # Build evidence
└── 📁 logs/
    └── 📄 audit.jsonl      # Audit trail
```

### 📜 Contract Schema

```json
{
  "schema_version": "1.0.0",
  "status": "LOCKED",
  "project_name": "My Project",
  "description": "Project description",
  "goals": ["Goal 1", "Goal 2"],
  "requirements": [
    {"id": "REQ-001", "description": "Must do X", "priority": "must"},
    {"id": "REQ-002", "description": "Should do Y", "priority": "should"}
  ],
  "acceptance_criteria": ["Tests pass", "Docs complete"],
  "constraints": ["Must use Rust"],
  "out_of_scope": ["Mobile support"]
}
```

---

## 🔐 Security

### 🛡️ Secret Redaction

Automatically redacts:
- 🔑 API keys (OpenAI, AWS, GitHub)
- 🎫 Bearer tokens
- 🔐 Passwords
- 📜 PEM private keys

### 🕵️ Privacy-First Design

- No external URLs stored by default
- Anonymized source IDs (SHA-256 hash)
- Clean exports exclude identifiers

### 📝 Audit Trail

All operations logged to `.vibeanvil/logs/audit.jsonl`:
- Commands executed
- State transitions
- Timestamps
- Session IDs

---

## 🌍 Environment Variables

| Variable | Description | Required |
|----------|-------------|----------|
| `GITHUB_TOKEN` | GitHub API token for harvesting | Optional |
| `RUST_LOG` | Logging level (info/debug/trace) | Optional |

---

## 🔧 Troubleshooting

### Common Issues

| Issue | Solution |
|-------|----------|
| "Workspace not initialized" | Run `vibeanvil init` first |
| "Wrong state" error | Run `vibeanvil status` to see current state and next step |
| "Contract already locked" | Contract is immutable once locked - start new workspace if needed |
| No search results | Run `vibeanvil brain ensure` to install Core BrainPack |
| Empty BrainPack | Run `vibeanvil brain ensure` or use `harvest` to populate |
| Rate limited by GitHub | Set `GITHUB_TOKEN` environment variable |

### Debug Commands

```bash
# Check current state and workflow progress
vibeanvil status -v

# View recent audit log entries
vibeanvil log -n 50

# Check BrainPack statistics
vibeanvil brain stats

# Check for updates
vibeanvil update
```

### Where Data Lives

| Location | Path | Scope |
|----------|------|-------|
| Workspace | `.vibeanvil/` (in project) | Per-project |
| BrainPack (Linux) | `~/.cache/vibeanvil/brainpack/` | User-level |
| BrainPack (macOS) | `~/Library/Caches/vibeanvil/brainpack/` | User-level |
| BrainPack (Windows) | `%LOCALAPPDATA%\vibeanvil\brainpack\` | User-level |

See [docs/DATA_LAYOUT.md](docs/DATA_LAYOUT.md) for complete details.

---

## 🤝 Contributing

Contributions are welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

```bash
# Run tests
cargo test

# Check lints
cargo clippy --all-targets

# Format code
cargo fmt
```

---

## 📄 License

This project is licensed under the **MIT License** - see [LICENSE](LICENSE) file.

---

## 🙏 Acknowledgments

- [Clap](https://github.com/clap-rs/clap) - CLI framework
- [Tokio](https://tokio.rs/) - Async runtime
- [SQLite](https://www.sqlite.org/) - Database with FTS5

---

## 💖 Support VibeAnvil

If you find VibeAnvil useful, consider supporting the project!

<p align="center">
  <a href="https://github.com/sponsors/ThanhNguyxn">
    <img src="https://img.shields.io/badge/Sponsor_on_GitHub-❤️-ea4aaa?style=for-the-badge&logo=github" alt="GitHub Sponsors">
  </a>
  <a href="https://buymeacoffee.com/thanhnguyxn">
    <img src="https://img.shields.io/badge/Buy_Me_A_Coffee-☕-ffdd00?style=for-the-badge&logo=buy-me-a-coffee" alt="Buy Me A Coffee">
  </a>
</p>

Your support helps us:
- 🚀 Add new features
- 🐛 Fix bugs faster
- 📚 Improve documentation
- 🌍 Support more platforms

---

<p align="center">
  Made with ❤️ by <a href="https://github.com/ThanhNguyxn">ThanhNguyxn</a>
</p>

<p align="center">
  <a href="https://github.com/ThanhNguyxn/vibeanvil/stargazers">⭐ Star this repo</a> •
  <a href="https://github.com/ThanhNguyxn/vibeanvil/issues">🐛 Report Bug</a> •
  <a href="https://github.com/ThanhNguyxn/vibeanvil/issues">💡 Request Feature</a>
</p>

