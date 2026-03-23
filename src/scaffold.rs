use std::{collections::BTreeMap, fs, path::PathBuf};

use anyhow::{Context, Result, bail};

use crate::{
    catalog::get_template,
    config::{StarterConfig, TemplateKind, write_if_changed},
};

pub fn scaffold(config: &StarterConfig) -> Result<PathBuf> {
    let project_dir = config.project.path.clone();
    if project_dir.exists() && !project_dir.is_dir() {
        bail!(
            "target path exists and is not a directory: {}",
            project_dir.display()
        );
    }
    fs::create_dir_all(&project_dir)
        .with_context(|| format!("failed to create {}", project_dir.display()))?;

    let template = get_template(config.project.template)?;
    let ctx = render_context(config, &template.manifest.default_runtime);
    let files = build_files(
        config,
        &ctx,
        &template.manifest.checks,
        &template.manifest.release_notes,
    );
    for (relative, content) in files {
        let path = project_dir.join(relative);
        write_if_changed(&path, &content)?;
    }
    config.save_to(&project_dir.join("revi.toml"))?;
    Ok(project_dir)
}

#[derive(Clone, Debug)]
struct LocalCommands {
    install: String,
    start: String,
    validate: String,
    release_prep: String,
    stack_specific: String,
}

fn render_context(config: &StarterConfig, runtime: &str) -> BTreeMap<&'static str, String> {
    let mut ctx = BTreeMap::new();
    ctx.insert("project_name", config.project.name.clone());
    ctx.insert("project_slug", config.project.slug.clone());
    ctx.insert("project_description", config.project.description.clone());
    ctx.insert("project_version", config.project.version.clone());
    ctx.insert(
        "template_kind",
        config.project.template.template_id().to_string(),
    );
    ctx.insert("default_runtime", runtime.to_string());
    ctx.insert("release_tag_example", "v0.1.0".to_string());
    ctx
}

fn build_files(
    config: &StarterConfig,
    ctx: &BTreeMap<&'static str, String>,
    checks: &[String],
    release_notes: &[String],
) -> BTreeMap<String, String> {
    let mut files = BTreeMap::new();
    let commands = local_commands(config);
    let readme_content = match config.project.template {
        TemplateKind::GenericProject => generic_readme(config, checks, &commands),
        _ => readme(config, checks, &commands),
    };
    files.insert(
        ".gitignore".to_string(),
        render(&gitignore(config.project.template), ctx),
    );
    files.insert("README.md".to_string(), render(&readme_content, ctx));
    files.insert(
        "CONTRIBUTING.md".to_string(),
        render(&contributing(config, checks), ctx),
    );
    files.insert("CHANGELOG.md".to_string(), render(changelog(), ctx));

    match config.project.template {
        TemplateKind::GenericProject => {
            files.insert("src/.gitkeep".to_string(), String::new());
            files.insert("docs/.gitkeep".to_string(), String::new());
            if config.generic.scripts_dir {
                files.insert("scripts/.gitkeep".to_string(), String::new());
            }
            if config.generic.docs_expanded {
                files.insert("docs/notes/.gitkeep".to_string(), String::new());
                files.insert("docs/runbooks/.gitkeep".to_string(), String::new());
            }
        }
        TemplateKind::PythonService => {
            files.insert(
                "pyproject.toml".to_string(),
                render(&python_pyproject(config), ctx),
            );
            files.insert("src/__init__.py".to_string(), String::new());
            files.insert("src/main.py".to_string(), python_main());
            files.insert("tests/test_smoke.py".to_string(), python_test());
        }
        TemplateKind::NodeWeb => {
            files.insert(
                "package.json".to_string(),
                render(&node_package_json(config), ctx),
            );
            files.insert("src/index.js".to_string(), node_index());
            files.insert(
                "dist/index.html".to_string(),
                node_dist_html(&config.project.name),
            );
            files.insert("tests/smoke.test.mjs".to_string(), node_test());
        }
        TemplateKind::DesktopTauri => {
            files.insert(
                "package.json".to_string(),
                render(&tauri_package_json(config), ctx),
            );
            files.insert("src/main.js".to_string(), tauri_frontend());
            files.insert(
                "dist/index.html".to_string(),
                node_dist_html(&config.project.name),
            );
            files.insert(
                "src-tauri/Cargo.toml".to_string(),
                render(&tauri_cargo(config), ctx),
            );
            files.insert("src-tauri/build.rs".to_string(), tauri_build_rs());
            files.insert("src-tauri/src/main.rs".to_string(), tauri_main());
            files.insert(
                "src-tauri/tauri.conf.json".to_string(),
                render(&tauri_conf(config), ctx),
            );
        }
    }

    if should_emit_project_context(config) {
        files.insert("docs/PROJECT_BRIEF.md".to_string(), project_brief(config));
        files.insert("docs/ARCHITECTURE.md".to_string(), architecture_doc(config));
        files.insert("docs/DECISIONS.md".to_string(), decisions_doc(config));
    }

    if config.ai_tools.enabled {
        if config.ai_tools.tool_docs {
            files.insert("docs/AI_TOOLS.md".to_string(), ai_tools_doc(config, &commands));
        }
        if config.ai_tools.codex {
            files.insert("AGENTS.md".to_string(), agents_md(config, &commands));
        }
        if config.ai_tools.claude_code {
            files.insert("CLAUDE.md".to_string(), claude_md(config, &commands));
        }
        if config.ai_tools.gemini_cli {
            files.insert("GEMINI.md".to_string(), gemini_md(config, &commands));
        }
    }

    if config.github.enabled {
        files.insert(
            ".github/pull_request_template.md".to_string(),
            pr_template(),
        );
        files.insert(
            ".github/ISSUE_TEMPLATE/bug_report.md".to_string(),
            issue_bug_template(),
        );
        files.insert(
            ".github/ISSUE_TEMPLATE/feature_request.md".to_string(),
            issue_feature_template(),
        );
        let include_workflows = config.project.template != TemplateKind::GenericProject
            || config.generic.placeholder_workflows;
        if include_workflows {
            files.insert(
                ".github/workflows/ci.yml".to_string(),
                render(&ci_workflow(config), ctx),
            );
            files.insert(
                ".github/workflows/release.yml".to_string(),
                render(&release_workflow(config, release_notes), ctx),
            );
        }
        if config.github.codeowners {
            files.insert(
                "CODEOWNERS".to_string(),
                "* @your-github-handle\n".to_string(),
            );
        }
    }

    files
}

