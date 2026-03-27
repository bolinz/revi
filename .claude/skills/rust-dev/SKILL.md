---
name: rust-dev
description: Rust development tasks - building, testing, linting
allowed-tools: Bash,Read,Edit,Write,Glob,Grep
---

# Rust Development Skill

## Commands

### Build
```bash
cargo build
cargo build --release
cargo build --features <feature>
```

### Test
```bash
cargo test
cargo test -- --nocapture
cargo test <test_name>
cargo clippy -- -D warnings
cargo fmt --check
```

### Documentation
```bash
cargo doc --open
cargo doc --no-deps
```

### Dependency Management
```bash
cargo update
cargo add <crate>
cargo remove <crate>
cargo tree
```

## Common Patterns

- Run `cargo check` for fast type checking without full compilation
- Use `cargo watch` for development with auto-rebuild
- Run `cargo audit` to check for security vulnerabilities
- Use `cargo-outdated` to check for outdated dependencies
