---
name: release-workflow
description: Release workflow - versioning, tagging, GitHub Actions
allowed-tools: Bash,Read,Edit,Write,Glob,Grep
---

# Release Workflow Skill

## Git Flow

- `main` - production-ready code only
- `feat/<name>` - feature branches
- `fix/<name>` - bug fix branches
- `hotfix/<name>` - urgent fixes

## Release Commands

### Tagging
```bash
# Create version tag
git tag v0.1.0
git push origin v0.1.0

# List tags
git tag -l

# Delete local tag
git tag -d v0.1.0

# Delete remote tag
git push origin --delete v0.1.0
```

### GitHub CLI
```bash
# Create release
gh release create v0.1.0 --title "Release v0.1.0" --notes "Release notes"

# View releases
gh release list

# Download release assets
gh release download v0.1.0
```

## GitHub Actions

- CI runs on push to `main` and on pull requests
- Release builds triggered on tag push `v*`
- Release assets published to GitHub Releases

## Versioning

Semantic versioning: MAJOR.MINOR.PATCH
- MAJOR: breaking changes
- MINOR: new features (backwards compatible)
- PATCH: bug fixes