fn should_emit_project_context(config: &StarterConfig) -> bool {
    match config.project.template {
        TemplateKind::GenericProject => config.generic.agent_context_files,
        _ => config.ai_tools.enabled && config.ai_tools.tool_docs,
    }
}

fn render(template: &str, ctx: &BTreeMap<&'static str, String>) -> String {
    let mut output = template.to_string();
    for (key, value) in ctx {
        output = output.replace(&format!("{{{{{key}}}}}"), value);
    }
    output
}

fn local_commands(config: &StarterConfig) -> LocalCommands {
    match config.project.template {
        TemplateKind::GenericProject => LocalCommands {
            install: "Choose your runtime first, then add the install command here.".to_string(),
            start: "Choose a stack-specific start command after selecting your language or framework."
                .to_string(),
            validate: "Replace this with your chosen test or validation command.".to_string(),
            release_prep: "Replace placeholder validation and packaging steps before cutting a release."
                .to_string(),
            stack_specific:
                "This template is intentionally stack-agnostic. Define install, run, test, build, and publish commands once the stack is chosen."
                    .to_string(),
        },
        TemplateKind::PythonService => LocalCommands {
            install: "python -m pip install -e \".[dev]\"".to_string(),
            start: "python src/main.py".to_string(),
            validate: "pytest -q".to_string(),
            release_prep:
                "pytest -q && docker build -t ghcr.io/<owner>/<repo>:latest .".to_string(),
            stack_specific:
                "Swap the placeholder service entrypoint for your framework of choice if you move beyond the generated smoke skeleton."
                    .to_string(),
        },
        TemplateKind::NodeWeb => LocalCommands {
            install: "npm install".to_string(),
            start: "npm run dev".to_string(),
            validate: "npm run ci".to_string(),
            release_prep: "npm run ci".to_string(),
            stack_specific:
                "Wire hosting or deployment commands into the release workflow after choosing your target platform."
                    .to_string(),
        },
        TemplateKind::DesktopTauri => LocalCommands {
            install: "npm install".to_string(),
            start: "npm run dev".to_string(),
            validate: "npm run ci".to_string(),
            release_prep: "npm run ci && npm run tauri build".to_string(),
            stack_specific:
                "Add signing, notarization, or updater setup before shipping production desktop builds."
                    .to_string(),
        },
    }
}

fn gitignore(kind: TemplateKind) -> &'static str {
    match kind {
        TemplateKind::GenericProject => {
            ".DS_Store\n.idea/\n.vscode/\ndist/\nbuild/\ncoverage/\n.tmp/\n"
        }
        TemplateKind::PythonService => "__pycache__/\n.pytest_cache/\n.venv/\ndist/\nbuild/\n",
        TemplateKind::NodeWeb => "node_modules/\ndist/\ncoverage/\n",
        TemplateKind::DesktopTauri => "node_modules/\ndist/\nsrc-tauri/target/\n",
    }
}

fn changelog() -> &'static str {
    "# Changelog\n\n## 0.1.0 - TBD\n\n- Initial scaffold generated by revi\n"
}

fn readme(config: &StarterConfig, checks: &[String], commands: &LocalCommands) -> String {
    let checks = checks
        .iter()
        .map(|item| format!("- `{item}`"))
        .collect::<Vec<_>>()
        .join("\n");
    let command_section = command_section(config, commands);
    let release_targets = match config.project.template {
        TemplateKind::GenericProject => {
            "- Source-first GitHub Release by default\n- Add stack-specific build or publish steps after choosing a language or framework".to_string()
        }
        TemplateKind::PythonService => {
            "- Container image publication via GHCR-compatible registry\n- GitHub Release notes for deployment guidance".to_string()
        }
        TemplateKind::NodeWeb => {
            "- GitHub Actions validation\n- Release workflow placeholder for platform deployment handoff".to_string()
        }
        TemplateKind::DesktopTauri => {
            "- Multi-platform desktop bundle release via GitHub Releases\n- Tag-driven Tauri publish workflow".to_string()
        }
    };
    let ai_section = ai_entrypoint_section(config);
    format!(
        "# {{{{project_name}}}}\n\n{{{{project_description}}}}\n\n## Template\n\n- Kind: `{{{{template_kind}}}}`\n- Runtime: `{{{{default_runtime}}}}`\n- Version: `{{{{project_version}}}}`\n\n## Git Workflow\n\n- Stable branch: `main`\n- Feature branches: `feat/<name>`\n- Bugfix branches: `fix/<name>`\n- Release blockers: `hotfix/<name>`\n- Release tags: `vX.Y.Z`\n\n## Commands\n\n{command_section}\n\n## Local Checks\n\n{checks}\n\n{ai_section}\n## Release\n\n{release_targets}\n\nRelease checklist:\n1. Merge work back to `main`\n2. Update version and `CHANGELOG.md`\n3. Tag `{{{{release_tag_example}}}}`\n4. Push `main` and the tag\n"
    )
}

