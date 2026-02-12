# Prompt System Roadmap

This roadmap captures high-value upgrades for VibeAnvil's prompt and brainpack system based on external research and internal template audits.

## Current Strengths
- Role-specific templates with clear mission and output formats.
- Self-check sections added across active templates.
- Security/reliability brainpack coverage expanded.
- CLI prompt surfacing for all built-in workflow templates.

## High-ROI Upgrades

### 1) Rule Layering and Precedence
- Add explicit instruction hierarchy: global defaults -> workspace rules -> task-specific overrides.
- Define conflict resolution policy: stricter safety rule wins; task-level may narrow scope but not relax security constraints.
- Document precedence in one canonical guide and mirror it in templates.

### 2) Context Packaging Contracts
- Standardize runtime variables across templates: `{{context}}`, `{{tech_stack}}`, `{{constraints}}`, `{{history}}` where relevant.
- Add a minimum context contract per template so missing inputs are flagged early.
- Include "known unknowns" handling in all decision-heavy templates.

### 3) Verification-First Outputs
- Require output sections that map findings to evidence and validation commands.
- For critical workflows (security/migration/refactor), require rollback or fallback path in the output.
- Add strict schema checks for template outputs when feasible.

### 4) Anti-Hallucination Hardening
- Enforce "no invented APIs/files/dependencies" consistently in all templates.
- Require explicit uncertainty labeling and recommended defaults when context is incomplete.
- Prefer file-referenced statements over generic claims.

### 5) Brainpack Production Domains (Next)
- Prioritize entries for:
  - distributed tracing patterns
  - idempotency and retry-safe endpoint design
  - feature flags and canary rollout
  - backup/restore and disaster recovery runbooks
  - database migration rollback strategy
  - rate limiting and error budgets

## Implementation Order
1. Lock template structure consistency and variable contracts.
2. Add remaining high-impact brainpack entries for production reliability.
3. Add rule/memory docs and CLI guidance.
4. Add optional output-schema validation for critical templates.

## Success Criteria
- Reduced ambiguity in generated outputs.
- Higher consistency across templates and commands.
- Better production readiness in default AI guidance.
- Fewer follow-up corrections needed during build/review cycles.
