use anyhow::{Context, Result, bail};
use serde::Deserialize;

use crate::config::TemplateKind;
use crate::manifest::TemplateSpec as TemplateSpecV2;

/// Legacy template manifest for backward compatibility
#[derive(Clone, Debug)]
pub struct TemplateManifest {
    pub id: String,
    pub kind: TemplateKind,
    pub display_name: String,
    pub description: String,
    pub default_runtime: String,
    pub checks: Vec<String>,
    pub release_notes: Vec<String>,
}

/// Template spec wrapping the new V2 manifest
#[derive(Clone, Debug)]
pub struct TemplateSpec {
    pub manifest: TemplateManifest,
    pub is_v2: bool,
}

impl TemplateSpec {
    /// Get the V2 manifest if available
    pub fn as_v2(&self) -> Option<&TemplateSpecV2> {
        // This is a placeholder - in practice we'd store the V2 separately
        None
    }
}

/// Load templates - tries V2 format first from filesystem, falls back to embedded V1
pub fn load_templates() -> Result<Vec<TemplateSpec>> {
    let mut templates = Vec::new();

    // Try to load V2 manifests from filesystem
    if let Ok(v2_templates) = load_templates_v2_from_fs() {
        for spec in v2_templates {
            let manifest = TemplateManifest {
                id: spec.manifest.template.id.clone(),
                kind: TemplateKind::from_template_id(&spec.manifest.template.id),
                display_name: spec.manifest.template.display_name.clone(),
                description: spec.manifest.template.description.clone(),
                default_runtime: spec.manifest.runtime.default.clone(),
                checks: spec.manifest.checks.clone(),
                release_notes: spec.manifest.release_notes.clone(),
            };
            templates.push(TemplateSpec {
                manifest,
                is_v2: true,
            });
        }
    }

    // Add embedded V1 manifests for templates not in filesystem
    for (id, raw) in [
        (
            "generic-project",
            include_str!("../templates/generic-project/manifest.toml"),
        ),
        (
            "python-service",
            include_str!("../templates/python-service/manifest.toml"),
        ),
        (
            "node-web",
            include_str!("../templates/node-web/manifest.toml"),
        ),
        (
            "desktop-tauri",
            include_str!("../templates/desktop-tauri/manifest.toml"),
        ),
    ] {
        let manifest: LegacyManifestV1 = toml::from_str(raw)
            .with_context(|| format!("invalid manifest {id}"))?;
        // Only add if not already present from V2 loading
        if !templates.iter().any(|t| t.manifest.id == manifest.id) {
            templates.push(TemplateSpec {
                manifest: TemplateManifest {
                    id: manifest.id,
                    kind: manifest.kind,
                    display_name: manifest.display_name,
                    description: manifest.description,
                    default_runtime: manifest.default_runtime,
                    checks: manifest.checks,
                    release_notes: manifest.release_notes,
                },
                is_v2: false,
            });
        }
    }

    templates.sort_by(|a, b| a.manifest.id.cmp(&b.manifest.id));
    Ok(templates)
}

/// Load V2 manifests from filesystem
fn load_templates_v2_from_fs() -> Result<Vec<TemplateSpecV2>> {
    use crate::manifest::load_templates_from_dir;
    load_templates_from_dir()
}

/// Legacy V1 manifest structure for parsing old format
#[derive(Debug, Deserialize)]
struct LegacyManifestV1 {
    id: String,
    kind: TemplateKind,
    display_name: String,
    description: String,
    default_runtime: String,
    checks: Vec<String>,
    release_notes: Vec<String>,
}

impl TemplateKind {
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

pub fn get_template(kind: TemplateKind) -> Result<TemplateSpec> {
    let id = kind.template_id();
    load_templates()?
        .into_iter()
        .find(|spec| spec.manifest.id == id)
        .with_context(|| format!("template {id} not found"))
}

pub fn format_template_list() -> Result<String> {
    let templates = load_templates()?;
    if templates.is_empty() {
        bail!("no templates available");
    }
    let mut out = String::new();
    for spec in templates {
        let manifest = &spec.manifest;
        let version_tag = if spec.is_v2 { " (v2)" } else { "" };
        out.push_str(&format!(
            "{}{}\n  kind: {:?}\n  runtime: {}\n  {}\n\n",
            manifest.id,
            version_tag,
            manifest.kind,
            manifest.default_runtime,
            manifest.description
        ));
    }
    Ok(out.trim_end().to_string())
}
