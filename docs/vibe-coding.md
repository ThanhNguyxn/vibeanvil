# Vibe Coding Features 🌊

VibeAnvil now supports "Vibe Coding" workflows to help you stay in the flow.

## 🧠 Smart Context (Repository Map)

The `plan` command is now aware of your entire codebase structure. It scans your project to understand types, functions, and dependencies before generating an implementation plan.

**Usage:**
```bash
vibeanvil plan
```

**What it does:**
- Scans all source files (Rust, JS, TS, etc.).
- Extracts high-level signatures (structs, classes, functions).
- Feeds this "map" to the AI to prevent hallucinations.

## ✍️ Interactive Auto-Commit

Forget about writing "update" commit messages. VibeAnvil can dream them up for you.

**Usage:**
1. Start a manual build:
   ```bash
   vibeanvil build manual start
   ```
2. Make your code changes.
3. Complete the build:
   ```bash
   vibeanvil build manual complete
   ```

**The Flow:**
- VibeAnvil analyzes your `git diff`.
- It generates a **Conventional Commit** message (e.g., `feat: add user login`).
- It asks you to **Confirm**, **Edit**, or **Cancel**.
- You stay in control, but with less friction.

## 🎨 Premium CLI Experience

We've upgraded the CLI with:
- **Spinners**: To let you know when the AI is thinking.
- **Colors**: To highlight success, warnings, and errors.
- **Clean Output**: No more raw text dumps.
