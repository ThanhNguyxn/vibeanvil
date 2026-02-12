# 📋 Templates Guide

VibeAnvil has **two types** of templates with different purposes:

---

## 🎭 AI Prompt Templates (Vibecode)

**Purpose:** Guide AI behavior during tasks using the Vibecode partnership model.

| Template | Role | Key Features |
|----------|------|--------------|
| `architect` | Architect | System design analysis, blueprinting, and architectural decision records. |
| `developer` | Developer | Implementation guidance, code generation, and adherence to best practices. |
| `qa` | QA Engineer | Testing strategies, edge case detection, and bug finding. |
| `plan` | Planner | Breaking down requirements into actionable implementation plans and task lists. |
| `review` | Reviewer | Code review feedback, quality assessment, and security checks. |
| `commit` | Writer | Generating atomic, descriptive commit messages based on changes. |
| `install-vibeanvil` | Installer | Guided installation and setup prompt for new users to paste into an LLM. |

**Location:** `src/prompt/templates/`

**Usage:** Loaded internally by VibeAnvil commands like `plan` and `build`.
You can also print them manually using `vibeanvil prompt <KIND>`.

**Custom Templates:** Add `.md` files to `.vibeanvil/prompts/` to override or extend the built-in templates.

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
