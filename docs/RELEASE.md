# Release Process

This document describes the release process for fusabi-host.

## Release Types

### Patch Release (0.1.x)

- Bug fixes only
- No new features
- No breaking changes
- Can be released on-demand

### Minor Release (0.x.0)

- New features
- Deprecations (with warnings)
- Performance improvements
- Can include MSRV bumps
- Released quarterly (or as needed)

### Major Release (x.0.0)

- Breaking API changes
- Removal of deprecated features
- Major architectural changes
- Released yearly (or as needed)

## Pre-Release Checklist

Before starting the release process:

- [ ] All CI checks are passing on `main`
- [ ] No open P0/P1 bugs in the milestone
- [ ] Documentation is up to date
- [ ] CHANGELOG.md is updated with all changes
- [ ] Version numbers are bumped appropriately
- [ ] All examples compile and run
- [ ] MSRV check passes

## Version Bumping Strategy

Follow [Semantic Versioning 2.0.0](https://semver.org/):

### Breaking Changes (Major)

Increment major version for:
- Removing public APIs
- Changing function signatures
- Changing behavior of existing features
- Removing feature flags
- Incompatible data format changes

### New Features (Minor)

Increment minor version for:
- Adding new public APIs
- Adding new feature flags
- New capabilities or sandbox options
- Performance improvements
- Deprecating (but not removing) APIs
- MSRV bumps

### Bug Fixes (Patch)

Increment patch version for:
- Bug fixes that don't change API
- Documentation fixes
- Internal refactoring
- Dependency updates (patch level)

## Release Steps

### 1. Prepare Release Branch

```bash
# Ensure main is up to date
git checkout main
git pull origin main

# Create release branch
git checkout -b release/v0.2.0

# Update version in Cargo.toml
# Edit: version = "0.2.0"
vim Cargo.toml

# Update version references in docs
find docs -name "*.md" -exec sed -i 's/0.1.x/0.2.x/g' {} \;
```

### 2. Update Documentation

```bash
# Copy vNEXT docs to new version
cp -r docs/versions/vNEXT docs/versions/v0.2.0

# Update version-specific references in new docs
cd docs/versions/v0.2.0
# Edit files to replace "vNEXT" with "v0.2.0"

# Create new vNEXT for next development cycle
cp -r docs/versions/v0.2.0 docs/versions/vNEXT

# Update symlink to compat.md
cd docs
ln -sf versions/v0.2.0/compat.md compat.md
```

### 3. Update CHANGELOG.md

Follow [Keep a Changelog](https://keepachangelog.com/) format:

```markdown
## [0.2.0] - 2025-12-05

### Added
- HostContext trait for logging/metrics/cancellation (#5)
- Embedding guide for daemon vs CLI patterns (#6)
- Release workflow with automated publishing (#4)

### Changed
- Updated MSRV to 1.75 for let-else syntax
- Improved error messages for capability violations

### Deprecated
- Direct Engine creation without EngineConfig (use Engine::new())

### Fixed
- Pool deadlock under high contention (#42)
- Memory leak in bytecode cache (#38)

### Documentation
- Added versioned docs structure (#3)
- Created STRUCTURE.md and RELEASE.md (#3, #4)
- Documented no_std and WASM investigation (#5)

[0.2.0]: https://github.com/fusabi-lang/fusabi-host/compare/v0.1.0...v0.2.0
```

### 4. Run Pre-Release Tests

```bash
# Clean build
cargo clean

# Check all features
cargo check --all-features
cargo check --no-default-features

# Run tests
cargo test --all-features

# Check MSRV
cargo +1.75 check --all-features

# Build docs
cargo doc --all-features --no-deps

# Run clippy
cargo clippy --all-targets --all-features -- -D warnings

# Check formatting
cargo fmt --all -- --check

# Validate examples
cargo build --examples --all-features
```

### 5. Create Release Commit

```bash
# Stage changes
git add -A

# Commit with standard message
git commit -m "chore: release v0.2.0

- Update version to 0.2.0
- Update CHANGELOG.md
- Finalize v0.2.0 docs
- Create vNEXT for next cycle

Closes #3, #4, #5, #6
"

# Push release branch
git push origin release/v0.2.0
```

### 6. Create Pull Request

```bash
# Create PR using gh CLI
gh pr create \
  --title "Release v0.2.0" \
  --body "$(cat <<EOF
# Release v0.2.0

This PR prepares the v0.2.0 release.

## Changes

See [CHANGELOG.md](CHANGELOG.md) for full details.

## Checklist

- [x] Version bumped in Cargo.toml
- [x] CHANGELOG.md updated
- [x] Documentation finalized
- [x] All CI checks passing
- [x] Examples tested
- [x] MSRV verified

## Related Issues

Closes #3, #4, #5, #6

## Post-Merge Steps

After merging, the release workflow will:
1. Build and test the crate
2. Publish to crates.io
3. Create GitHub release
4. Generate and attach artifacts

/cc @maintainers
EOF
)" \
  --base main
```

### 7. Review and Merge

- Wait for CI to pass
- Get approval from CODEOWNERS
- Merge to main (squash or merge commit, not rebase)

### 8. Tag and Publish (Automated)

After merge, the GitHub Actions workflow will automatically:

1. Detect the release commit
2. Create a git tag `v0.2.0`
3. Build the crate with `cargo build --release`
4. Run tests with `cargo test --all-features`
5. Publish to crates.io with `cargo publish`
6. Create GitHub release with:
   - CHANGELOG excerpt
   - Binary artifacts (if applicable)
   - Documentation links

### 9. Manual Steps (if automation fails)

If the automated workflow fails:

```bash
# Tag the release
git tag -a v0.2.0 -m "Release v0.2.0"
git push origin v0.2.0

# Publish to crates.io
cargo publish

# Create GitHub release manually
gh release create v0.2.0 \
  --title "v0.2.0" \
  --notes-file <(awk '/## \[0.2.0\]/,/## \[0.1.0\]/' CHANGELOG.md | head -n -1)
```

## Post-Release Tasks

After successful release:

- [ ] Verify crate published to crates.io
- [ ] Verify docs.rs built successfully
- [ ] Verify GitHub release created
- [ ] Update dependent projects
- [ ] Announce on social media / blog / forum
- [ ] Close release milestone
- [ ] Create next milestone

## Hotfix Process

For critical bugs in released versions:

### 1. Create Hotfix Branch

```bash
# Branch from the release tag
git checkout v0.2.0
git checkout -b hotfix/v0.2.1

# Make the fix
# ... edit files ...

# Update version to 0.2.1
vim Cargo.toml
```

### 2. Update CHANGELOG

```markdown
## [0.2.1] - 2025-12-10

### Fixed
- Critical security issue in sandbox validation (#123)
- Panic when pool size is zero (#124)

[0.2.1]: https://github.com/fusabi-lang/fusabi-host/compare/v0.2.0...v0.2.1
```

### 3. Fast-Track Release

```bash
# Commit fix
git commit -am "fix: critical security issue in sandbox validation

Fixes #123
"

# Push and create PR
git push origin hotfix/v0.2.1
gh pr create --title "Hotfix v0.2.1" --base main

# After approval, merge
# Tag and publish (automated or manual)
git tag v0.2.1
git push origin v0.2.1
cargo publish
```

### 4. Backport to Main

```bash
# Cherry-pick hotfix to main
git checkout main
git cherry-pick <hotfix-commit-sha>
git push origin main
```

## Release Cadence

- **Patch releases:** As needed (typically within days of bug discovery)
- **Minor releases:** Quarterly (January, April, July, October)
- **Major releases:** Annually (January)

## Branch Protection Rules

The `main` branch has the following protections:

- Require pull request reviews (1 approver)
- Require status checks to pass:
  - CI checks (fmt, clippy, test, doc)
  - MSRV check
  - Feature matrix tests
- Require branches to be up to date
- No force pushes
- No deletions

## Rollback Procedure

If a release has critical issues:

### 1. Yank from crates.io

```bash
cargo yank --vers 0.2.0
```

### 2. Create Hotfix

Follow hotfix process to release patched version.

### 3. Communicate

- Update GitHub release notes with warning
- Post announcement in community channels
- Update security advisories if applicable

## Semantic Versioning and crates.io

### Pre-1.0 Versions (0.x.y)

While fusabi-host is pre-1.0:
- Minor bumps (0.x) can include breaking changes
- Patch bumps (0.x.y) are for compatible fixes only
- We maintain a CHANGELOG for all breaking changes

### Post-1.0 Versions

Once fusabi-host reaches 1.0:
- Strict semver compliance
- Breaking changes only in major versions
- Longer deprecation periods (at least one minor version)

## Automation Tools

### Release Workflow (.github/workflows/release.yml)

Triggers on:
- Tags matching `v*.*.*`
- Workflow dispatch (manual trigger)

Jobs:
1. Build and test
2. Publish to crates.io
3. Create GitHub release
4. Update documentation

### Version Bump Script (scripts/bump-version.sh)

```bash
./scripts/bump-version.sh patch   # 0.2.0 -> 0.2.1
./scripts/bump-version.sh minor   # 0.2.0 -> 0.3.0
./scripts/bump-version.sh major   # 0.2.0 -> 1.0.0
```

Updates:
- Cargo.toml version
- CHANGELOG.md template
- Documentation version references

## Changelog Generation

Use [git-cliff](https://git-cliff.org/) for automated changelog generation:

```bash
# Generate changelog for current version
git cliff --latest --output CHANGELOG.md

# Generate full changelog
git cliff --output CHANGELOG.md
```

Configuration in `.cliff.toml`.

## Release Checklist Template

Copy this checklist for each release:

```markdown
## Release v0.X.Y Checklist

### Pre-Release
- [ ] All CI passing on main
- [ ] No P0/P1 bugs
- [ ] CHANGELOG.md updated
- [ ] Version bumped in Cargo.toml
- [ ] Docs finalized (vNEXT -> vX.Y.Z)
- [ ] Examples tested
- [ ] MSRV verified

### Release
- [ ] Release branch created
- [ ] PR created and reviewed
- [ ] PR merged to main
- [ ] Tag created (v0.X.Y)
- [ ] Published to crates.io
- [ ] GitHub release created

### Post-Release
- [ ] docs.rs built successfully
- [ ] Dependent projects updated
- [ ] Release announcement posted
- [ ] Milestone closed
- [ ] Next milestone created
```

## Contact

For questions about the release process:
- Open an issue with label `release`
- Contact @release-manager on Discord
- Email: releases@fusabi-lang.org
