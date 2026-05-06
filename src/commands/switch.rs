//! `pj __shell` — emit the script the shell wrapper will eval to switch
//! into a project. Stdout is the script; stderr is for human messages.
//!
//! Two paths:
//!   - name given: resolve via registry, emit script.
//!   - name omitted: launch interactive picker, then emit script.

use std::fmt;

use anyhow::{Context, Result};
use inquire::Select;
use owo_colors::OwoColorize;

use crate::config::ProjectConfig;
use crate::paths;
use crate::registry::{Project, Registry};
use crate::shell::{build_switch_script, ShellKind};
use crate::ui;

pub fn run(shell: ShellKind, query: Option<&str>) -> Result<()> {
    let registry_path = paths::registry_path()?;
    let mut registry = Registry::load(&registry_path)
        .with_context(|| format!("loading registry from {}", registry_path.display()))?;

    let target_name = match query {
        Some(q) => registry.find(q)?.name.clone(),
        None => pick_interactive(&registry)?,
    };

    let project = registry.find(&target_name)?.clone();

    if !project.path.exists() {
        ui::print_warn(&format!(
            "registered path no longer exists: {}",
            project.path.display()
        ));
    }

    let config = match ProjectConfig::load(&project.path) {
        Ok(cfg) => cfg,
        Err(e) => {
            ui::print_warn(&format!("ignoring project.toml: {e}"));
            None
        }
    };

    registry.touch(&project.name)?;
    registry.save(&registry_path)?;

    let script = build_switch_script(shell, &project.path, config.as_ref());
    print!("{script}");
    Ok(())
}

fn pick_interactive(registry: &Registry) -> Result<String> {
    let projects = registry.sorted_by_recency();
    if projects.is_empty() {
        return Err(anyhow::anyhow!(
            "no projects registered — run `pj register` first"
        ));
    }

    let options: Vec<Pick> = projects.iter().map(|p| Pick { project: p }).collect();
    let chosen = Select::new("Switch to:", options)
        .with_help_message("type to filter · ↑↓ to move · enter to select")
        .with_page_size(15)
        .prompt()
        .context("interactive selection cancelled")?;
    Ok(chosen.project.name.clone())
}

struct Pick<'a> {
    project: &'a Project,
}

impl fmt::Display for Pick<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:<28} {}",
            self.project.name.bold(),
            self.project.path.display().to_string().dimmed()
        )
    }
}
