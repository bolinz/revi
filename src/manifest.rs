use std::collections::BTreeMap;

use anyhow::{Context, Result};
use serde::Deserialize;

/// Enhanced template manifest with file definitions
#[derive(Clone, Debug, Deserialize)]
pub struct TemplateManifestV2 {
    pub template: TemplateInfo,
    #[serde(default)]
    pub runtime: RuntimeInfo,
    #[serde(default)]
    pub commands: CommandsInfo,
    #[serde(default)]
    pub files: FilesInfo,
    /// Legacy fields for backward compatibility
    #[serde(default)]
    pub checks: Vec<String>,
    #[serde(default)]
    pub release_notes: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TemplateInfo {
    pub id: String,
    pub kind: String,
    pub display_name: String,
    pub description: String,
    #[serde(default = "default_version")]
    pub version: String,
}

fn default_version() -> String {
    "1.0.0".to_string()
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct RuntimeInfo {
    #[serde(default = "default_runtime")]
    pub default: String,
}

fn default_runtime() -> String {
    "custom".to_string()
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct CommandsInfo {
    #[serde(default)]
    pub install: String,
    #[serde(default)]
    pub start: String,
    #[serde(default)]
    pub validate: String,
    #[serde(default)]
    pub release_prep: String,
    #[serde(default)]
    pub stack_specific: String,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct FilesInfo {
    #[serde(default, rename = "static")]
    pub static_files: Vec<String>,
    #[serde(default)]
    pub conditional: Vec<ConditionalFile>,
    #[serde(default)]
    pub inline: Vec<InlineFile>,
    #[serde(default)]
    pub external: Vec<ExternalFile>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ConditionalFile {
    pub path: String,
    #[serde(default)]
    pub when: WhenCondition,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct WhenCondition {
    #[serde(default)]
    pub generic: GenericConditions,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct GenericConditions {
    #[serde(default)]
    pub scripts_dir: bool,
    #[serde(default)]
    pub docs_expanded: bool,
    #[serde(default)]
    pub agent_context_files: bool,
    #[serde(default)]
    pub placeholder_workflows: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub struct InlineFile {
    pub path: String,
    pub content: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ExternalFile {
    pub path: String,
    pub template: String,
}

/// Parse a manifest.toml string into TemplateManifestV2
pub fn parse_manifest(content: &str) -> Result<TemplateManifestV2> {
    toml::from_str(content).context("failed to parse manifest.toml")
}

/// Load all templates from the templates directory
pub fn load_templates_from_dir() -> Result<Vec<TemplateSpec>> {
    let templates_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("templates");

    let mut templates = Vec::new();

    for entry in std::fs::read_dir(&templates_dir)? {
        let entry = entry?;
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        let manifest_path = path.join("manifest.toml");
        if !manifest_path.exists() {
            continue;
        }

        let content = match std::fs::read_to_string(&manifest_path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Warning: failed to read {}: {}", manifest_path.display(), e);
                continue;
            }
        };

        // Try to parse as V2 manifest, skip if it fails (might be V1 format)
        let manifest = match toml::from_str::<TemplateManifestV2>(&content) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("Warning: skipping {} (not V2 manifest): {}", path.display(), e);
                continue;
            }
        };

        templates.push(TemplateSpec {
            manifest,
            root_path: path,
        });
    }

    templates.sort_by(|a, b| a.manifest.template.id.cmp(&b.manifest.template.id));
    Ok(templates)
}

#[derive(Clone, Debug)]
pub struct TemplateSpec {
    pub manifest: TemplateManifestV2,
    pub root_path: std::path::PathBuf,
}

impl TemplateSpec {
    /// Get a static file path from the template directory
    pub fn get_static_file(&self, relative_path: &str) -> Option<std::path::PathBuf> {
        let path = self.root_path.join(relative_path);
        if path.exists() && path.is_file() {
            Some(path)
        } else {
            None
        }
    }

    /// Get an external template file path
    pub fn get_template_file(&self, template_name: &str) -> Option<std::path::PathBuf> {
        // Template files live in a "templates/" subdirectory
        let path = self.root_path.join("templates").join(template_name);
        if path.exists() && path.is_file() {
            Some(path)
        } else {
            None
        }
    }
}

/// Render template variables into a string
pub fn render_template(content: &str, ctx: &BTreeMap<&str, String>) -> String {
    let mut output = content.to_string();
    for (key, value) in ctx {
        let placeholder = format!("{{{{{key}}}}}");
        output = output.replace(&placeholder, value);
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_manifest() {
        let content = r#"
[template]
id = "test"
kind = "test"
display_name = "Test Template"
description = "A test template"
"#;
        let manifest = parse_manifest(content).unwrap();
        assert_eq!(manifest.template.id, "test");
        assert_eq!(manifest.template.kind, "test");
    }

    #[test]
    fn test_parse_full_manifest() {
        let content = r#"
[template]
id = "generic-project"
kind = "generic-project"
display_name = "Generic Project"
description = "A generic project"
version = "2.0.0"

[runtime]
default = "custom"

[commands]
install = "npm install"
start = "npm run dev"
validate = "npm test"
release_prep = "npm run build"

[files]
static = [".gitignore", "src/.gitkeep"]

[[files.conditional]]
path = "scripts/.gitkeep"
when.generic.scripts_dir = true
"#;
        let manifest = parse_manifest(content).unwrap();
        assert_eq!(manifest.template.version, "2.0.0");
        assert_eq!(manifest.runtime.default, "custom");
        assert_eq!(manifest.commands.install, "npm install");
        assert_eq!(manifest.files.static_files.len(), 2);
        assert_eq!(manifest.files.conditional.len(), 1);
    }

    #[test]
    fn test_render_template() {
        let ctx: BTreeMap<&str, String> = [
            ("project_name", "My Project".to_string()),
            ("version", "1.0.0".to_string()),
        ].into();

        let result = render_template("{{project_name}} v{{version}}", &ctx);
        assert_eq!(result, "My Project v1.0.0");
    }
}
