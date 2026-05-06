//! The project registry: a TOML-backed list of registered projects.
//!
//! The registry is the source of truth for `pj list`, `pj register`,
//! `pj remove`, and project lookup. It is intentionally a small, owned data
//! structure — load, mutate, save. No background sync, no caching layer.

use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RegistryError {
    #[error("project '{0}' is not registered")]
    NotFound(String),

    #[error("a project named '{0}' is already registered")]
    DuplicateName(String),

    #[error("'{query}' is ambiguous — matches: {}", candidates.join(", "))]
    Ambiguous {
        query: String,
        candidates: Vec<String>,
    },

    #[error("registry file I/O failed: {0}")]
    Io(#[from] std::io::Error),

    #[error("could not parse registry TOML: {0}")]
    TomlDe(#[from] toml::de::Error),

    #[error("could not serialize registry TOML: {0}")]
    TomlSer(#[from] toml::ser::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Project {
    pub name: String,
    pub path: PathBuf,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_accessed: Option<DateTime<Utc>>,
}

impl Project {
    pub fn new(name: impl Into<String>, path: impl Into<PathBuf>) -> Self {
        Self {
            name: name.into(),
            path: path.into(),
            last_accessed: None,
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Registry {
    #[serde(default, rename = "projects")]
    projects: Vec<Project>,
}

impl Registry {
    pub fn load(path: &Path) -> Result<Self, RegistryError> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let contents = std::fs::read_to_string(path)?;
        if contents.trim().is_empty() {
            return Ok(Self::default());
        }
        let registry: Registry = toml::from_str(&contents)?;
        Ok(registry)
    }

    pub fn save(&self, path: &Path) -> Result<(), RegistryError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let serialized = toml::to_string_pretty(self)?;
        std::fs::write(path, serialized)?;
        Ok(())
    }

    pub fn projects(&self) -> &[Project] {
        &self.projects
    }

    pub fn add(&mut self, project: Project) -> Result<(), RegistryError> {
        if self.projects.iter().any(|p| p.name == project.name) {
            return Err(RegistryError::DuplicateName(project.name));
        }
        self.projects.push(project);
        Ok(())
    }

    pub fn remove(&mut self, name: &str) -> Result<Project, RegistryError> {
        let pos = self
            .projects
            .iter()
            .position(|p| p.name == name)
            .ok_or_else(|| RegistryError::NotFound(name.to_string()))?;
        Ok(self.projects.remove(pos))
    }

    /// Find a project by name. Resolution order:
    ///   1. case-sensitive exact match
    ///   2. case-insensitive exact match
    ///   3. case-insensitive substring match (must be unique)
    pub fn find(&self, query: &str) -> Result<&Project, RegistryError> {
        if let Some(p) = self.projects.iter().find(|p| p.name == query) {
            return Ok(p);
        }
        let lower = query.to_lowercase();
        if let Some(p) = self
            .projects
            .iter()
            .find(|p| p.name.to_lowercase() == lower)
        {
            return Ok(p);
        }
        let matches: Vec<&Project> = self
            .projects
            .iter()
            .filter(|p| p.name.to_lowercase().contains(&lower))
            .collect();
        match matches.as_slice() {
            [] => Err(RegistryError::NotFound(query.to_string())),
            [only] => Ok(only),
            many => Err(RegistryError::Ambiguous {
                query: query.to_string(),
                candidates: many.iter().map(|p| p.name.clone()).collect(),
            }),
        }
    }

    /// Stamp `last_accessed = now` on the named project.
    pub fn touch(&mut self, name: &str) -> Result<(), RegistryError> {
        let project = self
            .projects
            .iter_mut()
            .find(|p| p.name == name)
            .ok_or_else(|| RegistryError::NotFound(name.to_string()))?;
        project.last_accessed = Some(Utc::now());
        Ok(())
    }

    /// Most-recently-accessed first. Never-accessed projects sort last,
    /// stable on insertion order.
    pub fn sorted_by_recency(&self) -> Vec<&Project> {
        let mut out: Vec<&Project> = self.projects.iter().collect();
        out.sort_by(|a, b| match (a.last_accessed, b.last_accessed) {
            (Some(x), Some(y)) => y.cmp(&x),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        });
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Registry {
        let mut r = Registry::default();
        r.add(Project::new("acme-backend", "/code/acme-backend"))
            .unwrap();
        r.add(Project::new("acme-web", "/code/acme-web")).unwrap();
        r.add(Project::new("personal-blog", "/code/blog")).unwrap();
        r
    }

    #[test]
    fn add_rejects_duplicates() {
        let mut r = sample();
        let err = r
            .add(Project::new("acme-backend", "/elsewhere"))
            .unwrap_err();
        assert!(matches!(err, RegistryError::DuplicateName(ref n) if n == "acme-backend"));
    }

    #[test]
    fn remove_returns_the_removed_project() {
        let mut r = sample();
        let removed = r.remove("acme-web").unwrap();
        assert_eq!(removed.name, "acme-web");
        assert_eq!(r.projects().len(), 2);
    }

    #[test]
    fn find_prefers_exact_match() {
        let r = sample();
        assert_eq!(r.find("acme-backend").unwrap().name, "acme-backend");
    }

    #[test]
    fn find_handles_case_insensitive_exact() {
        let r = sample();
        assert_eq!(r.find("ACME-BACKEND").unwrap().name, "acme-backend");
    }

    #[test]
    fn find_unique_substring_matches() {
        let r = sample();
        assert_eq!(r.find("blog").unwrap().name, "personal-blog");
    }

    #[test]
    fn find_ambiguous_substring_errors() {
        let r = sample();
        let err = r.find("acme").unwrap_err();
        match err {
            RegistryError::Ambiguous { candidates, .. } => {
                assert_eq!(candidates.len(), 2);
            }
            _ => panic!("expected Ambiguous, got {err:?}"),
        }
    }

    #[test]
    fn find_unknown_returns_not_found() {
        let r = sample();
        assert!(matches!(
            r.find("nope").unwrap_err(),
            RegistryError::NotFound(_)
        ));
    }

    #[test]
    fn touch_sets_last_accessed() {
        let mut r = sample();
        assert!(r.find("acme-web").unwrap().last_accessed.is_none());
        r.touch("acme-web").unwrap();
        assert!(r.find("acme-web").unwrap().last_accessed.is_some());
    }

    #[test]
    fn sorted_by_recency_orders_touched_first() {
        let mut r = sample();
        r.touch("personal-blog").unwrap();
        let order: Vec<&str> = r
            .sorted_by_recency()
            .iter()
            .map(|p| p.name.as_str())
            .collect();
        assert_eq!(order[0], "personal-blog");
    }
}
