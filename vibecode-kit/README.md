# Vibecode Kit V4 (English Edition)

> A comprehensive collection of AI-ready prompts for vibe coding with partnership philosophy.

## Philosophy

Vibecode Kit transforms the AI-human collaboration from a "client-contractor" relationship to an **"Architect-Homeowner Partnership"**. The AI is not just executing orders—it's a creative partner that understands your vision and helps build it together.

### Core Principles

| Principle | Description |
|-----------|-------------|
| **Partnership** | AI as collaborative architect, not just order executor |
| **Vision First** | Understand the "why" before the "what" |
| **Iterative Refinement** | Build-test-improve cycles |
| **Quality by Design** | Built-in testing, security, and performance |
| **Human-Friendly** | Clear communication, no jargon dumps |

## Prompts

### Core Development

| Prompt | Purpose |
|--------|---------|
| [VIBECODE-MASTER](prompts/VIBECODE-MASTER.md) | Universal project scaffold with 6 project types |
| [DEBUG-MASTER](prompts/DEBUG-MASTER.md) | 9-step systematic debugging protocol |
| [QA-MASTER](prompts/QA-MASTER.md) | 3-tier testing (Core/Edge/Performance) |
| [XRAY-MASTER](prompts/XRAY-MASTER.md) | Handover & documentation protocol |

### Security & Performance

| Prompt | Purpose |
|--------|---------|
| [SECURITY-MASTER](prompts/SECURITY-MASTER.md) | OWASP Top 10 security audit |
| [PERFORMANCE-MASTER](prompts/PERFORMANCE-MASTER.md) | Core Web Vitals optimization |

### Integration

| Prompt | Purpose |
|--------|---------|
| [INTEGRATION-MASTER](prompts/INTEGRATION-MASTER.md) | Third-party APIs, OAuth, payments, webhooks |

## Templates

| Template | Purpose |
|----------|---------|
| [E-COMMERCE](templates/E-COMMERCE.md) | Full e-commerce with products, cart, checkout, admin |

## Quick Start

### 1. Choose Your Project Type

```
I want to build a [Landing Page / SaaS / Dashboard / Blog / Portfolio / E-Commerce].

Project: [Your project name]
Core features:
1. [Feature 1]
2. [Feature 2]
3. [Feature 3]

Tech preferences: [Next.js, React, Vue, etc. or "recommend"]
```

### 2. Start with VIBECODE-MASTER

Copy the [VIBECODE-MASTER](prompts/VIBECODE-MASTER.md) prompt into your AI assistant and provide your project description. The AI will:

1. **Clarify** - Ask questions to understand your vision
2. **Scaffold** - Create project structure
3. **Build** - Implement features iteratively
4. **Test** - Verify functionality works
5. **Document** - Explain what was built and how to extend

### 3. Use Specialized Prompts

As your project grows, use specialized prompts:

- **Found a bug?** → Use DEBUG-MASTER
- **Need tests?** → Use QA-MASTER
- **Security review?** → Use SECURITY-MASTER
- **Slow performance?** → Use PERFORMANCE-MASTER
- **Adding integrations?** → Use INTEGRATION-MASTER
- **Handover/docs?** → Use XRAY-MASTER

## Workflow

```
┌─────────────────────────────────────────────────────────────┐
│                    VIBECODE WORKFLOW                         │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│   1. VISION          What are we building and why?           │
│      ↓                                                       │
│   2. SCAFFOLD        Create project structure                │
│      ↓                                                       │
│   3. BUILD           Implement features (iterative)          │
│      ↓                                                       │
│   4. TEST            Verify it works                         │
│      ↓                                                       │
│   5. REFINE          Optimize, secure, polish                │
│      ↓                                                       │
│   6. DOCUMENT        Explain for handover                    │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

## Best Practices

### DO

- Start with clear project goals
- Provide context about your tech stack preferences
- Ask the AI to explain decisions
- Review code before accepting
- Test incrementally
- Use specialized prompts for specific tasks

### DON'T

- Skip the vision/clarification phase
- Accept code without understanding it
- Ignore test failures
- Rush through security considerations
- Forget to document

## Customization

All prompts are designed to be customizable:

1. **Tech Stack**: Mention your preferred technologies
2. **Style**: Specify coding conventions
3. **Complexity**: Indicate if you want simple or advanced patterns
4. **Context**: Provide existing code or constraints

## Contributing

This is an open collection. Feel free to:

- Add new prompts for specific frameworks
- Improve existing prompts based on experience
- Share templates for common project types
- Translate to other languages

## License

MIT License - Use freely for any project.

---

## Related Resources

- [VibeAnvil](https://github.com/ThanhNguyxn/vibeanvil) - Contract-first vibe coding CLI
- [Original Vibecode Kit (Vietnamese)](https://github.com/ThanhNguyxn/vibecode-kit) - Original version

---

*Built for developers who want AI as a true partner, not just a code generator.*
