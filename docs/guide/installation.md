# VibeAnvil Installation Guide

Follow these steps to install and configure VibeAnvil.

## Prerequisites

Before installing VibeAnvil, ensure you have the following:

- **Git**: Required for evidence capture and version control integration.
- **Rust 1.75+**: Required if you are building from source.
- **GITHUB_TOKEN** (optional): Recommended for higher API rate limits when harvesting repositories.

## Quick Install (Recommended)

### Linux / macOS

```bash
curl -fsSL https://raw.githubusercontent.com/ThanhNguyxn/vibeanvil/main/install.sh | bash
```

### Windows (PowerShell)

```powershell
irm https://raw.githubusercontent.com/ThanhNguyxn/vibeanvil/main/install.ps1 | iex
```

## Build from Source

If you prefer to build VibeAnvil from source:

1. **Clone the repository**:
   ```bash
   git clone https://github.com/ThanhNguyxn/vibeanvil.git
   cd vibeanvil
   ```

2. **Build the release binary**:
   ```bash
   cargo build --release
   ```

3. **Install the binary**:
   Move the binary to a directory in your `PATH`:
   ```bash
   # Linux/macOS
   mv target/release/vibeanvil /usr/local/bin/
   
   # Windows
   # Move target\release\vibeanvil.exe to a folder in your PATH
   ```

## Verify Installation

After installation, verify that VibeAnvil is working correctly:

```bash
# Check version
vibeanvil --version

# Run system health check
vibeanvil doctor
```

## First-Time Setup

Initialize your first workspace and install the Core BrainPack:

```bash
# Initialize workspace
vibeanvil init

# Install Core BrainPack (curated templates)
vibeanvil brain ensure
```

## Troubleshooting

### "Command not found"
Ensure the installation directory is in your system's `PATH`.
- **Linux/macOS**: Usually `~/.local/bin` or `/usr/local/bin`.
- **Windows**: Usually `%USERPROFILE%\.vibeanvil\bin`.

### Permission Denied
On Linux/macOS, you might need to use `sudo` for the install script if installing to a protected directory, or ensure your user has write access to the target directory.

### GitHub Rate Limiting
If you encounter rate limits when running `harvest`, set your `GITHUB_TOKEN`:
```bash
export GITHUB_TOKEN=your_token_here
```

## AI Install Prompt (Paste into LLM)

You can ask an AI assistant to help you install VibeAnvil:

```bash
vibeanvil prompt install
```

## Next Steps

- Read [Getting Started](../getting-started.md)
- Explore [Commands Reference](../commands.md)
- Learn about [Workflow](../workflow.md)

