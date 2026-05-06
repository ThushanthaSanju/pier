//! Integration tests for the registry: round-trip serialization and the
//! load/save lifecycle against a real temp directory.

use std::path::PathBuf;

use pier::registry::{Project, Registry};
use tempfile::tempdir;

#[test]
fn save_then_load_round_trips() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("projects.toml");

    let mut registry = Registry::default();
    registry
        .add(Project::new("alpha", PathBuf::from("/tmp/alpha")))
        .unwrap();
    registry
        .add(Project::new("beta", PathBuf::from("/tmp/beta")))
        .unwrap();
    registry.touch("alpha").unwrap();

    registry.save(&path).unwrap();

    let reloaded = Registry::load(&path).unwrap();
    assert_eq!(reloaded.projects().len(), 2);
    assert_eq!(reloaded.projects()[0].name, "alpha");
    assert!(reloaded.projects()[0].last_accessed.is_some());
    assert!(reloaded.projects()[1].last_accessed.is_none());
}

#[test]
fn load_missing_file_returns_empty_registry() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("does-not-exist.toml");
    let registry = Registry::load(&path).unwrap();
    assert!(registry.projects().is_empty());
}

#[test]
fn load_empty_file_returns_empty_registry() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("projects.toml");
    std::fs::write(&path, "").unwrap();
    let registry = Registry::load(&path).unwrap();
    assert!(registry.projects().is_empty());
}

#[test]
fn save_creates_parent_directory() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("nested").join("deeper").join("projects.toml");
    let registry = Registry::default();
    registry.save(&path).unwrap();
    assert!(path.exists());
}

#[test]
fn remove_then_save_persists() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("projects.toml");

    let mut registry = Registry::default();
    registry
        .add(Project::new("keep", PathBuf::from("/tmp/keep")))
        .unwrap();
    registry
        .add(Project::new("drop", PathBuf::from("/tmp/drop")))
        .unwrap();
    registry.save(&path).unwrap();

    let mut registry = Registry::load(&path).unwrap();
    registry.remove("drop").unwrap();
    registry.save(&path).unwrap();

    let registry = Registry::load(&path).unwrap();
    assert_eq!(registry.projects().len(), 1);
    assert_eq!(registry.projects()[0].name, "keep");
}
