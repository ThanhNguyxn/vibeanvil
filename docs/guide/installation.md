# VibeAnvil Installation Guide

Follow these steps to install and configure VibeAnvil.

## Quick Install (Recommended)

### Linux / macOS

```bash
curl -fsSL https://raw.githubusercontent.com/ThanhNguyxn/vibeanvil/main/install.sh | bash
```

### Windows (PowerShell)

```powershell
irm https://raw.githubusercontent.com/ThanhNguyxn/vibeanvil/main/install.ps1 | iex
```

## Verify Installation

```bash
vibeanvil --version
```

## AI Install Prompt (Paste into LLM)

```
Install and configure VibeAnvil by following the instructions here:
https://raw.githubusercontent.com/ThanhNguyxn/vibeanvil/main/docs/guide/installation.md
```

## First-Time Setup

```bash
vibeanvil init
vibeanvil brain ensure
```

## Optional: Install AI Providers

- Claude Code CLI:
  ```bash
  npm install -g @anthropic-ai/claude-code
  ```

## Next Steps

- Read `docs/getting-started.md`
- Run `vibeanvil intake -m "Your project requirements"`
- Continue the workflow: `blueprint` → `contract` → `plan` → `build`
