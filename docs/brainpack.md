# 🧠 BrainPack Guide

Learn how to harvest knowledge from GitHub and build your own searchable knowledge base.

---

## 🤔 What is BrainPack?

BrainPack is VibeAnvil's **privacy-first knowledge harvesting** system. It:

- 📥 Downloads code from GitHub repos
- 🔍 Chunks and indexes content for search
- 🔒 Anonymizes sources (SHA-256 hashed)
- 🚫 Never stores external URLs
- 💾 Uses SQLite FTS5 for fast full-text search

---

## 🚀 Quick Start

### Option 1: Install Core BrainPack (Recommended)

VibeAnvil ships with a **Core BrainPack** containing owner-authored templates for contracts, plans, evidence patterns, and more.
It also includes Vibecode workflow prompts and agent/command template schemas to bootstrap prompt packs.

```bash
# Install Core BrainPack (one-time, idempotent)
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

Now search immediately:
```bash
vibeanvil brain search "acceptance criteria"
vibeanvil brain search "iterate loop"
```

### Option 2: Harvest GitHub Repos

Extend your BrainPack with external content:

```bash
# Harvest top Rust CLI projects
vibeanvil harvest -t rust -t cli --min-stars 100 --max-repos 10
```

Output:
```
🌾 Harvesting GitHub Repos

  Query: topic:rust+topic:cli stars:>=100
  
  [1/10] ━━━━━━━━━━ clap-rs/clap ✓
  [2/10] ━━━━━━━━━━ burntsushi/ripgrep ✓
  [3/10] ━━━━━━━━━━ sharkdp/bat ✓
  ...
  
✓ Harvested 10 repos → 2,450 chunks
```

### 2. Check Your BrainPack

```bash
vibeanvil brain stats
```

Output:
```
🧠 BrainPack Statistics
════════════════════════════════════════

  Sources:        10
  Records:        1,234
  Chunks:         2,450
  JSONL size:     4.5 MB
  SQLite size:    12.3 MB

  By Language:
    • Rust          85%
    • Markdown      10%
    • TOML           5%
```

### 3. Search for Knowledge

```bash
vibeanvil brain search "error handling pattern"
```

Output:
```
🔍 Search Results for "error handling pattern"
════════════════════════════════════════════════

[1] Score: 0.95  │  Type: code  │  Lang: rust
────────────────────────────────────────────────
impl<T, E> Result<T, E> {
    pub fn map_err<F, O>(self, op: O) -> Result<T, F>
    where O: FnOnce(E) -> F {
        match self {
            Ok(t) => Ok(t),
            Err(e) => Err(op(e)),
        }
    }
}

[2] Score: 0.89  │  Type: code  │  Lang: rust
────────────────────────────────────────────────
// Use ? operator for propagating errors
fn read_config() -> Result<Config, Error> {
    let content = fs::read_to_string("config.toml")?;
    let config: Config = toml::from_str(&content)?;
    Ok(config)
}
```

---

## 📚 Detailed Usage

### Harvest Options

```bash
vibeanvil harvest [OPTIONS]
```

#### By Topic
```bash
# Single topic
vibeanvil harvest -t react

# Multiple topics (AND)
vibeanvil harvest -t react -t hooks -t typescript
```

#### By Search Query
```bash
# Search query
vibeanvil harvest -q "machine learning"

# Multiple queries
vibeanvil harvest -q "neural network" -q "deep learning"
```

#### By Language
```bash
vibeanvil harvest -t web -l typescript
vibeanvil harvest -q "api" -l go
```

#### Filtering
```bash
# High quality repos only
vibeanvil harvest -t rust --min-stars 500

# Recent activity
vibeanvil harvest -t rust --updated-within-days 90

# Limit count
vibeanvil harvest -t rust --max-repos 5
```

#### File Filtering
```bash
# Ignore patterns
vibeanvil harvest -t rust \
  --ignore-glob "*.md" \
  --ignore-glob "tests/**" \
  --ignore-glob "**/vendor/**"

# Allow only specific files
vibeanvil harvest -t rust \
  --allow-glob "*.rs" \
  --allow-glob "Cargo.toml"
```

---

## 🔒 Privacy Features

### Anonymized Sources

When harvesting, VibeAnvil:

1. Downloads repo content
2. Generates SHA-256 hash of `owner/repo@commit`
3. Stores only the hash as `source_id`
4. **Never stores:** URLs, owner names, repo names

```
Original: https://github.com/rust-lang/rust@abc123
Stored:   source_id: "a1b2c3d4e5f6..." (SHA-256 hash)
```

### No External URLs

- ✅ Code content stored locally
- ✅ Metadata anonymized
- ✅ Offline search capability
- ❌ No tracking
- ❌ No external requests during search

---

## 📤 Exporting BrainPack

### JSONL Format
```bash
vibeanvil brain export jsonl -o brain.jsonl
```

Output format (source_id redacted by default for privacy):
```json
{"source_id":"","commit":"abc123","license":"MIT","language":"rust","path":"src/main.rs","type":"code","signals":[],"summary":"...","chunks":[{"chunk_id":"...","text":"fn main()...","start_line":1,"end_line":10}],"tags":[]}
```

To include source IDs:
```bash
vibeanvil brain export jsonl --include-source-ids -o brain.jsonl
```

Output with source IDs:
```json
{"source_id":"a1b2c3...","commit":"abc123","license":"MIT","language":"rust","path":"src/main.rs","type":"code","signals":[],"summary":"...","chunks":[...],"tags":[]}
```

### Markdown Format
```bash
vibeanvil brain export md -o brain.md
```

Creates a readable markdown preview (default: 50 entries). Use `--limit` for more.

---

## 📦 Context Packaging (Local Codebase)

Pack your **current project** (not BrainPack) into a single AI-friendly file. Perfect for pasting into Copilot, Cursor, Claude, ChatGPT, etc.

```bash
# Pack to XML (best for Claude/Anthropic)
vibeanvil brain pack

