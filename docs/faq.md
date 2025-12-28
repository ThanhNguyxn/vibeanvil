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

### Can I delete harvested data?

Yes, you can delete BrainPack data from your OS cache directory. See [DATA_LAYOUT.md](DATA_LAYOUT.md) for exact paths per platform.

Example (varies by OS):
```bash
# Check your data location first
vibeanvil brain stats
```

---

## 🔨 Build

### What providers are supported?

Currently:
- `human` - IDE assistants (Copilot, Cursor, VS Code Chat)
- `claude-code` - Claude Code CLI automation
- `command` - External CLI agents (Aider, etc.)
- `patch` - Unified diff workflows

See [PROVIDERS.md](PROVIDERS.md) for a full comparison.

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

Currently, VibeAnvil uses standard tooling for your project:
- Rust: `cargo test`, `cargo clippy`
- Node: `npm test`, `npm run lint`
- Python: `pytest`, `flake8`

Custom commands coming in a future version.

### What does `--evidence` capture?

- Terminal output
- Test results
- Lint reports
- File changes
- Timestamps

All stored in `.vibeanvil/sessions/<id>/evidence/`

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

Still have questions? [Open an issue](https://github.com/ThanhNguyxn/vibeanvil/issues/new) and we'll help!

---

Made with ❤️ by the VibeAnvil team
