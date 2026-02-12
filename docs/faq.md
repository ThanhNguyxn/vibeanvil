# ❓ Frequently Asked Questions

Quick answers to common questions about VibeAnvil.

---

## 📦 Installation

### How do I install VibeAnvil?

**Windows:**
```powershell
irm https://raw.githubusercontent.com/ThanhNguyxn/vibeanvil/main/install.ps1 | iex
```

**Linux/macOS:**
```bash
curl -fsSL https://raw.githubusercontent.com/ThanhNguyxn/vibeanvil/main/install.sh | bash
```

### How do I update to the latest version?

```bash
vibeanvil upgrade
```

Or check for updates first:
```bash
vibeanvil update
```

### How do I uninstall?

**Windows:**
```powershell
irm https://raw.githubusercontent.com/ThanhNguyxn/vibeanvil/main/uninstall.ps1 | iex
```

**Linux/macOS:**
```bash
curl -fsSL https://raw.githubusercontent.com/ThanhNguyxn/vibeanvil/main/uninstall.sh | bash
```

### What's the difference between `update` and `upgrade`?

- `update` - **Checks** if a new version is available
- `upgrade` - **Downloads and installs** the latest version

---

## 🔄 Workflow

### Can I skip states in the workflow?

No. VibeAnvil enforces a strict state machine:
```
init → intake → blueprint → contract → plan → build → review → shipped
```

This ensures proper documentation and audit trails.

### Can I go back to a previous state?

Only in specific cases:
- `review fail` → Returns to `build` state
- Otherwise, you need to re-initialize

### What happens if I run `init --force`?

⚠️ **Warning:** This resets your entire workspace, including:
- State history
- Evidence collected
- Audit logs

Only use when starting completely fresh.

### Can I change the contract after locking?

**No.** Once locked, the contract is permanent. This is by design to ensure commitment to specifications.

If you need changes, you'll need to:
1. Complete the current workflow
2. Start a new workflow with updated requirements

---

## 🧠 BrainPack

### What is BrainPack?

BrainPack is VibeAnvil's knowledge harvesting system. It:
- Downloads code from GitHub repos
- Indexes content for fast search
- Provides context for AI builds

### Is my harvested data private?

**Yes, absolutely!** BrainPack is privacy-first:
- Source URLs are never stored
- Repo names are SHA-256 hashed
- All data stays local
- No external requests during search

### How much storage does BrainPack use?

Typically:
- 10 repos ≈ 10-50 MB
- 100 repos ≈ 100-500 MB

You can check with:
```bash
vibeanvil brain stats
```

### How do I install the Core BrainPack?

The Core BrainPack ships embedded in VibeAnvil. Install it with:
```bash
vibeanvil brain ensure
```

After upgrading VibeAnvil, refresh the Core BrainPack:
```bash
vibeanvil brain ensure --refresh-core
```

### How do I optimize my BrainPack database?

Over time, your database may contain duplicate chunks. Compact it with:
```bash
vibeanvil brain compact
```

This deduplicates JSONL data and optimizes the SQLite index.

### How do I export my codebase as context for AI?

Use `brain pack` to export your project as a single AI-friendly file:
```bash
# Export as XML (default)
vibeanvil brain pack

# Export as Markdown
vibeanvil brain pack --format md

# Include specific patterns only
vibeanvil brain pack --include "src/**/*.rs"
```

### Can I delete harvested data?

Yes, you can delete BrainPack data from your OS cache directory. See [docs/data-layout.md](data-layout.md) for exact paths per platform.

Example (varies by OS):
```bash
# Check your data location first
vibeanvil brain stats
```

---

## 🔨 Build

### What providers are supported?

VibeAnvil supports 20+ providers across several categories:

- **IDE AI (CLI):** `cursor`, `cline`, `continue`, `cody`
- **IDE AI (prompt-based):** `copilot`, `zed`, `windsurf`, `trae`, `jetbrains`, `supermaven`, `gemini-assist`
- **Terminal agents:** `claude-code`, `aider`, `opencode`, `goose`
- **Cloud AI:** `kiro` (AWS)
- **Self-hosted:** `tabby`, `ollama` (with 15+ model shortcuts)
- **Generic:** `human`, `command`, `patch`, `mock`

See [providers.md](providers.md) for full comparison and setup instructions.

### How does iterate mode work?

Iterate mode creates a feedback loop:
```
build → test → lint → fix → repeat
```

It continues until:
- ✅ All tests pass AND linting is clean
- ❌ Max iterations reached (default: 5)
- ❌ Error in strict mode

### Can I use my own test/lint commands?

**Yes!** Use the `--cmd` flag to run any test or lint command:

