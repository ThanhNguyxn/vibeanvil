# Start Here

Welcome to VibeAnvil! Get started in 3 steps:

## 1. Install

```bash
# macOS/Linux
curl -fsSL https://raw.githubusercontent.com/ThanhNguyxn/vibeanvil/main/install.sh | bash

# Windows (PowerShell admin)
irm https://raw.githubusercontent.com/ThanhNguyxn/vibeanvil/main/install.ps1 | iex

# Cargo
cargo install vibeanvil
```

## 2. Initialize

```bash
cd your-project
vibeanvil init
vibeanvil brain ensure  # Load curated templates
```

## 3. First Workflow

```bash
# Capture your requirements
vibeanvil intake -m "Build a CLI todo app with SQLite"

# Generate blueprint
vibeanvil blueprint --auto

# Lock your contract
vibeanvil contract create && vibeanvil contract lock

# Plan and build (choose your provider)
vibeanvil plan --provider human    # Manual coding
vibeanvil build iterate --provider claude-code  # AI-assisted
```

## Next Steps

| Want to... | Read |
|------------|------|
| Understand the full workflow | [docs/workflow.md](./workflow.md) |
| Search the BrainPack | [docs/brainpack.md](./brainpack.md) |
| Configure AI providers | [docs/PROVIDERS.md](./PROVIDERS.md) |
| Harvest more knowledge | [docs/DATA_SOURCES.md](./DATA_SOURCES.md) |
| See all commands | [docs/commands.md](./commands.md) |

## Quick Tips

- **Search first**: `vibeanvil brain search "pattern"` before coding
- **Lock contracts**: Don't skip `contract lock` - it prevents scope creep
- **Use presets**: `vibeanvil harvest --preset cli_framework_patterns`
- **Compact regularly**: `vibeanvil brain compact` keeps storage clean
