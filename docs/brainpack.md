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

### 1. Harvest Some Repos

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

Output format:
```json
{"type":"code","language":"rust","content":"fn main()...","source_id":"a1b2..."}
{"type":"doc","language":"markdown","content":"# README...","source_id":"a1b2..."}
```

### Markdown Format
```bash
vibeanvil brain export md -o brain.md
```

Creates a readable markdown document with all chunks organized by source.

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