fn generic_readme(config: &StarterConfig, checks: &[String], commands: &LocalCommands) -> String {
    let checks = checks
        .iter()
        .map(|item| format!("- `{item}`"))
        .collect::<Vec<_>>()
        .join("\n");
    let mut next_steps = vec![
        "Choose the primary language or framework for this repository.".to_string(),
        "Replace the placeholder CI and release commands with stack-specific validation and packaging.".to_string(),
        "Update `README.md` and `CONTRIBUTING.md` once the stack is decided.".to_string(),
    ];
    if config.generic.agent_context_files {
        next_steps.insert(
            0,
            "Read `docs/PROJECT_BRIEF.md`, `docs/ARCHITECTURE.md`, and `docs/DECISIONS.md` before implementing the stack.".to_string(),
        );
    }
    let next_steps = next_steps
        .iter()
        .enumerate()
        .map(|(idx, item)| format!("{}. {}", idx + 1, item))
        .collect::<Vec<_>>()
        .join("\n");
    let docs_section = if config.generic.agent_context_files {
        "- `docs/PROJECT_BRIEF.md`: product goals, users, and scope\n- `docs/ARCHITECTURE.md`: current structure and planned module boundaries\n- `docs/DECISIONS.md`: confirmed and pending technical decisions".to_string()
    } else {
        "- Add stack and product context files in `docs/` before handing the repo to an AI agent."
            .to_string()
    };
    let scripts_section = if config.generic.scripts_dir {
        "- `scripts/`: add repeatable automation entry points here once your toolchain is chosen"
    } else {
        "- Add a `scripts/` directory later if the project needs repeatable local automation"
    };
    let ai_section = ai_entrypoint_section(config);
    format!(
        "# {{{{project_name}}}}\n\n{{{{project_description}}}}\n\n## Template\n\n- Kind: `{{{{template_kind}}}}`\n- Runtime: `{{{{default_runtime}}}}`\n- Version: `{{{{project_version}}}}`\n\n## Git Workflow\n\n- Stable branch: `main`\n- Feature branches: `feat/<name>`\n- Bugfix branches: `fix/<name>`\n- Release blockers: `hotfix/<name>`\n- Release tags: `vX.Y.Z`\n\n## Agent Context\n\n{docs_section}\n{scripts_section}\n\n## Commands\n\n{}\n\n## Local Checks\n\n{checks}\n\n{ai_section}## Next Steps\n\n{next_steps}\n\n## Release\n\n- Source-first GitHub Release by default\n- Add stack-specific build or publish steps after choosing a language or framework\n",
        command_section(config, commands)
    )
}

fn ai_entrypoint_section(config: &StarterConfig) -> String {
    if !config.ai_tools.enabled {
        return String::new();
    }

    let mut files = Vec::new();
    if config.ai_tools.codex {
        files.push("- `AGENTS.md`: Codex entrypoint");
    }
    if config.ai_tools.claude_code {
        files.push("- `CLAUDE.md`: Claude Code memory");
    }
    if config.ai_tools.gemini_cli {
        files.push("- `GEMINI.md`: Gemini CLI context");
    }
    if config.ai_tools.tool_docs {
        files.push("- `docs/AI_TOOLS.md`: shared tool compatibility guide");
    }

    format!("## AI Tool Entry Points\n\n{}\n\n", files.join("\n"))
}

fn command_section(config: &StarterConfig, commands: &LocalCommands) -> String {
    let stack_note = match config.project.template {
        TemplateKind::GenericProject => format!("\nStack note: {}\n", commands.stack_specific),
        _ => String::new(),
    };
    format!(
        "- Install: `{}`\n- Start: `{}`\n- Validate: `{}`\n- Release prep: `{}`{}\n",
        commands.install, commands.start, commands.validate, commands.release_prep, stack_note
    )
}

fn contributing(config: &StarterConfig, checks: &[String]) -> String {
    let checks = checks
        .iter()
        .map(|item| format!("- `{item}`"))
        .collect::<Vec<_>>()
        .join("\n");
    let extra_care = match config.project.template {
        TemplateKind::GenericProject => {
            "- adding your first runtime or build toolchain\n- defining stack-specific CI or release behavior\n- changing generated repository conventions"
        }
        TemplateKind::PythonService => {
            "- image/tagging changes\n- deployment manifest edits\n- model or toolchain behavior updates"
        }
        TemplateKind::NodeWeb => {
            "- build pipeline changes\n- hosting/runtime configuration changes\n- public API contract changes"
        }
        TemplateKind::DesktopTauri => {
            "- bundling/signing changes\n- updater/package channel changes\n- native capability or filesystem scope changes"
        }
    };
    format!(
        "# Contributing\n\n## Branching\n\n- `main` is always intended to stay releasable\n- use `feat/<topic>` for new work\n- use `fix/<topic>` for bug fixes\n- use `hotfix/<topic>` for release blockers\n\n## Pull Requests\n\n- keep changes scoped\n- update docs when behavior changes\n- include validation steps in the PR body\n\n## Validation\n\nRun before opening a PR:\n{checks}\n\n## Change Types Requiring Extra Care\n\n{extra_care}\n\n## Release Flow\n\n1. Land changes on `main`\n2. Update versioned files together\n3. Create a release commit if needed\n4. Tag `vX.Y.Z`\n5. Push `main` and the tag\n"
    )
}

