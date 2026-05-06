use anyhow::{Context, Result};

use crate::paths;
use crate::registry::Registry;
use crate::ui;

pub fn run(name: &str) -> Result<()> {
    let registry_path = paths::registry_path()?;
    let mut registry = Registry::load(&registry_path)
        .with_context(|| format!("loading registry from {}", registry_path.display()))?;

    let removed = registry.remove(name)?;
    registry.save(&registry_path)?;

    ui::print_success(&format!(
        "removed '{}' ({})",
        removed.name,
        removed.path.display()
    ));
    Ok(())
}
