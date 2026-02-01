# 📋 Templates Guide

VibeAnvil has **two types** of templates with different purposes:

---

## 🎭 AI Prompt Templates

**Purpose:** Guide AI behavior during tasks.

| Template | Role | What It Does |
|----------|------|--------------|
| `plan` | Planner | Break down work into tasks |
| `review` | Reviewer | Provide code feedback |
| `commit` | Writer | Generate commit messages |
| `architect` | Architect | System design analysis |
| `developer` | Developer | Implementation guidance |
| `qa` | QA | Testing and bug finding |
| `install-vibeanvil` | Installer | LLM paste-in prompt for VibeAnvil |

**Location:** `src/prompt/templates/`

**Usage:** Loaded internally by VibeAnvil commands.
Use `vibeanvil prompt install` to print the installer prompt.

**Custom Templates:** Add `.md` files to `.vibeanvil/prompts/`

---

## 📦 Contract Templates

**Purpose:** Pre-built project contracts with requirements.

| Template | Category | For Building |
|----------|----------|--------------|
| `web-app` | Web | Apps with auth/CRUD |
| `cli-tool` | CLI | Command-line tools |
| `api-service` | API | REST APIs with JWT |
| `library` | Lib | Reusable libraries |

**Location:** `src/contract/templates.rs`

**Variables:** Use `{{name}}` to substitute values.

---

## Key Difference

| Prompt Templates | Contract Templates |
|------------------|-------------------|
| Tell AI *how to think* | Define *what to build* |
| AI role instructions | Project requirements |
| Used during commands | Used with `contract create` |

---

Made with ❤️ by the VibeAnvil team