fn project_brief(config: &StarterConfig) -> String {
    match config.project.template {
        TemplateKind::GenericProject => generic_project_brief(),
        _ => format!(
            "# Project Brief\n\n## Goal\n\nDescribe the outcome this `{}` project should deliver.\n\n## Users\n\nList the primary users, operators, or internal teams.\n\n## In Scope\n\n- Core functionality for the selected template\n- Local validation and release flow generated by Revi\n\n## Out Of Scope\n\n- Major stack changes that are not yet documented here\n- Release automation beyond the generated defaults\n\n## Current State\n\nThis repository was generated from Revi's `{}` template with starter commands, repository conventions, and AI coding CLI compatibility files.\n",
            config.project.template.template_id(),
            config.project.template.template_id()
        ),
    }
}

fn architecture_doc(config: &StarterConfig) -> String {
    match config.project.template {
        TemplateKind::GenericProject => generic_architecture(),
        TemplateKind::PythonService => "# Architecture\n\n## Current Structure\n\n- `src/`: Python service code and entrypoint placeholder\n- `tests/`: smoke validation\n- `docs/`: project, architecture, and decision context\n\n## Planned Modules\n\nDescribe the service modules, adapters, and deployment boundaries here.\n\n## Open Questions\n\n- Which framework will own the runtime entrypoint?\n- Which deployment target should the release workflow publish to?\n"
            .to_string(),
        TemplateKind::NodeWeb => "# Architecture\n\n## Current Structure\n\n- `src/`: application entrypoint\n- `dist/`: generated static output placeholder\n- `tests/`: smoke validation\n- `docs/`: project, architecture, and decision context\n\n## Planned Modules\n\nDescribe routing, UI modules, APIs, and hosting boundaries here.\n\n## Open Questions\n\n- Which web framework or bundler should replace the starter scaffold?\n- Which hosting platform should the release flow target?\n"
            .to_string(),
        TemplateKind::DesktopTauri => "# Architecture\n\n## Current Structure\n\n- `src/`: frontend placeholder code\n- `dist/`: generated frontend output placeholder\n- `src-tauri/`: Rust desktop shell and Tauri config\n- `docs/`: project, architecture, and decision context\n\n## Planned Modules\n\nDescribe frontend, native, and packaging boundaries here.\n\n## Open Questions\n\n- Which frontend stack should back the Tauri shell?\n- Which signing or update channel requirements apply before release?\n"
            .to_string(),
    }
}

fn decisions_doc(config: &StarterConfig) -> String {
    let pending = match config.project.template {
        TemplateKind::GenericProject => {
            "- Primary language or framework\n- Test command and CI stack\n- Build and release packaging strategy"
        }
        TemplateKind::PythonService => {
            "- Service framework and runtime entrypoint\n- Container/deployment target\n- Required observability or background job integrations"
        }
        TemplateKind::NodeWeb => {
            "- Framework or bundler choice\n- Hosting target\n- Client-side versus server-side rendering split"
        }
        TemplateKind::DesktopTauri => {
            "- Frontend framework selection\n- Signing and notarization requirements\n- Update distribution strategy"
        }
    };
    format!(
        "# Decisions\n\n## Confirmed\n\n- Lightweight release flow on `main`\n- Tag-based releases using `vX.Y.Z`\n- AI coding CLI compatibility files generated by Revi\n\n## Pending\n\n{pending}\n"
    )
}

fn ai_tools_doc(config: &StarterConfig, commands: &LocalCommands) -> String {
    let mut tools = Vec::new();
    if config.ai_tools.codex {
        tools.push("- Codex reads `AGENTS.md`");
    }
    if config.ai_tools.claude_code {
        tools.push("- Claude Code reads `CLAUDE.md`");
    }
    if config.ai_tools.gemini_cli {
        tools.push("- Gemini CLI reads `GEMINI.md`");
    }
    let docs = if should_emit_project_context(config) {
        "- `README.md`\n- `docs/PROJECT_BRIEF.md`\n- `docs/ARCHITECTURE.md`\n- `docs/DECISIONS.md`"
            .to_string()
    } else {
        "- `README.md`\n- Add `docs/PROJECT_BRIEF.md`, `docs/ARCHITECTURE.md`, and `docs/DECISIONS.md` before handing the repository to an AI agent."
            .to_string()
    };
    let command_section = if config.ai_tools.command_helpers {
        format!(
            "## Command Helpers\n\n- Install: `{}`\n- Start: `{}`\n- Validate: `{}`\n- Release prep: `{}`\n\nStack-specific note: {}\n",
            commands.install, commands.start, commands.validate, commands.release_prep, commands.stack_specific
        )
    } else {
        String::new()
    };
    format!(
        "# AI Tools\n\n## Supported Tools\n\n{}\n\n## Recommended Read Order\n\n{}\n\n{}\n## Git And Release Rules\n\n- `main` stays releasable\n- Use `feat/<name>`, `fix/<name>`, and `hotfix/<name>` branches\n- Cut releases from `vX.Y.Z` tags\n",
        tools.join("\n"),
        docs,
        command_section
    )
}

