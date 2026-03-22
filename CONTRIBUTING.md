# Contributing

## Project Workflow

Revi uses a lightweight release flow:

- `main` is the only long-lived branch and should stay releasable
- `feat/<topic>` is for new features
- `fix/<topic>` is for bug fixes
- `hotfix/<topic>` is for urgent fixes to already published behavior

Do not introduce `develop`, `release/*`, or other long-lived integration branches.

## Main Branch Protection

The `main` branch is expected to be protected in GitHub:

- changes land through pull requests
- direct pushes are not part of the normal workflow
- at least one approval is required before merge
- required status checks must pass before merge
- force-pushes and branch deletion are disabled

## Local Development

Recommended checks before opening a pull request:

```bash
cargo test
cargo run -- --help
cargo run -- init --non-interactive --name "Smoke Test" --template node-web --path /tmp/revi-smoke
```

## Pull Requests

- Keep each PR scoped to one logical change
- Update docs when the user-facing behavior changes
- Call out release impact if the change affects versioning, tags, release notes, or generated workflows
- Include the exact validation commands you ran

## Versioning And Releases

Revi uses semantic versioning and manual tag-based releases.

Release steps:

1. Merge the intended changes into `main`
2. Update `Cargo.toml` and `CHANGELOG.md`
3. Run local validation
4. Create a release commit on `main` if needed
5. Create tag `vX.Y.Z`
6. Push `main`
7. Push the tag
8. GitHub Actions builds release assets and publishes or updates the GitHub Release automatically

Release notes are extracted from `CHANGELOG.md`. If the tagged version does not match `Cargo.toml` or does not have a changelog entry, the release workflow should fail.

## Areas Requiring Extra Care

- Changes to generated workflow files or branch/release defaults
- Changes to CLI command names or config filenames
- Changes to repository bootstrap behavior through `git` or `gh`
