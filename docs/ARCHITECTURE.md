# Architecture

## Repository Shape

- `src/cli.rs`: CLI argument model and command surface
- `src/main.rs`: command dispatch entrypoint
- `src/config.rs`: persisted config model for generated projects
- `src/wizard.rs`: interactive and non-interactive config resolution
- `src/catalog.rs`: embedded template catalog and manifest loading
- `src/scaffold.rs`: file generation logic per template
- `src/bootstrap.rs`: local git bootstrap and optional GitHub repo creation
- `src/doctor.rs`: local environment checks
- `src/providers/`: AI Provider plugins
- `templates/`: embedded template manifests

## Runtime Flow

1. Parse CLI input.
2. Resolve project config from flags, interactive prompts, or `revi.toml`.
3. Load the selected template manifest from the embedded catalog.
4. Generate files for the selected template.
5. Optionally initialize git, create the initial commit, and attempt GitHub bootstrap.

## Template Model

Revi currently uses a typed template enum plus per-template branching in scaffold code.

Implications:

- Fast to evolve for a small template set
- Easy to keep strong defaults per template
- More template-specific branching in Rust as template count and options grow

The current `generic-project` work is the first place where template-specific configuration is being introduced beyond simple template selection.

## Release Model

- `main` is the only long-lived branch
- GitHub Actions run CI on pushes to `main` and on pull requests
- Tag pushes `vX.Y.Z` trigger release builds
- Release notes are sourced from `CHANGELOG.md`
- Revi itself is distributed as GitHub Release assets

## Important Constraints

- Keep public CLI interfaces stable unless there is a strong reason to change them
- Avoid diverging between Revi's own repository workflow and the workflows it generates
- Generic templates must remain stack-agnostic
- Configuration expansion should stay minimal and avoid breaking existing templates

## AI Provider Architecture

Revi uses a plugin-based AI Provider system:

```
src/providers/
├── mod.rs          # AiProvider trait and ProviderRegistry
├── minimax.rs      # MiniMax API provider
├── ollama.rs       # Local Ollama LLM provider
└── claude.rs      # Anthropic Claude API provider
```

### AiProvider Trait

```rust
#[async_trait]
pub trait AiProvider: Send + Sync {
    fn name(&self) -> &str;
    async fn generate_skill(&self, context: &SkillContext) -> Result<String>;
    async fn generate_agent(&self, context: &AgentContext) -> Result<String>;
}
```

### Adding a New Provider

1. Create `src/providers/<name>.rs` implementing `AiProvider`
2. Export in `src/providers/mod.rs`
3. Add case to `create_provider()` function