fn agents_md(config: &StarterConfig, commands: &LocalCommands) -> String {
    tool_file(
        config,
        commands,
        "AGENTS.md",
        "Codex should use this file as the repo-level entrypoint before making changes.",
        "Codex",
    )
}

fn claude_md(config: &StarterConfig, commands: &LocalCommands) -> String {
    tool_file(
        config,
        commands,
        "CLAUDE.md",
        "Claude Code should treat this file as project memory and keep it aligned with the shared docs.",
        "Claude Code",
    )
}

fn gemini_md(config: &StarterConfig, commands: &LocalCommands) -> String {
    tool_file(
        config,
        commands,
        "GEMINI.md",
        "Gemini CLI should read this file first, then fall through to the shared repo docs.",
        "Gemini CLI",
    )
}

fn tool_file(
    config: &StarterConfig,
    commands: &LocalCommands,
    file_name: &str,
    intro: &str,
    tool_name: &str,
) -> String {
    let context = if should_emit_project_context(config) {
        "- `README.md`\n- `docs/PROJECT_BRIEF.md`\n- `docs/ARCHITECTURE.md`\n- `docs/DECISIONS.md`\n"
            .to_string()
    } else {
        "- `README.md`\n- Add `docs/PROJECT_BRIEF.md`, `docs/ARCHITECTURE.md`, and `docs/DECISIONS.md` before asking an AI tool to make structural decisions.\n".to_string()
    };
    let command_section = if config.ai_tools.command_helpers {
        format!(
            "## Commands\n\n- Install: `{}`\n- Start: `{}`\n- Validate: `{}`\n- Release prep: `{}`\n\nStack-specific note: {}\n\n",
            commands.install, commands.start, commands.validate, commands.release_prep, commands.stack_specific
        )
    } else {
        String::new()
    };
    format!(
        "# {file_name}\n\n{intro}\n\n## Tool\n\n- Consumer: `{tool_name}`\n- Template: `{}`\n- Runtime default: `{}`\n\n## Read First\n\n{context}\n## Repo Workflow\n\n- `main` must stay releasable\n- Use `feat/<name>`, `fix/<name>`, or `hotfix/<name>` for changes\n- Release from `vX.Y.Z` tags\n\n{command_section}## Guidance\n\n- Treat the shared docs as the source of truth instead of duplicating project context here.\n- Update docs when behavior, architecture, or release assumptions change.\n",
        config.project.template.template_id(),
        match config.project.template {
            TemplateKind::GenericProject => "custom",
            TemplateKind::PythonService => "python3.11",
            TemplateKind::NodeWeb => "node22",
            TemplateKind::DesktopTauri => "node22 + rust",
        }
    )
}

fn python_pyproject(config: &StarterConfig) -> String {
    format!(
        "[build-system]\nrequires = [\"setuptools>=68\", \"wheel\"]\nbuild-backend = \"setuptools.build_meta\"\n\n[project]\nname = \"{}\"\nversion = \"{}\"\ndescription = \"{}\"\nreadme = \"README.md\"\nrequires-python = \">=3.11\"\ndependencies = []\n\n[project.optional-dependencies]\ndev = [\"pytest>=8,<9\"]\n\n[tool.pytest.ini_options]\ntestpaths = [\"tests\"]\n",
        config.project.slug, config.project.version, config.project.description
    )
}

fn node_package_json(config: &StarterConfig) -> String {
    format!(
        "{{\n  \"name\": \"{}\",\n  \"version\": \"{}\",\n  \"private\": true,\n  \"type\": \"module\",\n  \"scripts\": {{\n    \"dev\": \"node src/index.js\",\n    \"build\": \"node -e \\\"const fs=require('node:fs'); fs.mkdirSync('dist', {{ recursive: true }}); fs.writeFileSync('dist/index.html', '<!doctype html><html><body><h1>{}</h1></body></html>');\\\"\",\n    \"test\": \"node --test tests/*.test.mjs\",\n    \"ci\": \"npm run build && npm run test\"\n  }}\n}}\n",
        config.project.slug, config.project.version, config.project.name
    )
}

fn tauri_package_json(config: &StarterConfig) -> String {
    format!(
        "{{\n  \"name\": \"{}\",\n  \"version\": \"{}\",\n  \"private\": true,\n  \"type\": \"module\",\n  \"scripts\": {{\n    \"dev\": \"node src/main.js\",\n    \"build\": \"node -e \\\"const fs=require('node:fs'); fs.mkdirSync('dist', {{ recursive: true }}); fs.writeFileSync('dist/index.html', '<!doctype html><html><body><h1>{}</h1></body></html>');\\\"\",\n    \"test\": \"node -e \\\"console.log('tauri smoke test placeholder')\\\"\",\n    \"ci\": \"npm run build && npm run test\",\n    \"tauri\": \"tauri\"\n  }},\n  \"devDependencies\": {{\n    \"@tauri-apps/cli\": \"^2.0.0\"\n  }}\n}}\n",
        config.project.slug, config.project.version, config.project.name
    )
}

fn tauri_cargo(config: &StarterConfig) -> String {
    format!(
        "[package]\nname = \"{}-desktop\"\nversion = \"{}\"\nedition = \"2024\"\nbuild = \"build.rs\"\n\n[dependencies]\ntauri = {{ version = \"2\", features = [] }}\n\n[build-dependencies]\ntauri-build = {{ version = \"2\", features = [] }}\n",
        config.project.slug, config.project.version
    )
}

