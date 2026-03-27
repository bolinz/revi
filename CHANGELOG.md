# Changelog

## 0.1.5 - TBD

- Added Ollama provider for local LLM support
- Added Claude provider for Anthropic API
- Added AI generation integration in scaffold workflow
- Added project_type-aware skill generation
- Added fallback to static templates when AI unavailable

## 0.1.4 - 2026-03-27

- Added Ollama and Claude AI providers
- Updated README.md with skills and provider documentation
- Updated ARCHITECTURE.md with Provider architecture

## 0.1.3 - 2026-03-27

- Added AI Provider plugin architecture with `AiProvider` trait
- Implemented MiniMax provider for AI-generated content
- Added Claude Code skill files generation (`.claude/skills/`)
- Added agent configuration files generation
- Added `.claude/settings.json` with permissions for cargo, git, npm, gh
- Added pre-built skills: rust-dev, revi-dev, release-workflow
- Added AI coding CLI compatibility output for Codex, Claude Code, and Gemini CLI
- Added shared AI tool guidance and command helpers across all generated templates
- Added optional AI CLI checks to `revi doctor`

## 0.1.2 - 2026-03-23

- Enhanced `generic-project` for AI-agent-friendly project handoff
- Added configurable generic template options for agent context files, scripts, placeholder workflows, and expanded docs
- Added stack-agnostic next-step guidance and repository context files for generic projects

## 0.1.1 - 2026-03-23

- Added repository governance files for license, changelog, contributing, and security guidance
- Defined the official lightweight Git flow for the Revi repository
- Added GitHub Actions CI for `main` pushes and pull requests
- Added automated tag-driven release publishing with multi-platform binaries and changelog-backed release notes
- Documented branch protection expectations for `main`

## 0.1.0 - 2026-03-23

- Renamed the project to `Revi` and aligned the CLI, crate, and config filename
- Added interactive and non-interactive project bootstrap flows
- Added template support for Python service, Node/Web, and Desktop/Tauri projects
- Added local git bootstrap and optional GitHub repository creation hooks
- Added the `doctor` command for local environment checks
- Published the initial source-first GitHub release as `v0.1.0`
