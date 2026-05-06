use anyhow::{Context, Result};

use crate::paths;
use crate::registry::Registry;
use crate::ui;

pub fn run() -> Result<()> {
    let registry_path = paths::registry_path()?;
    let registry = Registry::load(&registry_path)
        .with_context(|| format!("loading registry from {}", registry_path.display()))?;

    if registry.projects().is_empty() {
        ui::print_info("no projects registered yet — try `pj register`");
        return Ok(());
    }

    let sorted = registry.sorted_by_recency();
    println!("{}", ui::render_projects_table(&sorted));
    Ok(())
}