fn tauri_conf(config: &StarterConfig) -> String {
    format!(
        "{{\n  \"$schema\": \"https://schema.tauri.app/config/2\",\n  \"productName\": \"{}\",\n  \"version\": \"{}\",\n  \"identifier\": \"com.example.{}\",\n  \"build\": {{\n    \"beforeDevCommand\": \"npm run dev\",\n    \"beforeBuildCommand\": \"npm run build\",\n    \"frontendDist\": \"../dist\"\n  }},\n  \"app\": {{\n    \"windows\": [{{ \"title\": \"{}\", \"width\": 1200, \"height\": 800 }}]\n  }},\n  \"bundle\": {{\n    \"active\": true,\n    \"targets\": \"all\"\n  }}\n}}\n",
        config.project.name, config.project.version, config.project.slug, config.project.name
    )
}

fn python_main() -> String {
    "def main() -> None:\n    print('hello from revi python-service template')\n\n\nif __name__ == '__main__':\n    main()\n".to_string()
}

fn python_test() -> String {
    "def test_smoke() -> None:\n    assert True\n".to_string()
}

fn generic_project_brief() -> String {
    "# Project Brief\n\n## Goal\n\nDescribe what this repository should become.\n\n## Users\n\nList the primary users or operators.\n\n## In Scope\n\n- \n\n## Out Of Scope\n\n- \n\n## Current State\n\nThis repository was generated from Revi's `generic-project` template and does not assume any language, framework, or deployment platform yet.\n"
        .to_string()
}

fn generic_architecture() -> String {
    "# Architecture\n\n## Current Structure\n\n- `src/`: implementation entry point once the stack is chosen\n- `docs/`: product, architecture, and decision context\n- `scripts/`: optional automation hooks if enabled\n\n## Planned Modules\n\nDescribe the major modules after the stack is selected.\n\n## Open Questions\n\n- \n"
        .to_string()
}

fn node_index() -> String {
    "console.log('hello from revi node-web template');\n".to_string()
}

fn node_test() -> String {
    "import test from 'node:test';\nimport assert from 'node:assert/strict';\n\ntest('smoke', () => {\n  assert.equal(1 + 1, 2);\n});\n".to_string()
}

fn node_dist_html(project_name: &str) -> String {
    format!("<!doctype html>\n<html>\n  <body>\n    <h1>{project_name}</h1>\n  </body>\n</html>\n")
}

fn tauri_frontend() -> String {
    "console.log('tauri frontend placeholder');\n".to_string()
}

fn tauri_build_rs() -> String {
    "fn main() {\n    tauri_build::build()\n}\n".to_string()
}

fn tauri_main() -> String {
    "#![cfg_attr(not(debug_assertions), windows_subsystem = \"windows\")]\n\nfn main() {\n    tauri::Builder::default()\n        .run(tauri::generate_context!())\n        .expect(\"error while running tauri application\");\n}\n"
        .to_string()
}

fn pr_template() -> String {
    "## Summary\n\n## Changes\n- \n\n## Validation\n- [ ] local checks passed\n- [ ] docs updated if behavior changed\n".to_string()
}

fn issue_bug_template() -> String {
    "---\nname: Bug report\nabout: Report a reproducible issue\nlabels: bug\n---\n\n## Summary\n\n## Steps to reproduce\n1.\n2.\n3.\n\n## Expected behavior\n\n## Actual behavior\n\n## Logs / screenshots\n".to_string()
}

fn issue_feature_template() -> String {
    "---\nname: Feature request\nabout: Suggest an improvement\nlabels: enhancement\n---\n\n## Problem\n\n## Proposal\n\n## Alternatives considered\n".to_string()
}

fn ci_workflow(config: &StarterConfig) -> String {
    match config.project.template {
        TemplateKind::GenericProject => "name: CI\n\non:\n  push:\n    branches: [main]\n  pull_request:\n\njobs:\n  validate:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@v5\n      - run: echo \"Add stack-specific validation commands to this workflow.\"\n".to_string(),
        TemplateKind::PythonService => "name: CI\n\non:\n  push:\n    branches: [main]\n  pull_request:\n\njobs:\n  test:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@v5\n      - uses: actions/setup-python@v6\n        with:\n          python-version: '3.11'\n      - run: python -m pip install -e \".[dev]\"\n      - run: pytest -q\n".to_string(),
        TemplateKind::NodeWeb => "name: CI\n\non:\n  push:\n    branches: [main]\n  pull_request:\n\njobs:\n  validate:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@v5\n      - uses: actions/setup-node@v4\n        with:\n          node-version: 22\n          cache: npm\n      - run: npm install\n      - run: npm run ci\n".to_string(),
        TemplateKind::DesktopTauri => "name: CI\n\non:\n  push:\n    branches: [main]\n  pull_request:\n\njobs:\n  validate:\n    runs-on: macos-latest\n    steps:\n      - uses: actions/checkout@v5\n      - uses: actions/setup-node@v4\n        with:\n          node-version: 22\n          cache: npm\n      - uses: dtolnay/rust-toolchain@stable\n      - run: npm install\n      - run: npm run ci\n".to_string(),
    }
}

