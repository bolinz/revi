use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct StarterConfig {
    pub schema_version: u32,
    pub project: ProjectConfig,
    pub workflow: WorkflowConfig,
    pub bootstrap: BootstrapConfig,
    pub github: GithubConfig,
    #[serde(default)]
    pub ai_tools: AiToolsConfig,
    #[serde(default, skip_serializing_if = "GenericTemplateConfig::is_default")]
    pub generic: GenericTemplateConfig,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectConfig {
    pub name: String,
    pub slug: String,
    pub template: TemplateKind,
    pub path: PathBuf,
    pub description: String,
    pub version: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkflowConfig {
    pub branch_strategy: BranchStrategy,
    pub release: ReleaseConfig,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReleaseConfig {
    pub channel: ReleaseChannel,
    pub registry: bool,
    pub github_release: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct BootstrapConfig {
    pub init_git: bool,
    pub initial_commit: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct GithubConfig {
    pub enabled: bool,
    pub create_repo: bool,
    pub owner: Option<String>,
    pub repo: Option<String>,
    pub push_after_create: bool,
    pub codeowners: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct AiToolsConfig {
    pub enabled: bool,
    pub codex: bool,
    pub claude_code: bool,
    pub gemini_cli: bool,
    pub tool_docs: bool,
    pub command_helpers: bool,
    pub skills: bool,
    pub agents: bool,
    pub use_ai_api: bool,
    #[serde(default = "default_provider")]
    pub ai_provider: String,
}

fn default_provider() -> String {
    "minimax".to_string()
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct GenericTemplateConfig {
    pub agent_context_files: bool,
    pub scripts_dir: bool,
    pub placeholder_workflows: bool,
    pub docs_expanded: bool,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum TemplateKind {
    GenericProject,
    PythonService,
    NodeWeb,
    DesktopTauri,
    RustWeb,
    GoApi,
    KotlinCli,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum BranchStrategy {
    LightweightRelease,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ReleaseChannel {
    GithubReleaseAndRegistry,
}

impl StarterConfig {
    pub fn load(path: &Path) -> Result<Self> {
        let raw = std::fs::read_to_string(path)
            .with_context(|| format!("failed to read config {}", path.display()))?;
        let config = toml::from_str(&raw)
            .with_context(|| format!("failed to parse TOML {}", path.display()))?;
        Ok(config)
    }

    pub fn save_to(&self, path: &Path) -> Result<()> {
        let raw = toml::to_string_pretty(self).context("failed to encode config")?;
        write_if_changed(path, &raw)
    }
}

impl Default for GenericTemplateConfig {
    fn default() -> Self {
        Self {
            agent_context_files: true,
            scripts_dir: true,
            placeholder_workflows: true,
            docs_expanded: true,
        }
    }
}

impl Default for AiToolsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            codex: true,
            claude_code: true,
            gemini_cli: true,
            tool_docs: true,
            command_helpers: true,
            skills: true,
            agents: false,
            use_ai_api: false,
            ai_provider: "minimax".to_string(),
        }
    }
}

impl AiToolsConfig {
    pub fn is_default(config: &Self) -> bool {
        config == &Self::default()
    }
}

impl GenericTemplateConfig {
    pub fn is_default(config: &Self) -> bool {
        config == &Self::default()
    }
}

pub fn write_if_changed(path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    match std::fs::read_to_string(path) {
        Ok(existing) if existing == content => return Ok(()),
        Ok(_) | Err(_) => {}
    }
    std::fs::write(path, content).with_context(|| format!("failed to write {}", path.display()))
}

impl TemplateKind {
    pub fn template_id(self) -> &'static str {
        match self {
            Self::GenericProject => "generic-project",
            Self::PythonService => "python-service",
            Self::NodeWeb => "node-web",
            Self::DesktopTauri => "desktop-tauri",
            Self::RustWeb => "rust-web",
            Self::GoApi => "go-api",
            Self::KotlinCli => "kotlin-cli",
        }
    }

    /// Convert template ID string to TemplateKind
    pub fn from_template_id(id: &str) -> TemplateKind {
        match id {
            "generic-project" => TemplateKind::GenericProject,
            "python-service" => TemplateKind::PythonService,
            "node-web" => TemplateKind::NodeWeb,
            "desktop-tauri" => TemplateKind::DesktopTauri,
            "rust-web" => TemplateKind::RustWeb,
            "go-api" => TemplateKind::GoApi,
            "kotlin-cli" => TemplateKind::KotlinCli,
            _ => TemplateKind::GenericProject,
        }
    }
}
