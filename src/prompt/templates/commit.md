# Commit Message Prompt

Generate a Conventional Commit message for the following changes.

## Changes
{{diff}}

## Rules
1. Use format: `type(scope): description`
2. Types: feat, fix, docs, style, refactor, test, chore
3. Scope is optional but helpful (e.g., `cli`, `api`, `auth`)
4. Description should be concise (50 chars max)
5. Use imperative mood ("add", not "added")

## Examples
- feat(auth): add JWT token validation
- fix(cli): resolve path escaping on Windows
- docs: update installation guide

## Output
Provide ONLY the commit message, nothing else.
