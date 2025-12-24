# 🔍 Data Sources Guide

Strategies for discovering and harvesting high-quality content for your BrainPack.

---

## 📋 Overview

VibeAnvil's `harvest` command searches GitHub to build your knowledge base. This guide provides strategies for effective discovery **without pinning specific repositories**.

> 💡 **Privacy Note:** VibeAnvil stores content with anonymized source IDs. No repository URLs are persisted.

---

## 🏷️ GitHub Discovery Methods

### Topics to Explore

Use `-t, --topic` flags to search by GitHub topic:

| Category | Topics to Try |
|----------|---------------|
| **CLI Tools** | `cli`, `command-line`, `terminal`, `shell` |
| **Developer Tools** | `developer-tools`, `devtools`, `tooling` |
| **Scaffolding** | `scaffolding`, `boilerplate`, `starter`, `template` |
| **Code Generation** | `codegen`, `generator`, `automation` |
| **AI/LLM** | `llm`, `gpt`, `agent`, `prompt-engineering` |
| **Guardrails** | `guardrails`, `validation`, `safety` |
| **Workflow** | `workflow`, `pipeline`, `automation` |
| **State Machines** | `state-machine`, `fsm`, `statechart` |

### Keywords to Search

Use `-q, --query` flags for keyword searches:

| Category | Keywords |
|----------|----------|
| **Contract-First** | `contract-first`, `schema-driven`, `spec-driven` |
| **Evidence/Audit** | `evidence`, `audit log`, `audit trail`, `logging` |
| **Iterative** | `iterative`, `test fix loop`, `retry`, `autofix` |
| **Installers** | `installer`, `release`, `distribution`, `publish` |
| **Checksums** | `checksums`, `verification`, `sha256`, `integrity` |
| **CLI Patterns** | `cli framework`, `argument parsing`, `subcommand` |

---

## 🔎 Advanced Search Examples

### By Language + Topic + Stars

```bash
# High-quality Rust CLI tools
vibeanvil harvest -l rust -t cli --min-stars 100 --max-repos 15

# TypeScript state machines
vibeanvil harvest -l typescript -q "state machine" --min-stars 50

# Go installers and release tooling
vibeanvil harvest -l go -q "installer release" --min-stars 30 --max-repos 10
```

### By Recency

```bash
# Active projects (updated in last 90 days)
vibeanvil harvest -q "cli framework" --updated-within-days 90

# Fresh content only
vibeanvil harvest -t workflow --updated-within-days 30 --min-stars 20
```

### Combined Filters

```bash
# Quality + recency + language
vibeanvil harvest \
  -l rust \
  -t cli \
  --min-stars 200 \
  --updated-within-days 180 \
  --max-repos 20

# Multiple queries
vibeanvil harvest \
  -q "audit log" \
  -q "evidence capture" \
  --min-stars 50 \
  --max-repos 15
```

---

## 📦 Harvest Presets

Ready-to-use search configurations for common use cases:

### Preset: Workflow CLI

Tools that implement CLI workflows, state machines, and pipelines.

```bash
vibeanvil harvest \
  -t cli \
  -t workflow \
  -q "state machine" \
  --min-stars 50 \
  --updated-within-days 365 \
  --max-repos 20 \
  --ignore-glob "**/test*/**" \
  --ignore-glob "**/docs/**"
```

### Preset: Release Installers

Release automation, install scripts, and distribution tooling.

```bash
vibeanvil harvest \
  -q "install script" \
  -q "release automation" \
  -t installer \
  --min-stars 30 \
  --updated-within-days 365 \
  --max-repos 15
```

### Preset: Evidence & Audit

Logging, audit trails, and evidence collection patterns.

```bash
vibeanvil harvest \
  -q "audit log" \
  -q "evidence capture" \
  -q "structured logging" \
  --min-stars 50 \
  --updated-within-days 365 \
  --max-repos 20
```

### Preset: Iterate Loops

Test-fix loops, autofix, and iterative development patterns.

```bash
vibeanvil harvest \
  -q "test fix loop" \
  -q "autofix" \
  -q "lint fix" \
  --min-stars 30 \
  --updated-within-days 180 \
  --max-repos 15
```

### Preset: Prompt/Spec Packs

