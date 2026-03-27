---
name: revi-dev
description: Revi project development - bootstrapping, testing, releasing
allowed-tools: Bash,Read,Edit,Write,Glob,Grep,Agent
---

# Revi Development Skill

## Commands

### Build & Run
```bash
cargo build --release
cargo run -- --help
cargo run -- init --help
```

### Testing
```bash
cargo test
cargo test -- --nocapture
cargo clippy -- -D warnings
```

### Project Templates
```bash
cargo run -- templates list
cargo run -- init --template generic-project --path /tmp/test
```

### Doctor & Diagnostics
```bash
cargo run -- doctor
```

## Git Workflow

```bash
# Create feature branch
git checkout -b feat/<name>

# Run tests before commit
cargo test

# Commit with conventional format
git commit -m "feat: description"

# Create PR via GitHub CLI
gh pr create
```

## Release Process

1. Update CHANGELOG.md with changes
2. Create vX.Y.Z tag
3. Push tag to trigger GitHub Actions release
