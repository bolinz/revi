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
    let readme_content = match config.project.template {
        TemplateKind::GenericProject => generic_readme(config, checks),
        _ => readme(config, checks),
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
            if config.generic.agent_context_files {
                files.insert("docs/PROJECT_BRIEF.md".to_string(), generic_project_brief());
                files.insert("docs/ARCHITECTURE.md".to_string(), generic_architecture());
                files.insert("docs/DECISIONS.md".to_string(), generic_decisions());
            }
        }
        TemplateKind::PythonService => {
            files.insert(
                "pyproject.toml".to_string(),
                render(&python_pyproject(config), ctx),
            );
            files.insert("src/__init__.py".to_string(), String::new());
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

fn render(template: &str, ctx: &BTreeMap<&'static str, String>) -> String {
    let mut output = template.to_string();
    for (key, value) in ctx {
        output = output.replace(&format!("{{{{{key}}}}}"), value);
    }
    output
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

fn readme(config: &StarterConfig, checks: &[String]) -> String {
    let checks = checks
        .iter()
        .map(|item| format!("- `{item}`"))
        .collect::<Vec<_>>()
        .join("\n");
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
    format!(
        "# {{{{project_name}}}}\n\n{{{{project_description}}}}\n\n## Template\n\n- Kind: `{{{{template_kind}}}}`\n- Runtime: `{{{{default_runtime}}}}`\n- Version: `{{{{project_version}}}}`\n\n## Git Workflow\n\n- Stable branch: `main`\n- Feature branches: `feat/<name>`\n- Bugfix branches: `fix/<name>`\n- Release blockers: `hotfix/<name>`\n- Release tags: `vX.Y.Z`\n\n## Local Checks\n\n{checks}\n\n## Release\n\n{release_targets}\n\nRelease checklist:\n1. Merge work back to `main`\n2. Update version and `CHANGELOG.md`\n3. Tag `{{{{release_tag_example}}}}`\n4. Push `main` and the tag\n"
    )
}

fn generic_readme(config: &StarterConfig, checks: &[String]) -> String {
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
    format!(
        "# {{{{project_name}}}}\n\n{{{{project_description}}}}\n\n## Template\n\n- Kind: `{{{{template_kind}}}}`\n- Runtime: `{{{{default_runtime}}}}`\n- Version: `{{{{project_version}}}}`\n\n## Git Workflow\n\n- Stable branch: `main`\n- Feature branches: `feat/<name>`\n- Bugfix branches: `fix/<name>`\n- Release blockers: `hotfix/<name>`\n- Release tags: `vX.Y.Z`\n\n## Agent Context\n\n{docs_section}\n{scripts_section}\n\n## Local Checks\n\n{checks}\n\n## Next Steps\n\n{next_steps}\n\n## Release\n\n- Source-first GitHub Release by default\n- Add stack-specific build or publish steps after choosing a language or framework\n"
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

fn generic_decisions() -> String {
    "# Decisions\n\n## Confirmed\n\n- Lightweight release flow on `main`\n- Tag-based releases using `vX.Y.Z`\n\n## Pending\n\n- Primary language or framework\n- Test command and CI stack\n- Build and release packaging strategy\n"
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
        BootstrapConfig, BranchStrategy, GenericTemplateConfig, GithubConfig, ProjectConfig,
        ReleaseChannel, ReleaseConfig, StarterConfig, TemplateKind, WorkflowConfig,
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
        assert!(target.join(".github/workflows/ci.yml").exists());
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
    }

    #[test]
    fn scaffolds_tauri_template() {
        let temp = tempdir().expect("tempdir");
        let target = temp.path().join("tauri-demo");
        let config = test_config(target.clone(), TemplateKind::DesktopTauri, true);
        scaffold(&config).expect("scaffold");
        assert!(target.join("src-tauri/Cargo.toml").exists());
        assert!(target.join(".github/workflows/release.yml").exists());
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
    }
}
