# 🎯 Harvest Presets Guide

Learn how to use presets for efficient, focused BrainPack harvesting.

---

## 📋 Overview

Harvest presets are pre-configured search queries and filters designed to capture specific patterns. They help you:

- **Focus**: Target specific knowledge domains
- **Consistency**: Use proven query combinations
- **Efficiency**: Skip trial-and-error query tuning
- **Privacy**: No hardcoded repository URLs

---

## 🚀 Quick Start

### Using a Preset

```bash
# Use a preset (if CLI supports --preset)
vibeanvil harvest --preset workflow_state_machines

# Or manually apply preset settings
vibeanvil harvest \
  -q "state machine workflow cli" \
  -q "fsm finite state" \
  -l rust \
  --min-stars 100 \
  --max-repos 20
```

### List Available Presets

```bash
# View presets file
cat brainpacks/presets.yaml
```

---

## 📦 Built-in Presets

| Preset | Purpose | Signal Focus |
|--------|---------|--------------|
| `workflow_state_machines` | State machines and workflow patterns | `state_machine` |
| `contract_first_specs` | Contract-first development | `contract_lock` |
| `evidence_audit_logging` | Audit logs and evidence capture | `evidence_audit` |
| `iterate_test_fix_loops` | Automated test-fix patterns | `iterate_loop` |
| `release_installers_checksums` | Release and distribution | `release_install` |
| `cli_framework_patterns` | CLI tool structures | `command_surface` |
| `security_redaction_ignore_rules` | Secret detection and redaction | `security_pattern` |
| `templates_and_scaffolding` | Project scaffolding | `command_surface` |
| `docs_quality_onboarding` | Documentation patterns | `evidence_audit` |
| `provider_adapters` | AI/plugin integrations | `provider_adapter` |
| `error_handling_patterns` | Error handling and recovery | `iterate_loop` |
| `configuration_management` | Config file patterns | `command_surface` |

---

## 🔧 Preset Structure

Each preset in `brainpacks/presets.yaml` contains:

```yaml
preset_name:
  name: "Human-readable name"
  purpose: "What this preset captures"
  signals:
    - signal_type  # e.g., state_machine, iterate_loop
  queries:
    - "search query one"
    - "search query two"
  filters:
    min_stars: 50
    updated_within_days: 365
    languages:
      - Rust
      - Go
  allow_globs:
    - "**/*.rs"
    - "**/relevant_path/*"
```

### Fields

| Field | Required | Description |
|-------|----------|-------------|
| `name` | Yes | Display name |
| `purpose` | Yes | What patterns to capture |
| `signals` | Yes | BrainSignal types to tag results |
| `queries` | Yes | GitHub search query strings |
| `filters` | No | Override default filters |
| `allow_globs` | No | File patterns to include |

---

## ✏️ Creating Custom Presets

### 1. Add to presets.yaml

```yaml
presets:
  my_custom_preset:
    name: "My Custom Patterns"
    purpose: "Find specific patterns I need"
    signals:
      - command_surface
    queries:
      - "my specific query"
      - "another related query"
    filters:
      min_stars: 100
      languages:
        - Python
```

### 2. Apply Manually

Until CLI preset loading is implemented:

```bash
# Translate preset to harvest command
vibeanvil harvest \
  -q "my specific query" \
  -q "another related query" \
  -l python \
  --min-stars 100 \
  --max-repos 20
```

---

## 🔍 Query Writing Tips

### Effective Patterns

```yaml
# Combine related terms
queries:
  - "state machine workflow"      # Broad
  - "fsm finite state transition" # Specific
  - "step wizard flow"            # Alternative terms

# Use GitHub search syntax
queries:
  - "language:Rust stars:>100 state machine"
  - "topic:cli workflow"
```

### What to Avoid

- ❌ Too broad: `"code"`, `"project"`, `"app"`
- ❌ Too specific: Single file names
- ❌ Repository names: No hardcoded repos
- ❌ External URLs: Never include in presets

---

## 🎛️ Filters Reference

| Filter | Type | Description |
|--------|------|-------------|
| `min_stars` | int | Minimum GitHub stars |
| `updated_within_days` | int | Freshness filter |
| `max_repos` | int | Limit repos per harvest |
| `languages` | list | Programming languages |
| `allow_globs` | list | File patterns to include |
| `ignore_globs` | list | File patterns to exclude |

### Recommended Values

| Use Case | min_stars | updated_within_days | max_repos |
|----------|-----------|---------------------|-----------|
| High quality | 500+ | 180 | 10 |
| Balanced | 100 | 365 | 20 |
| Broad search | 30 | 730 | 50 |

---

## 📊 Signal Types

Signals tag harvested content for focused searching:

| Signal | Description |
|--------|-------------|
| `command_surface` | CLI commands and argument parsing |
| `state_machine` | State transitions and workflows |
| `contract_lock` | Contract-first patterns |
| `iterate_loop` | Test-fix and iteration patterns |
| `evidence_audit` | Audit logging and evidence |
| `provider_adapter` | Plugin/adapter patterns |
| `security_pattern` | Security and redaction |

---

## 🔄 Combining Presets

Run multiple presets to build a comprehensive BrainPack:

```bash
# Quality CLI patterns
vibeanvil harvest --preset cli_framework_patterns
vibeanvil harvest --preset workflow_state_machines

# Developer experience
vibeanvil harvest --preset docs_quality_onboarding
vibeanvil harvest --preset templates_and_scaffolding

# Release and security
vibeanvil harvest --preset release_installers_checksums
vibeanvil harvest --preset security_redaction_ignore_rules
```

---

## 🔒 Privacy Notes

Presets follow VibeAnvil's privacy-first design:

- ✅ **No repo URLs** in preset files
- ✅ **Query strings only** - you control what's searched
- ✅ **Anonymized sources** - results use SHA-256 hashed IDs
- ✅ **Local storage** - all data stays on your machine

---

## 💡 Best Practices

1. **Start focused**: Use one preset at a time
2. **Check stats**: `vibeanvil brain stats` after each harvest
3. **Iterate**: Refine queries based on search results
4. **Deduplicate**: Multiple harvests won't create duplicates
5. **Export regularly**: Backup with `vibeanvil brain export`

---

Made with ❤️ by the VibeAnvil team
