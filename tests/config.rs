//! Integration tests for `project.toml` loading from disk.

use std::fs;

use pier::config::ProjectConfig;
use tempfile::tempdir;

#[test]
fn returns_none_when_file_missing() {
    let dir = tempdir().unwrap();
    let cfg = ProjectConfig::load(dir.path()).unwrap();
    assert!(cfg.is_none());
}

#[test]
fn loads_full_config_from_disk() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("project.toml"),
        r#"
[project]
name = "acme"
description = "main api"

[env]
env_file = ".env"

[setup]
commands = ["docker compose up -d"]
"#,
    )
    .unwrap();

    let cfg = ProjectConfig::load(dir.path()).unwrap().unwrap();
    assert_eq!(cfg.project.name, "acme");
    assert_eq!(cfg.setup.commands.len(), 1);
    assert!(cfg.env.env_file.is_some());
}

#[test]
fn surfaces_parse_errors_with_path() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("project.toml"), "not valid = = toml").unwrap();
    let err = ProjectConfig::load(dir.path()).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("project.toml"), "error should name the file: {msg}");
}
