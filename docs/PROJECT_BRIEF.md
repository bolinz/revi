# Project Brief

## Goal

Revi is an AI-native bootstrap and release tool for code projects. Its job is to initialize repositories with practical defaults, Git flow conventions, release plumbing, and enough context for humans or AI agents to continue implementation safely.

## Primary Users

- Individual developers starting a new project quickly
- Small teams that want lightweight repository standards without a heavy platform
- AI coding agents that need a stable project skeleton and explicit project context

## What Revi Currently Does

- Initializes projects through `revi init`
- Supports `python-service`, `node-web`, `desktop-tauri`, and `generic-project` templates
- Writes project docs, changelog, gitignore, config, and optional GitHub workflows/templates
- Can bootstrap local git state and optionally create GitHub repositories
- Publishes Revi itself through tag-driven GitHub Releases with multi-platform binaries

## Current Focus

The active next-version work is improving `generic-project` so it becomes a better handoff template for AI agents:

- stack-agnostic next-step guidance
- configurable generic template options
- stable project context files

## Out Of Scope For Now

- crates.io publishing
- deep cloud deployment integrations
- large-scale enterprise workflow customization
- automatic language/framework inference for generic templates

## Success Criteria

- Revi can generate useful starting points without locking users into one stack
- Generated repositories are understandable to both humans and agents
- Repository governance and release flow stay lightweight and consistent
