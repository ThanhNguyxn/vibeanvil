# 🚀 Getting Started with VibeAnvil

Welcome to VibeAnvil! This guide will get you up and running in under 5 minutes.

## 📦 Installation

### Windows (PowerShell)

```powershell
# Install
irm https://raw.githubusercontent.com/ThanhNguyxn/vibeanvil/main/install.ps1 | iex

# Verify
vibeanvil --version
```

### Linux/macOS

```bash
# Install
curl -fsSL https://raw.githubusercontent.com/ThanhNguyxn/vibeanvil/main/install.sh | bash

# Verify
vibeanvil --version
```

### From Source (Cargo)

```bash
cargo install --git https://github.com/ThanhNguyxn/vibeanvil
```

---

## 🎯 Your First Workflow

### Step 1: Initialize Workspace 📁

```bash
cd my-project
vibeanvil init
```

Output:
```
╔═══════════════════════════════════════════════════════════════╗
║   ██╗   ██╗██╗██████╗ ███████╗ █████╗ ███╗   ██╗██╗   ██╗██╗  ║
║   ...                                                          ║
╚═══════════════════════════════════════════════════════════════╝

✓ Created .vibeanvil/
✓ Initialized state: intake
✓ Created audit log
```

### Step 2: Capture Requirements 📝

```bash
vibeanvil intake -m "Build a todo app with React and Node.js"
```

Or interactively:
```bash
vibeanvil intake
# Then type your requirements when prompted
```

### Step 3: Check Status 📊

```bash
vibeanvil status
```

Output:
```
┌─────────────────────────────────────────┐
│  📊 VibeAnvil Status                    │
├─────────────────────────────────────────┤
│  State:     intake ✓                    │
│  Intake:    todo-app-react-nodejs       │
│  Blueprint: (pending)                   │
│  Contract:  (pending)                   │
└─────────────────────────────────────────┘
```

### Step 4: Generate Blueprint 📐

```bash
vibeanvil blueprint --auto
```

This creates a structured blueprint from your intake.

### Step 5: Create Contract 📜

```bash
vibeanvil contract create
vibeanvil contract validate
vibeanvil contract lock
```

> ⚠️ Once locked, the contract cannot be changed!

### Step 6: Build 🔨

```bash
# Manual mode (step by step)
vibeanvil build manual start

# Human mode (Copilot/Cursor) - Recommended for first run
vibeanvil build iterate --provider human

# Auto mode (Claude Code)
vibeanvil build auto --provider claude-code

# Iterate mode (test → fix loop)
vibeanvil build iterate --max 5 --evidence
```

### Step 7: Review & Ship 🚀

```bash
vibeanvil review start
vibeanvil review pass  # or: vibeanvil review fail
vibeanvil ship --tag v1.0.0 -m "First release!"
```

---

## 🔄 Updating VibeAnvil

```bash
# Check for updates
vibeanvil update

# Download and install latest version
vibeanvil upgrade
```

---

## 🗑️ Uninstalling

### Windows
```powershell
irm https://raw.githubusercontent.com/ThanhNguyxn/vibeanvil/main/uninstall.ps1 | iex
```

### Linux/macOS
```bash
curl -fsSL https://raw.githubusercontent.com/ThanhNguyxn/vibeanvil/main/uninstall.sh | bash
```

---

## 🆕 New Features

### Undo Changes ↩️
Made a mistake? Undo the last AI change:
```bash
vibeanvil undo --dry-run  # Preview first
vibeanvil undo            # Revert
```

### Faceted Search 🔍
Filter brain search by type or language:
```bash
vibeanvil brain search "error" -l rust
vibeanvil brain search "parse" -t code
```

### Contract Templates 📋
Use pre-built templates to start faster:
- `web-app` - Web apps with auth/CRUD
- `cli-tool` - Command-line tools
- `api-service` - REST APIs with JWT
- `library` - Reusable libraries

---

## 📚 Next Steps

- [📋 Workflow Guide](workflow.md) - Deep dive into each state
- [🧠 BrainPack Guide](brainpack.md) - Harvest knowledge from GitHub
- [🔧 Commands Reference](commands.md) - All commands explained

---

## 💬 Need Help?

- 📖 [Full Documentation](https://github.com/ThanhNguyxn/vibeanvil/docs)
- 🐛 [Report a Bug](https://github.com/ThanhNguyxn/vibeanvil/issues)
- ⭐ [Star on GitHub](https://github.com/ThanhNguyxn/vibeanvil)

---

Happy vibe coding! 🎉
