# Rules and Memory Architecture

This document defines a practical, implementation-ready rule and memory model for VibeAnvil.

## Objectives
- Keep prompts senior-level and consistent across sessions.
- Reduce hallucinations through layered constraints and evidence requirements.
- Preserve project-specific conventions without hardcoding them in every template.

## Rule Layers

1. Global defaults
- Stable quality bars shared across projects (safety, evidence, validation).

2. Project rules
- Repository-specific conventions (naming, patterns, module boundaries).

3. Task constraints
- Request-specific requirements from contract/plan/debug contexts.

4. Runtime context
- Current files, diffs, diagnostics, test output, and command results.

## Precedence

Highest to lowest:
1. Explicit CLI/user instruction
2. Task constraints
3. Project rules
4. Global defaults

Conflict policy:
- Stricter safety and verification rules always win.
- Narrowing scope is allowed; relaxing security constraints is not.

## Activation Modes

- Always: core safety and quality constraints.
- Contextual: loaded when file globs or command intent match.
- Manual: opt-in rules for special workflows.

## Memory Model

Use persistent project memory for high-value context only:
- architecture decisions
- accepted conventions
- known pitfalls and anti-patterns
- reliable rollout/rollback patterns

Do not store ephemeral noise (one-off logs, transient command output).

## Integration Mapping

Templates should contain:
- role + mission
- required context variables
- protocol
- self-check
- output format

Brainpack should contain:
- reusable engineering patterns
- checklists and operating guides
- domain-specific runbooks

Docs should contain:
- layer definitions
- precedence rules
- activation behavior
- governance and update policy

CLI should expose:
- template printing (`vibeanvil prompt <kind>`)
- status visibility for loaded rules/memory (future)

## Governance

- Keep rules short and testable.
- Version notable changes with rationale.
- Require evidence-backed updates for safety-critical guidance.
- Prefer additive evolution over breaking rewrites.
