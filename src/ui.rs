//! Terminal output: tables, colors, relative timestamps, git branch lookup.
//!
//! Anything that touches stdout/stderr for the user (as opposed to
//! shell-eval'd snippets) lives here.

use std::path::Path;

use chrono::{DateTime, Utc};
use owo_colors::OwoColorize;
use tabled::{
    settings::{object::Columns, Alignment, Modify, Style},
    Table, Tabled,
};

use crate::registry::Project;

#[derive(Tabled)]
struct Row {
    #[tabled(rename = "NAME")]
    name: String,
    #[tabled(rename = "PATH")]
    path: String,
    #[tabled(rename = "BRANCH")]
    branch: String,
    #[tabled(rename = "LAST ACCESSED")]
    last_accessed: String,
}

pub fn render_projects_table(projects: &[&Project]) -> String {
    let rows: Vec<Row> = projects
        .iter()
        .map(|p| Row {
            name: p.name.bold().to_string(),
            path: shorten_home(&p.path.to_string_lossy()).dimmed().to_string(),
            branch: current_git_branch(&p.path)
                .map(|b| b.cyan().to_string())
                .unwrap_or_else(|| "—".dimmed().to_string()),
            last_accessed: relative_time(p.last_accessed).dimmed().to_string(),
        })
        .collect();

    let mut table = Table::new(rows);
    table
        .with(Style::blank())
        .with(Modify::new(Columns::new(..)).with(Alignment::left()));
    table.to_string()
}

/// Replace `$HOME` with `~` for compactness in listings.
fn shorten_home(path: &str) -> String {
    let Some(home) = directories::BaseDirs::new().map(|d| d.home_dir().to_path_buf()) else {
        return path.to_string();
    };
    let home = home.to_string_lossy();
    if let Some(rest) = path.strip_prefix(home.as_ref()) {
        format!("~{rest}")
    } else {
        path.to_string()
    }
}

/// Human relative time. Falls back to "never" for `None`.
pub fn relative_time(t: Option<DateTime<Utc>>) -> String {
    let Some(t) = t else {
        return "never".to_string();
    };
    let delta = Utc::now().signed_duration_since(t);
    let secs = delta.num_seconds();
    if secs < 0 {
        return "in the future".to_string();
    }
    if secs < 45 {
        return "just now".to_string();
    }
    if secs < 90 {
        return "a minute ago".to_string();
    }
    let minutes = delta.num_minutes();
    if minutes < 60 {
        return format!("{minutes} minutes ago");
    }
    let hours = delta.num_hours();
    if hours < 24 {
        return format!("{hours} hour{} ago", plural(hours));
    }
    let days = delta.num_days();
    if days < 30 {
        return format!("{days} day{} ago", plural(days));
    }
    let months = days / 30;
    if months < 12 {
        return format!("{months} month{} ago", plural(months));
    }
    let years = days / 365;
    format!("{years} year{} ago", plural(years))
}

fn plural(n: i64) -> &'static str {
    if n == 1 {
        ""
    } else {
        "s"
    }
}

/// Current git branch for the working tree at `path`, if any. Shells out to
/// `git`; if `git` is missing or this isn't a repo, returns `None`.
pub fn current_git_branch(path: &Path) -> Option<String> {
    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(path)
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let s = String::from_utf8(output.stdout).ok()?;
    let trimmed = s.trim();
    if trimmed.is_empty() || trimmed == "HEAD" {
        None
    } else {
        Some(trimmed.to_string())
    }
}

pub fn print_success(msg: &str) {
    eprintln!("{} {}", "✓".green().bold(), msg);
}

pub fn print_warn(msg: &str) {
    eprintln!("{} {}", "!".yellow().bold(), msg);
}

pub fn print_info(msg: &str) {
    eprintln!("{} {}", "·".cyan().bold(), msg);
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn relative_time_handles_none() {
        assert_eq!(relative_time(None), "never");
    }

    #[test]
    fn relative_time_handles_recent() {
        let t = Utc::now() - Duration::seconds(10);
        assert_eq!(relative_time(Some(t)), "just now");
    }

    #[test]
    fn relative_time_handles_minutes() {
        let t = Utc::now() - Duration::minutes(5);
        assert_eq!(relative_time(Some(t)), "5 minutes ago");
    }

    #[test]
    fn relative_time_handles_hours() {
        let t = Utc::now() - Duration::hours(3);
        assert_eq!(relative_time(Some(t)), "3 hours ago");
    }

    #[test]
    fn relative_time_singular_forms() {
        let t = Utc::now() - Duration::hours(1);
        assert_eq!(relative_time(Some(t)), "1 hour ago");
    }

    #[test]
    fn relative_time_handles_days() {
        let t = Utc::now() - Duration::days(2);
        assert_eq!(relative_time(Some(t)), "2 days ago");
    }
}