Prompt engineering, spec-driven development, and LLM tooling.

```bash
vibeanvil harvest \
  -t llm \
  -t prompt-engineering \
  -q "prompt template" \
  -q "spec driven" \
  --min-stars 50 \
  --updated-within-days 180 \
  --max-repos 20
```

---

## 🌐 Non-GitHub Sources

### Package Registry Ideas

Search these registries for patterns and best practices:

| Registry | Search Keywords |
|----------|-----------------|
| **crates.io** | `cli`, `workflow`, `state-machine`, `release` |
| **npm** | `cli-framework`, `state-machine`, `release-it` |
| **PyPI** | `cli`, `workflow`, `audit-log` |

### Official Documentation

Learn best practices from authoritative sources:

| Topic | Resource |
|-------|----------|
| **GitHub Search Syntax** | GitHub Docs → Searching on GitHub |
| **GitHub Releases** | GitHub Docs → About releases |
| **SQLite FTS5** | SQLite Docs → FTS5 Full-text Search |
| **Checksum Verification** | Your package manager's docs |

### Security References

High-level security patterns (no specific implementations):

- Secret scanning patterns (API keys, tokens, credentials)
- Secure installer practices (HTTPS, checksum verification)
- Safe subprocess execution patterns
- Input validation and sanitization

---

## ⚙️ Environment Variables

### GITHUB_TOKEN (Optional)

Set to increase API rate limits:

```bash
# Linux/macOS
export GITHUB_TOKEN=ghp_your_token_here

# Windows PowerShell
$env:GITHUB_TOKEN = "ghp_your_token_here"
```

| Without Token | With Token |
|---------------|------------|
| 10 requests/min | 30 requests/min |
| Limited searches | More repos per session |

### Getting a Token

1. Go to GitHub → Settings → Developer settings → Personal access tokens
2. Generate a token with `public_repo` scope (read-only is fine)
3. Set as environment variable before running harvest

---

## 📊 Balancing Quality vs. Quantity

### Recommendations

| Factor | Guidance |
|--------|----------|
| **Stars** | Use `--min-stars 50+` for quality, `10+` for diversity |
| **Recency** | `--updated-within-days 180` balances freshness and volume |
| **Max Repos** | Start with `--max-repos 10-20`, increase as needed |
| **Language** | Specify with `-l` to focus on relevant patterns |

### Quality Signals

- High star count → community validation
- Recent updates → active maintenance
- Good topic tags → well-organized project
- Multiple contributors → diverse perspectives

---

## 🎯 Targeting Strategies

### Strategy 1: Depth First

Focus on one domain deeply:

```bash
# Rust CLI ecosystem
vibeanvil harvest -l rust -t cli --min-stars 100 --max-repos 30
vibeanvil harvest -l rust -t cli -q "argument parsing" --max-repos 20
vibeanvil harvest -l rust -t cli -q "error handling" --max-repos 20
```

### Strategy 2: Breadth First

Cover multiple domains:

```bash
vibeanvil harvest -q "cli workflow" --max-repos 10
vibeanvil harvest -q "state machine" --max-repos 10
vibeanvil harvest -q "audit log" --max-repos 10
vibeanvil harvest -q "release installer" --max-repos 10
```

### Strategy 3: Problem-Specific

Target specific challenges:

```bash
# Authentication patterns
vibeanvil harvest -q "jwt authentication" -l typescript --max-repos 15

# Error handling
vibeanvil harvest -q "error handling pattern" --min-stars 100 --max-repos 10
```

---

## 🔒 Privacy Reminders

When harvesting:

- ✅ Content is stored locally
- ✅ Source IDs are SHA-256 hashed (anonymized)
- ✅ No repository URLs are persisted
- ✅ Search works offline after harvest
- ❌ No telemetry or tracking

---

## 💡 Tips

1. **Start small**: Begin with 10-20 repos, check quality, then expand
2. **Use filters**: `--ignore-glob` to skip tests/docs for code-focused packs
3. **Iterate**: Run multiple targeted harvests rather than one broad one
4. **Check stats**: `vibeanvil brain stats` to monitor your BrainPack size
5. **Export regularly**: `vibeanvil brain export jsonl` for backup

---

Made with ❤️ by the VibeAnvil team
