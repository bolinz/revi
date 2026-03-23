# Revi

`Revi` is an AI-native bootstrap and release tool for code projects.

## Install

```bash
cargo run -- --help
```

Current releases are published from Git tags. GitHub Releases are expected to include automated multi-platform binaries starting with `v0.1.1`.

## Commands

```bash
cargo run -- templates list
cargo run -- doctor
cargo run -- init
cargo run -- init --config /path/to/revi.toml
```

## Templates

- `python-service`
- `node-web`
- `desktop-tauri`
- `generic-project`

Each generated project includes:

- `README.md`
- `CONTRIBUTING.md`
- `.gitignore`
- `CHANGELOG.md`
- `revi.toml`
- optional `.github` workflows and issue/PR templates

## Git Flow

- `main` is the only long-lived branch and should stay releasable
- feature work uses `feat/<name>`
- bug fixes use `fix/<name>`
- urgent release fixes use `hotfix/<name>`
- official releases are created from `vX.Y.Z` tags

Detailed contributor and release rules live in [CONTRIBUTING.md](./CONTRIBUTING.md).

## Branch Protection

- `main` should be protected in GitHub
- pull requests are the default path for merging changes
- at least one approval is required before merge
- required CI checks must pass before merge
- force-pushes and branch deletion are disabled

## Release Policy

- Versioning follows semantic versioning
- Releases are cut manually by tagging `vX.Y.Z`
- `main` is the single source of truth for the next release
- GitHub Actions publishes release assets and notes from tag pushes
- `v0.1.0` is the initial source-first public release

## Project Files

- License: [LICENSE](./LICENSE)
- Changelog: [CHANGELOG.md](./CHANGELOG.md)
- Contributing guide: [CONTRIBUTING.md](./CONTRIBUTING.md)
- Security policy: [SECURITY.md](./SECURITY.md)

## Agent Context

- Project brief: [docs/PROJECT_BRIEF.md](./docs/PROJECT_BRIEF.md)
- Architecture notes: [docs/ARCHITECTURE.md](./docs/ARCHITECTURE.md)
- Decision log: [docs/DECISIONS.md](./docs/DECISIONS.md)

## Notes

- Local scaffold creation initializes Git and creates an initial commit by default.
- GitHub repository creation is optional and uses the `gh` CLI when enabled.
- Re-running `init --config revi.toml` is intended to be safe for unchanged files.
