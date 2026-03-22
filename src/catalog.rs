use anyhow::{Context, Result, bail};
use serde::Deserialize;

use crate::config::TemplateKind;

#[derive(Clone, Debug, Deserialize)]
pub struct TemplateManifest {
    pub id: String,
    pub kind: TemplateKind,
    pub display_name: String,
    pub description: String,
    pub default_runtime: String,
    pub checks: Vec<String>,
    pub release_notes: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct TemplateSpec {
    pub manifest: TemplateManifest,
}

pub fn load_templates() -> Result<Vec<TemplateSpec>> {
    let mut templates = Vec::new();
    for (id, raw) in [
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
        let manifest = toml::from_str::<TemplateManifest>(raw)
            .with_context(|| format!("invalid manifest {id}"))?;
        templates.push(TemplateSpec { manifest });
    }
    templates.sort_by(|a, b| a.manifest.id.cmp(&b.manifest.id));
    Ok(templates)
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
        let manifest = spec.manifest;
        out.push_str(&format!(
            "{}\n  kind: {:?}\n  runtime: {}\n  {}\n\n",
            manifest.id, manifest.kind, manifest.default_runtime, manifest.description
        ));
    }
    Ok(out.trim_end().to_string())
}
