# Documentation Structure

This document describes the required structure and sections for fusabi-host documentation.

## Directory Layout

```
docs/
├── STRUCTURE.md              # This file - describes doc organization
├── RELEASE.md                # Release process and checklist
├── compat.md                 # Current version compatibility (symlink to latest)
└── versions/                 # Version-specific documentation
    ├── v0.1.0/               # Released version docs
    │   ├── compat.md
    │   ├── embedding.md
    │   └── features.md
    └── vNEXT/                # Unreleased/development docs
        ├── compat.md
        ├── embedding.md
        └── features.md
```

## Required Documentation Sections

### 1. Compatibility (compat.md)

**Purpose:** Version compatibility matrix and migration guides

**Required sections:**
- Version Matrix (fusabi-host vs Fusabi runtime)
- Fusabi LTS Alignment
- API Stability (stable vs unstable APIs)
- Migration Notes (breaking changes by version)
- Feature Flag Compatibility
- Platform Support Matrix
- Reporting Compatibility Issues

**Audience:** Integrators upgrading between versions

### 2. Embedding Guide (embedding.md)

**Purpose:** How to integrate fusabi-host into different application types

**Required sections:**
- Embedding Patterns (daemon, CLI, request-scoped)
- HostContext Integration (logging, metrics, cancellation)
- Resource Management (limits, capabilities, sandbox)
- Error Handling patterns
- Performance Tuning
- Testing Embedded Applications
- Production Checklist

**Audience:** Developers integrating fusabi-host

### 3. Feature Flags & Platform Support (features.md)

**Purpose:** Document platform support, feature flags, and future directions

**Required sections:**
- no_std Support Investigation
- WASM Support Investigation
- Platform Support Matrix
- MSRV (Minimum Supported Rust Version)
- Feature Flag Matrix
- Testing Feature Combinations

**Audience:** Platform maintainers and advanced users

### 4. Release Process (RELEASE.md)

**Purpose:** Document release steps for maintainers

**Required sections:**
- Pre-release Checklist
- Version Bumping Strategy
- Changelog Generation
- Publishing to crates.io
- GitHub Release Creation
- Post-release Tasks
- Hotfix Process

**Audience:** Project maintainers

## Documentation Standards

### Writing Style

- Use present tense ("returns" not "will return")
- Be direct and concise
- Include code examples for all major features
- Use `rust` code blocks with proper syntax highlighting
- Link to relevant source code on GitHub
- Keep examples self-contained and runnable

### Code Examples

All code examples should:
- Be valid Rust code
- Compile with current MSRV
- Include necessary imports
- Show both success and error paths
- Be tested (ideally as doc tests)

Good example:
```rust
use fusabi_host::{Engine, EngineConfig};

fn main() -> Result<(), fusabi_host::Error> {
    let engine = Engine::new(EngineConfig::default())?;
    let result = engine.execute("1 + 2")?;
    println!("Result: {:?}", result);
    Ok(())
}
```

### Markdown Conventions

- Use ATX-style headers (`#` not underlines)
- Use fenced code blocks with language specifiers
- Use relative links for internal documentation
- Use absolute GitHub URLs for source code links
- Keep line length under 100 characters
- Use tables for structured data

### Version-Specific Documentation

When making changes:

1. **For released versions (v0.1.0, v0.2.0, etc.):**
   - These are **immutable** after release
   - Only fix critical errors (broken links, security issues)
   - Document fixes in commit message

2. **For vNEXT (unreleased):**
   - This is the **active development** documentation
   - Update freely as APIs change
   - Reflects current `main` branch state

3. **Creating new version:**
   - On release, copy `vNEXT/` to `vX.Y.Z/`
   - Update version-specific references
   - Create new empty `vNEXT/` for next development cycle

## Documentation Validation

### CI Checks

The following checks run in CI:

```bash
# Check for broken internal links
find docs -name "*.md" -exec markdown-link-check {} \;

# Check for required sections
./scripts/check-docs-structure.sh

# Validate Rust code examples compile
cargo test --doc

# Check formatting
prettier --check "docs/**/*.md"
```

### Required Sections Validation

Each versioned directory must contain:
- `compat.md` with all required sections
- `embedding.md` with all required sections
- `features.md` with all required sections

### Link Validation

- All internal links must resolve
- All code examples must be valid
- All external links should be checked (warn only)

## Updating Documentation

### For API Changes

When changing public APIs:

1. Update `docs/versions/vNEXT/compat.md` with compatibility notes
2. Update code examples in `embedding.md`
3. Update feature flags in `features.md` if needed
4. Run `cargo test --doc` to verify examples

### For New Features

When adding features:

1. Document in `docs/versions/vNEXT/embedding.md`
2. Add to feature matrix in `features.md`
3. Update compatibility notes in `compat.md`
4. Add code examples to README.md
5. Write doc tests in source code

### For Breaking Changes

When introducing breaking changes:

1. Document in `CHANGELOG.md`
2. Add migration guide to `compat.md`
3. Update all affected examples
4. Consider deprecation period before removal
5. Update MSRV if needed

## Review Process

All documentation changes should:

1. Be reviewed for technical accuracy
2. Be checked for grammar/spelling
3. Have code examples tested
4. Have links validated
5. Follow the style guide

## Documentation Lifecycle

```
Development → vNEXT → Release → vX.Y.Z (immutable)
                 ↓
            Next vNEXT
```

1. **Development:** Update `vNEXT` as code changes
2. **Pre-release:** Review and finalize `vNEXT` docs
3. **Release:** Copy `vNEXT` to `vX.Y.Z`, create new `vNEXT`
4. **Maintenance:** `vX.Y.Z` docs are frozen, only critical fixes

## External Documentation

### README.md

The README should:
- Provide quick start example
- Link to versioned docs for details
- Show feature flags
- Reference compatibility table
- Link to API documentation (docs.rs)

### API Documentation (docs.rs)

Rust doc comments should:
- Explain what each item does
- Include examples for public APIs
- Link to relevant guide sections
- Document panics, errors, safety

### CHANGELOG.md

Follow [Keep a Changelog](https://keepachangelog.com/):
- Group changes by type (Added, Changed, Deprecated, etc.)
- Reference GitHub issues/PRs
- Include migration notes for breaking changes
- Update on every PR merge

## Documentation Ownership

| Document | Primary Owner | Reviewers |
|----------|---------------|-----------|
| compat.md | Release Manager | Core Team |
| embedding.md | Tech Lead | Community |
| features.md | Platform Team | Core Team |
| RELEASE.md | Release Manager | Tech Lead |
| STRUCTURE.md | Tech Lead | All |

## Continuous Improvement

Documentation should be:
- Reviewed quarterly for accuracy
- Updated based on user feedback
- Expanded based on common questions
- Refactored for clarity as needed

File issues for documentation improvements with label `documentation`.
