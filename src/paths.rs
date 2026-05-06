//! Resolves the on-disk location of pier's configuration files.
//!
//! Resolution order:
//! 1. `$PIER_CONFIG_DIR` (override, mainly for tests)
//! 2. `$XDG_CONFIG_HOME/pier`
//! 3. `~/.config/pier`
//!
//! Note: this intentionally diverges from `directories::ProjectDirs` on macOS,
//! which would return `~/Library/Application Support/pier`. CLI-style XDG
//! paths are far more discoverable for the target audience.

use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};

const ENV_OVERRIDE: &str = "PIER_CONFIG_DIR";
const XDG_VAR: &str = "XDG_CONFIG_HOME";
const APP_DIR: &str = "pier";
const REGISTRY_FILE: &str = "projects.toml";

pub fn config_dir() -> Result<PathBuf> {
    if let Ok(p) = std::env::var(ENV_OVERRIDE) {
        if !p.is_empty() {
            return Ok(PathBuf::from(p));
        }
    }
    if let Ok(p) = std::env::var(XDG_VAR) {
        if !p.is_empty() {
            return Ok(PathBuf::from(p).join(APP_DIR));
        }
    }
    let home = directories::BaseDirs::new()
        .ok_or_else(|| anyhow!("could not determine the user's home directory"))?
        .home_dir()
        .to_path_buf();
    Ok(home.join(".config").join(APP_DIR))
}

pub fn registry_path() -> Result<PathBuf> {
    Ok(config_dir()
        .context("resolving pier config dir")?
        .join(REGISTRY_FILE))
}
