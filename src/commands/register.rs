use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};

use crate::paths;
use crate::registry::{Project, Registry};
use crate::ui;

pub fn run(path: Option<PathBuf>, name_override: Option<String>) -> Result<()> {
    let raw_path = match path {
        Some(p) => p,
        None => std::env::current_dir().context("reading current directory")?,
    };

    let canonical = std::fs::canonicalize(&raw_path)
        .with_context(|| format!("resolving {}", raw_path.display()))?;

    if !canonical.is_dir() {
        return Err(anyhow!(
            "{} is not a directory",
            canonical.display()
        ));
    }

    let name = match name_override {
        Some(n) => n,
        None => infer_name(&canonical)?,
    };
    validate_name(&name)?;

    let registry_path = paths::registry_path()?;
    let mut registry = Registry::load(&registry_path)
        .with_context(|| format!("loading registry from {}", registry_path.display()))?;

    registry.add(Project::new(name.clone(), canonical.clone()))?;
    registry.save(&registry_path)?;

    ui::print_success(&format!(
        "registered '{name}' → {}",
        canonical.display()
    ));
    Ok(())
}

fn infer_name(path: &std::path::Path) -> Result<String> {
    path.file_name()
        .and_then(|s| s.to_str())
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow!("could not infer a name from {}", path.display()))
}

fn validate_name(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(anyhow!("project name cannot be empty"));
    }
    if name.contains(char::is_whitespace) {
        return Err(anyhow!("project name cannot contain whitespace"));
    }
    Ok(())
}
