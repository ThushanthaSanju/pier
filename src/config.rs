//! Per-project `project.toml`.
//!
//! Optional. When present, `pj` reads it on switch to load env vars and run
//! setup commands. The schema is small on purpose; if you find yourself
//! reaching for more, that probably belongs in a Makefile or shell script.

use std::path::{Path, PathBuf};

use serde::Deserialize;
use thiserror::Error;

pub const PROJECT_CONFIG_FILE: &str = "project.toml";

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("could not read {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("could not parse {path}: {source}")]
    Parse {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct ProjectConfig {
    pub project: ProjectMeta,
    #[serde(default)]
    pub env: EnvConfig,
    #[serde(default)]
    pub setup: SetupConfig,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct ProjectMeta {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
pub struct EnvConfig {
    /// Path to a `.env`-style file, relative to the project root.
    #[serde(default)]
    pub env_file: Option<PathBuf>,
}

#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
pub struct SetupConfig {
    /// Shell commands to run after switching, in order.
    #[serde(default)]
    pub commands: Vec<String>,
}

impl ProjectConfig {
    /// Load `project.toml` from `project_root`. Returns `Ok(None)` when no
    /// config file exists — that is a normal, supported state.
    pub fn load(project_root: &Path) -> Result<Option<Self>, ConfigError> {
        let path = project_root.join(PROJECT_CONFIG_FILE);
        if !path.exists() {
            return Ok(None);
        }
        let contents = std::fs::read_to_string(&path).map_err(|source| ConfigError::Io {
            path: path.clone(),
            source,
        })?;
        let parsed: ProjectConfig =
            toml::from_str(&contents).map_err(|source| ConfigError::Parse {
                path: path.clone(),
                source,
            })?;
        Ok(Some(parsed))
    }
}

/// Parse a `.env`-style file into `(key, value)` pairs.
///
/// Supported syntax (intentionally minimal):
/// - `KEY=value`
/// - `KEY="value"` and `KEY='value'`
/// - `# comment` and blank lines
/// - leading `export ` is allowed and stripped
pub fn parse_env_file(path: &Path) -> Result<Vec<(String, String)>, ConfigError> {
    let contents = std::fs::read_to_string(path).map_err(|source| ConfigError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    Ok(parse_env_str(&contents))
}

pub fn parse_env_str(contents: &str) -> Vec<(String, String)> {
    contents.lines().filter_map(parse_env_line).collect()
}

fn parse_env_line(raw: &str) -> Option<(String, String)> {
    let line = raw.trim();
    if line.is_empty() || line.starts_with('#') {
        return None;
    }
    let line = line.strip_prefix("export ").unwrap_or(line);
    let (key, value) = line.split_once('=')?;
    let key = key.trim();
    if key.is_empty() {
        return None;
    }
    let value = strip_inline_comment(value.trim());
    let value = unquote(value);
    Some((key.to_string(), value))
}

fn strip_inline_comment(s: &str) -> &str {
    if s.starts_with('"') || s.starts_with('\'') {
        return s;
    }
    match s.find(" #") {
        Some(i) => s[..i].trim_end(),
        None => s,
    }
}

fn unquote(s: &str) -> String {
    let bytes = s.as_bytes();
    if bytes.len() >= 2
        && ((bytes[0] == b'"' && bytes[bytes.len() - 1] == b'"')
            || (bytes[0] == b'\'' && bytes[bytes.len() - 1] == b'\''))
    {
        s[1..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_full_config() {
        let toml_str = r#"
            [project]
            name = "acme"
            description = "main api"

            [env]
            env_file = ".env"

            [setup]
            commands = ["docker compose up -d", "echo ready"]
        "#;
        let cfg: ProjectConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(cfg.project.name, "acme");
        assert_eq!(cfg.project.description.as_deref(), Some("main api"));
        assert_eq!(cfg.env.env_file, Some(PathBuf::from(".env")));
        assert_eq!(cfg.setup.commands.len(), 2);
    }

    #[test]
    fn parses_minimal_config() {
        let toml_str = r#"[project]
            name = "minimal"
        "#;
        let cfg: ProjectConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(cfg.project.name, "minimal");
        assert!(cfg.project.description.is_none());
        assert!(cfg.env.env_file.is_none());
        assert!(cfg.setup.commands.is_empty());
    }

    #[test]
    fn env_parser_handles_common_shapes() {
        let contents = r#"
# comment line
KEY1=value1
KEY2="quoted value"
KEY3='single quoted'
export EXPORTED=ok
EMPTY=
WITH_HASH=val # trailing comment
"#;
        let pairs = parse_env_str(contents);
        let map: std::collections::HashMap<_, _> = pairs.into_iter().collect();
        assert_eq!(map.get("KEY1").map(String::as_str), Some("value1"));
        assert_eq!(map.get("KEY2").map(String::as_str), Some("quoted value"));
        assert_eq!(map.get("KEY3").map(String::as_str), Some("single quoted"));
        assert_eq!(map.get("EXPORTED").map(String::as_str), Some("ok"));
        assert_eq!(map.get("EMPTY").map(String::as_str), Some(""));
        assert_eq!(map.get("WITH_HASH").map(String::as_str), Some("val"));
    }

    #[test]
    fn env_parser_skips_malformed_lines() {
        let pairs = parse_env_str("not_a_kv_pair\n=novalue\n=\n");
        assert!(pairs.is_empty());
    }
}
