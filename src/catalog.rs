use anyhow::{Context, Result, bail};

use crate::config::TemplateKind;
use crate::manifest::TemplateSpec as TemplateSpecV2;

/// Template manifest (normalized V1/V2 format for internal use)
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

/// Template spec
#[derive(Clone, Debug)]
pub struct TemplateSpec {
    pub manifest: TemplateManifest,
    pub is_v2: bool,
    /// V2 manifest with file definitions (None for V1 templates)
    pub v2_spec: Option<TemplateSpecV2>,
}

/// Load templates from V2 manifests on filesystem
pub fn load_templates() -> Result<Vec<TemplateSpec>> {
    let v2_templates = load_templates_v2_from_fs()?;
    let mut templates = Vec::new();

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
            v2_spec: Some(spec),
        });
    }

    templates.sort_by(|a, b| a.manifest.id.cmp(&b.manifest.id));
    Ok(templates)
}

/// Load V2 manifests from filesystem
fn load_templates_v2_from_fs() -> Result<Vec<TemplateSpecV2>> {
    use crate::manifest::load_templates_from_dir;
    load_templates_from_dir()
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