# Pack to Markdown
vibeanvil brain pack --format md -o context.md

# Custom output
vibeanvil brain pack -o my_project.xml
```

Output:
```
➤ Context Pack
ℹ Packed 13,019 chars (~3,122 tokens)
✔ Packed to context.xml
```

The packer:
- ✅ Respects `.gitignore` and `.vibeanvilignore`
- ✅ Skips binary files (images, executables, etc.)
- ✅ Shows token estimation

---

## 💡 Best Practices

### 1. Start Focused
```bash
# Good: Specific topic
vibeanvil harvest -t "state-management" -l typescript --max-repos 20

# Bad: Too broad
vibeanvil harvest -q "code" --max-repos 100
```

### 2. Quality Over Quantity
```bash
# Prefer high-star repos
vibeanvil harvest -t rust --min-stars 500 --max-repos 10
```

### 3. Use Filters
```bash
# Ignore test files and docs
vibeanvil harvest -t rust \
  --ignore-glob "**/test*/**" \
  --ignore-glob "**/examples/**" \
  --allow-glob "*.rs"
```

### 4. Regular Updates
```bash
# Re-harvest with fresh repos
vibeanvil harvest -t rust --updated-within-days 30
```

---

## 🔍 Search Tips

### Exact Phrases
```bash
vibeanvil brain search "error handling"
```

### Multiple Terms
```bash
vibeanvil brain search "async await tokio"
```

### More Results
```bash
vibeanvil brain search "pattern" -n 50
```

---

## 🎭 AI Prompt Templates (Vibecode)

VibeAnvil includes the **Vibecode** prompt system—a set of high-performance, role-based templates that guide AI behavior during development. These prompts are designed for a **partnership model** between the human developer and the AI agent.

### 🛠️ Vibecode Prompts Provide:

- **Partnership Model**: Establishes a clear division of labor and collaboration style.
- **6-Step Workflow**: A structured approach from intake to shipping.
- **Debug Protocol**: Systematic troubleshooting and root cause analysis.
- **QA Framework**: Comprehensive testing strategies and edge case detection.
- **Security Audit**: Automated checks for common vulnerabilities and secret leaks.
- **Performance Review**: Optimization guidance for speed and resource usage.
- **Integration Protocol**: Ensuring new code fits perfectly into the existing architecture.
- **Handover Checklist**: Clear documentation and context for the next session.

### 📋 Available Templates

| Template | Role | Purpose |
|----------|------|---------|
| `architect` | Architect | System design, architecture analysis, and blueprinting |
| `developer` | Developer | Implementation guidance and code generation |
| `qa` | QA Engineer | Testing, bug finding, and validation |
| `plan` | Planner | Generating implementation plans and task lists |
| `review` | Reviewer | Code review feedback and quality assessment |
| `commit` | Writer | Atomic commit message generation |
| `install-vibeanvil` | Installer | Guided installation and setup for new users |

### ⌨️ Accessing Templates

You can quickly access and print these templates using the `vibeanvil prompt` command:

```bash
# Print the architect prompt
vibeanvil prompt architect

# Print the developer prompt
vibeanvil prompt developer
```

These templates are also automatically used by the `plan` and `build` commands when appropriate.

---

## 📋 Contract Templates

Pre-built **project contracts** with requirements and acceptance criteria:

| Template | Category | Description |
|----------|----------|-------------|
| `web-app` | Web | Standard web app with auth/CRUD |
| `cli-tool` | CLI | Command-line tool with subcommands |
| `api-service` | API | REST API with JWT auth |
| `library` | Lib | Reusable library with docs |

Location: `src/contract/templates.rs`

**Key Difference:**
- **Prompt Templates** = Tell AI *how to think* (roles)
- **Contract Templates** = Define *what to build* (requirements)

---

## 💖 Support VibeAnvil

Love BrainPack? Support the project!

<p align="center">
  <a href="https://github.com/sponsors/ThanhNguyxn">
    <img src="https://img.shields.io/badge/Sponsor-❤️-ea4aaa?style=for-the-badge&logo=github" alt="GitHub Sponsors">
  </a>
  <a href="https://buymeacoffee.com/thanhnguyxn">
    <img src="https://img.shields.io/badge/Buy%20Me%20A%20Coffee-☕-ffdd00?style=for-the-badge&logo=buy-me-a-coffee" alt="Buy Me A Coffee">
  </a>
</p>

---

Made with ❤️ by the VibeAnvil team
