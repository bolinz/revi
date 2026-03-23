# Decisions

## Confirmed

- Revi uses a lightweight release flow with `main`, `feat/*`, `fix/*`, and `hotfix/*`
- Revi releases are triggered by manual `vX.Y.Z` tags
- GitHub Actions are the release automation entrypoint
- `main` is protected and requires pull requests plus the `cargo-test` check
- Revi supports multiple templates, including a stack-agnostic `generic-project`
- Generic template enhancement is the current `v0.1.2` focus

## Open / Active

- How far generic-template configuration should go before a more structured template-config model is needed
- Whether future releases should add crates.io publishing
- Whether template rendering should stay code-driven or evolve toward more manifest-driven generation
- Whether AI-agent-specific files should expand beyond the current brief / architecture / decisions set

## Guardrails

- Do not add stack-specific files to `generic-project`
- Do not introduce heavy Git flow variants such as `develop` or `release/*`
- Do not change release semantics away from tag-driven publishing without revisiting repository governance docs
- Do not expand configuration in a way that breaks existing template defaults
