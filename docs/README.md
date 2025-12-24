# 📚 VibeAnvil Documentation

Welcome to the VibeAnvil documentation! This folder contains detailed guides for every feature.

## 📖 Quick Links

| Guide | Description |
|-------|-------------|
| [🚀 Getting Started](getting-started.md) | Installation and first steps |
| [📖 Usage Guide](USAGE.md) | Comprehensive command usage guide |
| [📋 Workflow Guide](workflow.md) | Complete workflow from intake to ship |
| [🧠 BrainPack Guide](brainpack.md) | Core BrainPack + harvesting knowledge |
| [🔧 Commands Reference](commands.md) | All CLI commands explained |
| [📁 Data Layout](DATA_LAYOUT.md) | Where data is stored (workspace + cache) |
| [🔍 Data Sources](DATA_SOURCES.md) | Discovery strategies for harvest |
| [❓ FAQ](faq.md) | Frequently asked questions |

## 🎯 What is VibeAnvil?

VibeAnvil is a **contract-first vibe coding CLI** that enforces structured development workflows with:

- 🔒 **State Machine Workflow** - Guided transitions from idea to shipped product
- 📝 **Evidence Collection** - Automatic documentation of your work
- 🧠 **BrainPack Harvesting** - Learn from GitHub repos dynamically
- 🔍 **Privacy-First** - No external URLs, anonymized sources
- 🤖 **AI Provider Plugins** - Claude Code integration for AI-assisted coding

## 🌟 Why VibeAnvil?

```
Traditional Coding           vs           VibeAnvil
────────────────────                    ────────────────
❌ Jump right into code                 ✅ Intake → Blueprint first
❌ No documentation                     ✅ Auto-generated evidence
❌ "It works on my machine"             ✅ Contract-locked specs
❌ Unaudited changes                    ✅ Full audit trail
❌ No learning from others              ✅ BrainPack knowledge harvest
```

## 💡 Quick Example

```bash
# Initialize workspace
vibeanvil init

# Capture requirements  
vibeanvil intake -m "Build a REST API with user auth"

# Generate blueprint
vibeanvil blueprint --auto

# Create and lock contract
vibeanvil contract create
vibeanvil contract lock

# Build with AI assistance
vibeanvil build iterate --max 5 --evidence

# Verify and ship
vibeanvil review start
vibeanvil ship --tag v1.0.0
```

---

Made with ❤️ by the VibeAnvil team
