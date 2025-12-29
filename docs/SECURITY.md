# Security

VibeAnvil includes built-in security hardening for filesystem operations.

## Path Validation

The `security` module provides protection against:

- **Path Traversal** - Blocks `../` and similar attempts to escape allowed directories
- **Absolute Paths** - Rejects absolute paths when relative expected
- **Null Bytes** - Prevents null byte injection attacks
- **Windows Reserved Names** - Blocks CON, PRN, AUX, NUL, COM1-9, LPT1-9

## Developer API

```rust
use crate::security::{validate_path, validate_filename, sanitize_filename};

// Validate path stays within base directory
let safe_path = validate_path(&base_dir, "user/input/file.txt")?;

// Validate filename is safe
validate_filename("config.json")?;

// Sanitize string into safe filename
let clean = sanitize_filename("unsafe<>name.txt"); // "unsafename.txt"
```

## Privacy Defaults

- Source IDs are SHA-256 hashed (anonymized)
- No external URLs stored in exports
- Secrets auto-redacted from evidence logs
