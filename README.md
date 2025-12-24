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
  <a href="#-features">Features</a> •
  <a href="#-installation">Installation</a> •
  <a href="#-workflow">Workflow</a> •
  <a href="#-commands">Commands</a> •
  <a href="#-brainpack">BrainPack</a> •
  <a href="#-contributing">Contributing</a>
</p>

---

## 🌟 Overview

**VibeAnvil** là một CLI production-grade thực thi quy trình phát triển **contract-first** với đầy đủ evidence capture và audit trails. Được build bằng Rust, ship dưới dạng single binary cross-platform không cần runtime dependencies.

### ✨ Tại sao chọn VibeAnvil?

| Feature | Description |
|---------|-------------|
| 🔒 **Contract-First** | Enforced state machine từ intake đến ship |
| 📋 **Evidence & Audit** | JSONL audit trail với secret redaction |
| 🔄 **Build Modes** | Manual, auto, iterate (test/lint/fix loop) |
| 🧠 **BrainPack** | Dynamic repo harvesting vào searchable knowledge base |
| 🔌 **Provider Plugins** | Claude Code CLI adapter với extension points |
| 🔐 **Privacy-First** | Anonymized source IDs, không lưu external URLs |

---

## 🚀 Features

### 🔐 Contract Locking
```
"Contract LOCKED = License to Build"
```
- SHA-256 hash của contract
- Immutable sau khi lock
- Validation trước khi cho phép build

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
- Git diff tự động capture
- Build/test/lint logs
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
vibeanvil plan --provider claude-code

# 6️⃣ Build with iterate mode
vibeanvil build iterate --max 5 --evidence

# 7️⃣ Review and ship
vibeanvil review pass
vibeanvil snapshot --message "v1.0.0 ready"
vibeanvil ship --tag v1.0.0
```

---

## 📖 Commands

### 🛠️ Core Commands

| Command | Description | Example |
|---------|-------------|---------|
| `init` | Initialize workspace | `vibeanvil init` |
| `intake` | Capture requirements | `vibeanvil intake -m "Build X"` |
| `blueprint` | Generate blueprint | `vibeanvil blueprint --auto` |
| `contract` | Manage contract | `vibeanvil contract create` |
| `plan` | Create impl plan | `vibeanvil plan` |
| `build` | Execute build | `vibeanvil build iterate` |
| `review` | Review changes | `vibeanvil review pass` |
| `snapshot` | Create snapshot | `vibeanvil snapshot -m "v1"` |
| `ship` | Mark as shipped | `vibeanvil ship --tag v1.0` |

### 🧠 BrainPack Commands

| Command | Description | Example |
|---------|-------------|---------|
| `harvest` | Harvest repos | `vibeanvil harvest --query "cli"` |
| `brain stats` | View statistics | `vibeanvil brain stats` |
| `brain search` | Search brain | `vibeanvil brain search "pattern"` |
| `brain export` | Export data | `vibeanvil brain export --format md` |

### 📊 Utility Commands

| Command | Description | Example |
|---------|-------------|---------|
| `status` | Show status | `vibeanvil status -v` |
| `log` | View audit log | `vibeanvil log -n 50` |

---

## 🔨 Build Modes

### 📝 Manual Mode
```bash
vibeanvil build manual start
# ... make changes ...
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
  --max 10 \           # Max iterations
  --strict \           # Fail on first error
  --timeout 300 \      # Timeout per iteration
  --evidence           # Capture evidence
```

---

## 🧠 BrainPack

### 🔍 Harvest Repos

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
vibeanvil brain export --format jsonl

# Markdown format
vibeanvil brain export --format md

# Include source IDs
vibeanvil brain export --include-source-ids=true
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

### 🕵️ Privacy-First

- No external URLs stored by default
- Anonymized source IDs (SHA-256 hash)
- Clean exports exclude identifiers

### 📝 Audit Trail

All operations logged to `.vibeanvil/logs/audit.jsonl`:
- Command executed
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

<p align="center">
  Made with ❤️ by <a href="https://github.com/ThanhNguyxn">ThanhNguyxn</a>
</p>

<p align="center">
  <a href="https://github.com/ThanhNguyxn/vibeanvil/stargazers">⭐ Star this repo</a> •
  <a href="https://github.com/ThanhNguyxn/vibeanvil/issues">🐛 Report Bug</a> •
  <a href="https://github.com/ThanhNguyxn/vibeanvil/issues">💡 Request Feature</a>
</p>
