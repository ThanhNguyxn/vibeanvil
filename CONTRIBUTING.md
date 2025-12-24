# Contributing to VibeAnvil

Thank you for your interest in contributing to VibeAnvil!

## Development Setup

1. **Prerequisites**
   - Rust 1.75+ (stable)
   - Git

2. **Clone and Build**
   ```bash
   git clone https://github.com/OWNER/vibeanvil.git
   cd vibeanvil
   cargo build
   ```

3. **Run Tests**
   ```bash
   cargo test
   cargo clippy --all-targets -- -D warnings
   cargo fmt --check
   ```

## Contribution Workflow

1. **Fork** the repository
2. **Create a branch** for your feature: `git checkout -b feature/my-feature`
3. **Make changes** following our code style
4. **Add tests** for new functionality
5. **Run CI checks locally**: `cargo test && cargo clippy && cargo fmt --check`
6. **Commit** with clear messages
7. **Push** and open a Pull Request

## Code Style

- Follow Rust idioms and best practices
- Use `rustfmt` for formatting
- Fix all `clippy` warnings
- Add documentation comments for public APIs
- Keep functions focused and small

## Pull Request Guidelines

- **Title**: Clear, descriptive summary
- **Description**: Explain what and why
- **Tests**: Include tests for new features
- **Documentation**: Update docs if needed
- **Breaking changes**: Clearly mark and explain

## Areas for Contribution

### Priority

- [ ] Provider plugins (OpenAI, local LLMs)
- [ ] Homebrew tap formula
- [ ] Scoop manifest for Windows
- [ ] Additional evidence capture (screenshots, metrics)
- [ ] Interactive contract builder

### Good First Issues

Look for issues labeled `good-first-issue` in the issue tracker.

## Architecture Overview

```
src/
├── main.rs          # Entry point
├── cli/             # Command handlers
├── state/           # State machine
├── contract/        # Contract management
├── evidence/        # Evidence capture
├── audit/           # JSONL audit logging
├── build/           # Build modes
├── provider/        # AI provider plugins
├── brain/           # BrainPack harvesting
└── workspace.rs     # Workspace management
```

## Testing

- **Unit tests**: In each module (use `#[cfg(test)] mod tests`)
- **Integration tests**: In `tests/` directory
- **Manual testing**: Run the full workflow

## Release Process

1. Update version in `Cargo.toml`
2. Update CHANGELOG.md
3. Create a git tag: `git tag v0.X.0`
4. Push tags: `git push --tags`
5. GitHub Actions will build and release

## Questions?

Open a GitHub Discussion or issue!
