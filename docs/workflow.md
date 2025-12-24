# 📋 Workflow Guide

Master the VibeAnvil workflow from idea to shipped product.

---

## 🔄 Workflow States

```
┌─────────┐    ┌───────────┐    ┌──────────┐    ┌───────────┐
│  INIT   │ →  │  INTAKE   │ →  │BLUEPRINT │ →  │ CONTRACT  │
└─────────┘    └───────────┘    └──────────┘    └───────────┘
                                                      │
                                                      ▼
┌─────────┐    ┌───────────┐    ┌──────────┐    ┌───────────┐
│ SHIPPED │ ←  │  REVIEW   │ ←  │  BUILD   │ ←  │   PLAN    │
└─────────┘    └───────────┘    └──────────┘    └───────────┘
```

---

## 📁 State: INIT

**Purpose:** Set up the VibeAnvil workspace

```bash
vibeanvil init
```

**What happens:**
- Creates `.vibeanvil/` directory
- Initializes state machine
- Sets up audit logging
- Prepares evidence collection

**Files created:**
```
.vibeanvil/
├── state.json       # Current state
├── audit.jsonl      # Audit log
├── evidence/        # Evidence directory
└── brainpack/       # BrainPack storage
```

---

## 📝 State: INTAKE

**Purpose:** Capture requirements and project goals

```bash
vibeanvil intake -m "Your project requirements here"
```

**Best practices:**

✅ **Good intake:**
```bash
vibeanvil intake -m "Build a REST API with:
- User authentication (JWT)
- CRUD for posts
- Rate limiting
- PostgreSQL database
- Docker deployment"
```

❌ **Bad intake:**
```bash
vibeanvil intake -m "Build an app"
```

**Tips:**
- Be specific about features
- Include technical constraints
- Mention preferred technologies
- Define success criteria

---

## 📐 State: BLUEPRINT

**Purpose:** Create a structured plan from intake

```bash
# Auto-generate from intake
vibeanvil blueprint --auto

# Or manually create
vibeanvil blueprint
```

**Blueprint structure:**
```markdown
# Project Blueprint

## Overview
[Generated from intake]

## Components
1. API Layer
2. Database Layer
3. Auth Layer

## Technical Stack
- Language: Rust
- Framework: Axum
- Database: PostgreSQL

## Milestones
[ ] MVP
[ ] Auth integration
[ ] Deployment
```

---

## 📜 State: CONTRACT

**Purpose:** Lock down the specifications

### Create Contract
```bash
vibeanvil contract create
```

### Validate Contract
```bash
vibeanvil contract validate
```

Checks:
- ✅ All required fields present
- ✅ Blueprint consistency
- ✅ Intake alignment
- ✅ No conflicts

### Lock Contract ⚠️
```bash
vibeanvil contract lock
```

> ⚠️ **WARNING:** Once locked, the contract **cannot be changed!**
> This ensures commit to the agreed specifications.

---

## 📋 State: PLAN

**Purpose:** Create implementation plan

```bash
```bash
vibeanvil plan --provider claude-code
# or
vibeanvil plan --provider human
```

**What happens:**
- AI analyzes contract
- Generates step-by-step implementation
- Creates task breakdown
- Estimates complexity

**Output:**
```
📋 Implementation Plan Generated

  1. [ ] Set up project structure
  2. [ ] Configure database schema
  3. [ ] Implement auth middleware
  4. [ ] Create API endpoints
  5. [ ] Add tests
  6. [ ] Configure Docker
```

---

## 🔨 State: BUILD

**Purpose:** Execute the implementation

### Manual Build
```bash
vibeanvil build manual start
# ... do your coding ...
vibeanvil build manual evidence  # capture evidence
vibeanvil build manual complete
```

### Auto Build
```bash
vibeanvil build auto --provider claude-code --evidence
```

Single-shot AI-assisted build.

### Iterate Build ⭐
```bash
vibeanvil build iterate --max 5 --evidence --provider human
```

Loop that:
1. 🔨 Builds code
2. 🧪 Runs tests
3. 🔍 Runs lints
4. 🔄 Fixes issues
5. Repeats until success or max iterations

**Example output:**
```
🔄 Iterate Build Mode
══════════════════════════════════════

[1/5] Building...
      Tests:  ❌ 3 failures
      Lint:   ⚠️  5 warnings
      → Fixing issues...

[2/5] Building...
      Tests:  ❌ 1 failure
      Lint:   ✓ Clean
      → Fixing issues...

[3/5] Building...
      Tests:  ✓ All passing
      Lint:   ✓ Clean
      
✓ Build complete after 3 iterations
```

---

## 🤖 Choosing a Provider

VibeAnvil supports multiple AI providers. See [PROVIDERS.md](PROVIDERS.md) for full details.

| Provider | Best For |
|----------|----------|
| `human` | GitHub Copilot, Cursor, VS Code Chat |
| `claude-code` | Fully automated CLI workflows |
| `command` | External CLI agents (Aider, etc.) |
| `patch` | API-based LLMs returning diffs |

---

## 👀 State: REVIEW

**Purpose:** Review and approve the build

### Start Review
```bash
vibeanvil review start
```

### Check Changes
Review the generated code, tests, and evidence.

### Pass or Fail
```bash
# If satisfied
vibeanvil review pass

# If needs changes
vibeanvil review fail
```

On fail, you return to BUILD state to fix issues.

---

## 🚀 State: SHIPPED

**Purpose:** Mark project as complete

```bash
vibeanvil ship --tag v1.0.0 -m "Initial release"
```

**What happens:**
- Creates final snapshot
- Generates ship manifest
- Archives evidence
- Writes audit entry

**Output:**
```
🚀 Project Shipped!
══════════════════════════════════════

  Version:    v1.0.0
  Message:    Initial release
  Hash:       abc123...
  Timestamp:  2024-12-24T18:00:00Z
  
  Evidence:   12 files captured
  Audit:      156 entries logged
  
Congratulations! 🎉
```

---

## 🔀 State Transitions

| From | To | Command |
|------|----|---------|
| (none) | init | `vibeanvil init` |
| init | intake | `vibeanvil intake` |
| intake | blueprint | `vibeanvil blueprint` |
| blueprint | contract | `vibeanvil contract create` |
| contract | plan | `vibeanvil contract lock` → `vibeanvil plan` |
| plan | build | `vibeanvil build` |
| build | review | `vibeanvil review start` |
| review | shipped | `vibeanvil review pass` → `vibeanvil ship` |
| review | build | `vibeanvil review fail` |

---

## 💖 Support VibeAnvil

<p align="center">
  <a href="https://github.com/sponsors/ThanhNguyxn">
    <img src="https://img.shields.io/badge/Sponsor-❤️-ea4aaa?style=for-the-badge&logo=github" alt="Sponsor on GitHub">
  </a>
  <a href="https://buymeacoffee.com/thanhnguyxn">
    <img src="https://img.shields.io/badge/Buy%20Me%20A%20Coffee-☕-ffdd00?style=for-the-badge&logo=buy-me-a-coffee" alt="Buy Me A Coffee">
  </a>
</p>

---

Made with ❤️ by the VibeAnvil team