```bash
# Custom test command
vibeanvil test --cmd "pytest -v"

# Custom lint command
vibeanvil lint --cmd "eslint ."

# With auto-fix
vibeanvil test --cmd "cargo test" --fix
vibeanvil lint --cmd "cargo clippy" --fix
```

Without `--cmd`, VibeAnvil auto-detects standard tooling for your project:
- Rust: `cargo test`, `cargo clippy`
- Node: `npm test`, `npm run lint`
- Python: `pytest`, `flake8`

### What does `--evidence` capture?

- Terminal output
- Test results
- Lint reports
- File changes
- Timestamps

All stored in `.vibeanvil/sessions/<id>/evidence/`

---

## 🤖 AI & Prompt Templates

### What prompt templates are available?

VibeAnvil includes 13 built-in prompt templates for different roles:

```bash
# List all templates
vibeanvil prompt --list

# Print a specific template
vibeanvil prompt developer
vibeanvil prompt security
vibeanvil prompt architect
```

Available kinds: `install`, `architect`, `developer`, `qa`, `plan`, `review`, `commit`, `debug`, `xray`, `vision`, `security`, `migrate`, `refactor`.

### Can I render templates with variables?

**Yes!** Use `--render` with `--var`:
```bash
vibeanvil prompt vision --render --var description="Build a SaaS dashboard" --var tech_stack="nextjs"
```

### What are chat modes?

VibeAnvil supports Aider-style chat modes for quick AI interactions:

```bash
vibeanvil chat ask "How does the state machine work?"
vibeanvil chat code "Add a new field to the user model"
vibeanvil chat architect "Should we use Redis for caching?"
```

Modes: `ask` (questions), `code` (implementation), `architect` (design decisions), `help` (usage help).

---

## 🔧 Utilities

### How do I check system health?

Run the built-in doctor to verify your environment:
```bash
vibeanvil doctor
```

This checks for required tools (git, etc.), permissions, and configuration.

### Is there an interactive mode?

**Yes!** The wizard provides a guided menu for common workflows:
```bash
vibeanvil wizard
```

---

## 🐛 Troubleshooting

### "Error: No contract to lock"

You need to create a contract first:
```bash
vibeanvil contract create
vibeanvil contract lock
```

### "Error: Contract not locked"

Lock your contract before proceeding:
```bash
vibeanvil contract lock
```

### "Error: Plan not created"

Create a plan first:
```bash
vibeanvil plan
```

### Made a mistake? Undo it!

If the last AI change was wrong, undo it:
```bash
vibeanvil undo --dry-run  # Preview first
vibeanvil undo            # Revert
```

### "Could not fetch checksums, skipping verification"

This is **normal** if:
- It's your first install
- The release was just created

The checksum file takes a moment to propagate.

### Install fails with "Failed to download binary"

Check that:
1. The release exists on GitHub
2. Your internet connection is working
3. The correct platform binary is available

---

## 💰 Pricing

### Is VibeAnvil free?

**Yes!** VibeAnvil is open source and free to use.

### How can I support the project?

We'd love your support! 💖

[![GitHub Sponsors](https://img.shields.io/badge/Sponsor-❤️-ea4aaa?style=for-the-badge&logo=github)](https://github.com/sponsors/ThanhNguyxn)
[![Buy Me A Coffee](https://img.shields.io/badge/Buy%20Me%20A%20Coffee-☕-ffdd00?style=for-the-badge&logo=buy-me-a-coffee)](https://buymeacoffee.com/thanhnguyxn)

Other ways to help:
- ⭐ Star the repo
- 🐛 Report bugs
- 📝 Contribute docs
- 💻 Submit PRs

---

## 📞 Getting Help

### Where can I report bugs?

[GitHub Issues](https://github.com/ThanhNguyxn/vibeanvil/issues)

### Where can I request features?

[GitHub Discussions](https://github.com/ThanhNguyxn/vibeanvil/discussions)

### Where can I find the source code?

[GitHub Repository](https://github.com/ThanhNguyxn/vibeanvil)

---

## 🔒 Security

### Is VibeAnvil secure?

Yes! VibeAnvil includes built-in security hardening:

- **Path Traversal Protection** - Blocks `../` attacks
- **Filename Sanitization** - Validates all file operations
- **Privacy-First** - Source IDs are anonymized (SHA-256)
- **Secret Redaction** - Sensitive data auto-redacted from logs

### Does VibeAnvil send my code anywhere?

No. All operations are local unless you explicitly use the `harvest` command to download GitHub repos. Your code never leaves your machine.

### Can I use VibeAnvil on private projects?

Absolutely! VibeAnvil is designed for private, secure workflows. No telemetry, no cloud dependencies.

---

Made with ❤️ by the VibeAnvil team
