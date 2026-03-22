# Revi

`Revi` is an AI-native bootstrap and release tool for code projects.

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

Each generated project includes:

- `README.md`
- `CONTRIBUTING.md`
- `.gitignore`
- `CHANGELOG.md`
- `revi.toml`
- optional `.github` workflows and issue/PR templates

## Workflow Defaults

- `main` stays releasable
- feature branches use `feat/<name>`
- bug fixes use `fix/<name>`
- urgent release blockers use `hotfix/<name>`
- releases are triggered from `vX.Y.Z` tags

## Notes

- Local scaffold creation initializes Git and creates an initial commit by default.
- GitHub repository creation is optional and uses the `gh` CLI when enabled.
- Re-running `init --config revi.toml` is intended to be safe for unchanged files.
