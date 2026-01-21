# Provider Capability Matrix & Auto-Selection

VibeAnvil includes a comprehensive capability matrix for all 24+ AI coding providers, enabling automatic provider selection based on task requirements.

## Capability Matrix

View the full capability matrix:

```bash
vibeanvil providers matrix
```

### Provider Tiers

| Tier | Description | Examples |
|------|-------------|----------|
| 1 | Premium Agentic AI | Claude Code, GitHub Copilot Agent, Cursor Agent, Windsurf, Augment |
| 2 | Standard AI Tools | Aider, Cline, Claude API, OpenAI API, Gemini API |
| 3 | Specialized Tools | Sourcery, Grit, Amazon Q, Tabnine |
| 4 | Local/Offline | Ollama, LMStudio, Continue.dev |

### Capabilities Tracked

| Code | Capability | Description |
|------|------------|-------------|
| GEN | Code Generation | Can generate new code |
| REV | Code Review | Can review existing code |
| EXP | Code Explanation | Can explain code |
| REF | Refactoring | Can refactor/improve code |
| FIX | Bug Fixing | Can fix bugs |
| TST | Test Generation | Can write tests |
| DOC | Documentation | Can write documentation |
| ARC | Architecture | Can analyze architecture |
| PLN | Planning | Can plan implementation |
| DBG | Debugging | Can debug issues |
| SHL | Shell Commands | Can run terminal commands |
| MUL | Multi-File | Can work with multiple files |
| SRC | Code Search | Can search codebase |
| WEB | Web Access | Can access internet |
| VIS | Vision | Can handle images |
| STR | Streaming | Supports response streaming |
| INT | Interactive | Supports interactive mode |
| AGT | Agentic | Has autonomous capabilities |
| BKG | Background | Can run in background |
| MCP | MCP Support | Supports MCP protocol |

## Auto-Selection

Get provider recommendations for a specific task:

```bash
vibeanvil providers recommend "fix the authentication bug"
```

### How It Works

1. **Task Type Inference**: Analyzes the task description to determine type:
   - Code Generation
   - Code Review
   - Refactoring
   - Bug Fixing
   - Test Writing
   - Documentation
   - Architecture Design
   - Planning
   - Debugging
   - Multi-File Editing
   - Shell Tasks
   - Explanation

2. **Capability Matching**: Maps task types to required capabilities:
   - Bug Fix → BugFixing, Debugging, CodeGeneration
   - Code Review → CodeReview, CodeExplanation
   - Multi-File Edit → MultiFile, Agentic, CodeGeneration

3. **Scoring & Ranking**: Scores providers based on:
   - Required capability scores (10x weight)
   - Preferred capability scores (3x weight)
   - Provider tier bonus
   - Cost/latency penalties (if requested)

## Compare Providers

Compare specific providers side-by-side:

```bash
vibeanvil providers compare claude_code github_copilot_agent aider
```

Output shows:
- Capability scores for each
- Tier comparison
- Cost per 1K tokens
- Context window size

## Selection Criteria

When selecting a provider programmatically:

```rust
use vibeanvil::provider::{ProviderSelector, SelectionCriteria, TaskType};

let selector = ProviderSelector::new();

// Basic selection
let criteria = SelectionCriteria::for_task(TaskType::BugFix);
let best = selector.select(&criteria);

// With constraints
let criteria = SelectionCriteria::for_task(TaskType::CodeGeneration)
    .agentic()      // Require agentic capabilities
    .local()        // Require local/offline
    .low_cost()     // Prefer lower cost
    .fast()         // Prefer lower latency
    .exclude_provider("openai_api");

let best = selector.select(&criteria);
```

## Provider Chain

Get a fallback chain of providers:

```bash
# Get top 3 providers for a task
vibeanvil providers recommend "implement REST API" --count 3
```

Use in automation:
```bash
# Try first provider, fall back if unavailable
for provider in $(vibeanvil providers recommend "task" --json | jq -r '.[].name'); do
    if vibeanvil build iterate --provider "$provider"; then
        break
    fi
done
```

## Provider Profiles

Each provider has a profile with:

- **Name**: Unique identifier
- **Tier**: 1-4 (premium to local)
- **Capabilities**: Score 0-10 for each capability
- **Cost**: Estimated cost per 1K tokens
- **Latency**: Typical response time in ms
- **Context Window**: Maximum context size
- **Tags**: Categories (agentic, ide, api, local, etc.)

## Adding Custom Providers

Extend the capability matrix with custom providers:

```rust
use vibeanvil::provider::{CapabilityMatrix, ProviderProfile, Capability};

let mut matrix = CapabilityMatrix::build_default();

matrix.add(ProviderProfile::new("my_custom_provider", 2)
    .with_capability(Capability::CodeGeneration, 8)
    .with_capability(Capability::Agentic, 7)
    .with_cost(0.005)
    .with_latency(1000)
    .with_context(64000)
    .with_tag("custom"));
```

## Best Practices

1. **Use Recommendations**: Let VibeAnvil suggest providers based on task
2. **Set Up Fallbacks**: Configure multiple providers for reliability
3. **Consider Cost**: Use `--low-cost` for non-critical tasks
4. **Local First**: Use `--local` for sensitive codebases
5. **Match Capabilities**: Agentic tasks need agentic providers