fn release_workflow(config: &StarterConfig, release_notes: &[String]) -> String {
    let notes = release_notes
        .iter()
        .map(|item| format!("            - {item}"))
        .collect::<Vec<_>>()
        .join("\n");
    match config.project.template {
        TemplateKind::GenericProject => format!(
            "name: Release\n\non:\n  push:\n    tags: ['v*']\n\njobs:\n  release:\n    runs-on: ubuntu-latest\n    permissions:\n      contents: write\n    steps:\n      - uses: actions/checkout@v5\n      - env:\n          GH_TOKEN: ${{{{ secrets.GITHUB_TOKEN }}}}\n        run: |\n          cat > /tmp/release-notes.txt <<'EOF'\n          Release for ${{{{ github.ref_name }}}}\n\n{notes}\n\n          This generic template does not assume any language or framework.\n          Add stack-specific build artifacts after selecting your toolchain.\n          EOF\n          gh release create \"${{{{ github.ref_name }}}}\" --notes-file /tmp/release-notes.txt || gh release edit \"${{{{ github.ref_name }}}}\" --notes-file /tmp/release-notes.txt\n"
        ),
        TemplateKind::PythonService => format!(
            "name: Release\n\non:\n  push:\n    branches: [main]\n    tags: ['v*']\n\nenv:\n  IMAGE_NAME: ghcr.io/${{{{ github.repository_owner }}}}/{{{{project_slug}}}}\n\njobs:\n  container:\n    runs-on: ubuntu-latest\n    permissions:\n      contents: read\n      packages: write\n    steps:\n      - uses: actions/checkout@v5\n      - run: echo \"${{{{ secrets.GITHUB_TOKEN }}}}\" | docker login ghcr.io -u \"${{{{ github.actor }}}}\" --password-stdin\n      - run: |\n          docker build -t \"${{{{ env.IMAGE_NAME }}}}:latest\" -t \"${{{{ env.IMAGE_NAME }}}}:sha-${{{{ github.sha }}}}\" .\n      - run: |\n          docker push \"${{{{ env.IMAGE_NAME }}}}:latest\"\n          docker push \"${{{{ env.IMAGE_NAME }}}}:sha-${{{{ github.sha }}}}\"\n\n  release-notes:\n    if: startsWith(github.ref, 'refs/tags/v')\n    runs-on: ubuntu-latest\n    permissions:\n      contents: write\n    steps:\n      - env:\n          GH_TOKEN: ${{{{ secrets.GITHUB_TOKEN }}}}\n        run: |\n          cat > /tmp/release-notes.txt <<'EOF'\n          Release for ${{{{ github.ref_name }}}}\n\n{notes}\n          EOF\n          gh release create \"${{{{ github.ref_name }}}}\" --notes-file /tmp/release-notes.txt || gh release edit \"${{{{ github.ref_name }}}}\" --notes-file /tmp/release-notes.txt\n"
        ),
        TemplateKind::NodeWeb => format!(
            "name: Release\n\non:\n  push:\n    tags: ['v*']\n\njobs:\n  release:\n    runs-on: ubuntu-latest\n    permissions:\n      contents: write\n    steps:\n      - uses: actions/checkout@v5\n      - uses: actions/setup-node@v4\n        with:\n          node-version: 22\n          cache: npm\n      - run: npm install\n      - run: npm run ci\n      - env:\n          GH_TOKEN: ${{{{ secrets.GITHUB_TOKEN }}}}\n        run: |\n          cat > /tmp/release-notes.txt <<'EOF'\n          Release for ${{{{ github.ref_name }}}}\n\n{notes}\n\n          Hosting deployment should be wired for this project type.\n          EOF\n          gh release create \"${{{{ github.ref_name }}}}\" --notes-file /tmp/release-notes.txt || gh release edit \"${{{{ github.ref_name }}}}\" --notes-file /tmp/release-notes.txt\n"
        ),
        TemplateKind::DesktopTauri => format!(
            "name: Release\n\non:\n  push:\n    tags: ['v*']\n\npermissions:\n  contents: write\n\njobs:\n  publish-tauri:\n    strategy:\n      fail-fast: false\n      matrix:\n        include:\n          - platform: macos-latest\n            args: --target aarch64-apple-darwin\n          - platform: macos-latest\n            args: --target x86_64-apple-darwin\n          - platform: ubuntu-22.04\n            args: ''\n          - platform: windows-latest\n            args: ''\n    runs-on: ${{{{ matrix.platform }}}}\n    steps:\n      - uses: actions/checkout@v5\n      - uses: actions/setup-node@v4\n        with:\n          node-version: 22\n          cache: npm\n      - uses: dtolnay/rust-toolchain@stable\n      - run: npm install\n      - uses: tauri-apps/tauri-action@v0\n        env:\n          GITHUB_TOKEN: ${{{{ secrets.GITHUB_TOKEN }}}}\n        with:\n          tagName: ${{{{ github.ref_name }}}}\n          releaseName: {{{{project_name}}}} ${{{{ github.ref_name }}}}\n          releaseBody: |\n            Release for ${{{{ github.ref_name }}}}\n{notes}\n          releaseDraft: false\n          prerelease: false\n          projectPath: .\n          args: ${{{{ matrix.args }}}}\n"
        ),
    }
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use crate::config::{
        AiToolsConfig, BootstrapConfig, BranchStrategy, GenericTemplateConfig, GithubConfig,
        ProjectConfig, ReleaseChannel, ReleaseConfig, StarterConfig, TemplateKind,
        WorkflowConfig,
    };

    use super::scaffold;

    fn test_config(
        path: std::path::PathBuf,
        template: TemplateKind,
        github: bool,
    ) -> StarterConfig {
        StarterConfig {
            schema_version: 1,
            project: ProjectConfig {
                name: "Demo".to_string(),
                slug: "demo".to_string(),
                template,
                path,
                description: "Demo project".to_string(),
                version: "0.1.0".to_string(),
            },
            workflow: WorkflowConfig {
                branch_strategy: BranchStrategy::LightweightRelease,
                release: ReleaseConfig {
                    channel: ReleaseChannel::GithubReleaseAndRegistry,
                    registry: true,
                    github_release: true,
                },
            },
            bootstrap: BootstrapConfig {
                init_git: false,
                initial_commit: false,
            },
            github: GithubConfig {
                enabled: github,
                create_repo: false,
                owner: None,
                repo: Some("demo".to_string()),
                push_after_create: false,
                codeowners: github,
            },
            ai_tools: AiToolsConfig::default(),
            generic: GenericTemplateConfig::default(),
        }
    }

    #[test]
    fn scaffolds_python_template() {
        let temp = tempdir().expect("tempdir");
        let target = temp.path().join("python-demo");
        let config = test_config(target.clone(), TemplateKind::PythonService, true);
        scaffold(&config).expect("scaffold");
        assert!(target.join("pyproject.toml").exists());
        assert!(target.join("src/main.py").exists());
        assert!(target.join(".github/workflows/ci.yml").exists());
        assert!(target.join("AGENTS.md").exists());
        assert!(target.join("CLAUDE.md").exists());
        assert!(target.join("GEMINI.md").exists());
        assert!(target.join("docs/AI_TOOLS.md").exists());
        assert!(target.join("docs/PROJECT_BRIEF.md").exists());
        assert!(target.join("revi.toml").exists());
    }

    #[test]
    fn scaffolds_node_template_without_github_files() {
        let temp = tempdir().expect("tempdir");
        let target = temp.path().join("node-demo");
        let config = test_config(target.clone(), TemplateKind::NodeWeb, false);
        scaffold(&config).expect("scaffold");
        assert!(target.join("package.json").exists());
        assert!(!target.join(".github").exists());
        assert!(target.join("AGENTS.md").exists());
        assert!(target.join("docs/AI_TOOLS.md").exists());
    }

    #[test]
    fn scaffolds_tauri_template() {
        let temp = tempdir().expect("tempdir");
        let target = temp.path().join("tauri-demo");
        let config = test_config(target.clone(), TemplateKind::DesktopTauri, true);
        scaffold(&config).expect("scaffold");
        assert!(target.join("src-tauri/Cargo.toml").exists());
        assert!(target.join(".github/workflows/release.yml").exists());
        assert!(target.join("GEMINI.md").exists());
    }

    #[test]
    fn scaffolds_generic_template() {
        let temp = tempdir().expect("tempdir");
        let target = temp.path().join("generic-demo");
        let config = test_config(target.clone(), TemplateKind::GenericProject, true);
        scaffold(&config).expect("scaffold");
        assert!(target.join("src/.gitkeep").exists());
        assert!(target.join("docs/.gitkeep").exists());
        assert!(target.join("docs/PROJECT_BRIEF.md").exists());
        assert!(target.join("scripts/.gitkeep").exists());
        assert!(target.join(".github/workflows/ci.yml").exists());
        assert!(target.join("AGENTS.md").exists());
        assert!(target.join("docs/AI_TOOLS.md").exists());
        assert!(!target.join("package.json").exists());
        assert!(!target.join("pyproject.toml").exists());
    }

    #[test]
    fn scaffolds_generic_template_with_disabled_optional_files() {
        let temp = tempdir().expect("tempdir");
        let target = temp.path().join("generic-minimal");
        let mut config = test_config(target.clone(), TemplateKind::GenericProject, true);
        config.generic = GenericTemplateConfig {
            agent_context_files: false,
            scripts_dir: false,
            placeholder_workflows: false,
            docs_expanded: false,
        };
        scaffold(&config).expect("scaffold");
        assert!(!target.join("docs/PROJECT_BRIEF.md").exists());
        assert!(!target.join("scripts").exists());
        assert!(!target.join(".github/workflows/ci.yml").exists());
        assert!(target.join(".github/pull_request_template.md").exists());
        assert!(target.join("AGENTS.md").exists());
    }

    #[test]
    fn omits_ai_tool_files_when_disabled() {
        let temp = tempdir().expect("tempdir");
        let target = temp.path().join("node-no-ai");
        let mut config = test_config(target.clone(), TemplateKind::NodeWeb, false);
        config.ai_tools.enabled = false;
        scaffold(&config).expect("scaffold");
        assert!(!target.join("AGENTS.md").exists());
        assert!(!target.join("CLAUDE.md").exists());
        assert!(!target.join("GEMINI.md").exists());
        assert!(!target.join("docs/AI_TOOLS.md").exists());
    }

    #[test]
    fn omits_single_tool_file_when_tool_disabled() {
        let temp = tempdir().expect("tempdir");
        let target = temp.path().join("python-no-gemini");
        let mut config = test_config(target.clone(), TemplateKind::PythonService, false);
        config.ai_tools.gemini_cli = false;
        scaffold(&config).expect("scaffold");
        assert!(target.join("AGENTS.md").exists());
        assert!(target.join("CLAUDE.md").exists());
        assert!(!target.join("GEMINI.md").exists());
    }
}
