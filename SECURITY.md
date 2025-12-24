# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Reporting a Vulnerability

If you discover a security vulnerability in VibeAnvil, please report it responsibly:

1. **Do NOT** open a public GitHub issue for security vulnerabilities
2. Email your findings to: security@example.com (replace with actual address)
3. Include:
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact
   - Suggested fix (if any)

## Response Timeline

- **Initial response**: Within 48 hours
- **Status update**: Within 7 days
- **Fix timeline**: Depends on severity
  - Critical: 24-48 hours
  - High: 7 days
  - Medium: 30 days
  - Low: Next release

## Security Features

### Secret Detection & Redaction

VibeAnvil automatically redacts secrets from evidence logs:

- API keys (OpenAI, AWS, GitHub tokens)
- Passwords in environment variables
- Bearer tokens
- Other common secret patterns

### Audit Trail

All operations are logged to `.vibeanvil/logs/audit.jsonl` for forensic analysis.

### Contract Locking

Once locked, contracts are immutable. The lock includes:
- SHA-256 hash of canonical contract JSON
- Tool version
- Schema version

### Scope Lock for Harvesting

The LOCKFILE.json freezes research scope:
- Exact search queries used
- Selected repository URLs
- Commit SHAs

## Best Practices

1. **Don't commit secrets**: Use `.gitignore` for `.vibeanvil/sessions/`
2. **Review evidence**: Check captured logs for accidental secret exposure
3. **Verify downloads**: Always verify checksums when installing
4. **Keep updated**: Install security updates promptly

## Known Limitations

- Secret detection is pattern-based and may not catch all secrets
- Evidence files may contain sensitive data despite redaction
- BrainPack harvesting only supports public repositories
