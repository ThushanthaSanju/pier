//! Shell integration: init snippets and switch-script generation.
//!
//! The "cd problem" — a child process can't change the parent shell's
//! directory. Solution (zoxide-style): the binary emits a shell script to
//! stdout, and a wrapper function in the user's shell `eval`s it.

use std::path::Path;

use clap::ValueEnum;

use crate::config::ProjectConfig;

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ShellKind {
    Bash,
    Zsh,
    Fish,
}

impl ShellKind {
    fn name(self) -> &'static str {
        match self {
            ShellKind::Bash => "bash",
            ShellKind::Zsh => "zsh",
            ShellKind::Fish => "fish",
        }
    }
}

/// Init code to be eval'd in the user's shell rc. Defines the `pj` wrapper
/// function that handles the cd-into-parent-shell trick.
pub fn init_snippet(shell: ShellKind) -> String {
    match shell {
        ShellKind::Bash | ShellKind::Zsh => posix_init(shell),
        ShellKind::Fish => fish_init(),
    }
}

fn posix_init(shell: ShellKind) -> String {
    format!(
        r#"# pj shell integration ({name})
# Add to your rc file:
#   eval "$(pj init {name})"

pj() {{
    case "${{1:-}}" in
        register|list|remove|init|help|--help|-h|--version|-V)
            command pj "$@"
            return $?
            ;;
    esac
    local _pj_script
    _pj_script="$(command pj __shell --shell {name} "$@")" || return $?
    eval "$_pj_script"
}}
"#,
        name = shell.name()
    )
}

fn fish_init() -> String {
    String::from(
        r#"# pj shell integration (fish)
# Add to ~/.config/fish/config.fish:
#   pj init fish | source

function pj
    if test (count $argv) -gt 0
        switch $argv[1]
            case register list remove init help --help -h --version -V
                command pj $argv
                return $status
        end
    end
    set -l _pj_script (command pj __shell --shell fish $argv)
    or return $status
    eval $_pj_script
end
"#,
    )
}

/// Quote a string so it survives a single round of POSIX shell expansion.
/// Strategy: if the string is already safe, return it unchanged; otherwise
/// wrap in single quotes and escape any embedded single quotes.
pub fn shell_quote_posix(s: &str) -> String {
    if !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '/' | ':' | '@'))
    {
        return s.to_string();
    }
    let escaped = s.replace('\'', r"'\''");
    format!("'{escaped}'")
}

/// Fish quoting differs from POSIX. Single quotes in fish do not interpret
/// escapes — except that `\'` and `\\` are recognized inside them.
pub fn shell_quote_fish(s: &str) -> String {
    if !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '/' | ':' | '@'))
    {
        return s.to_string();
    }
    let escaped = s.replace('\\', r"\\").replace('\'', r"\'");
    format!("'{escaped}'")
}

/// Build the shell script that switches into a project. Emitted to stdout,
/// captured and `eval`'d by the shell wrapper.
pub fn build_switch_script(
    shell: ShellKind,
    project_path: &Path,
    config: Option<&ProjectConfig>,
) -> String {
    let mut script = String::new();
    let path_str = project_path.to_string_lossy();

    match shell {
        ShellKind::Bash | ShellKind::Zsh => {
            script.push_str(&format!("cd -- {} || return\n", shell_quote_posix(&path_str)));
        }
        ShellKind::Fish => {
            script.push_str(&format!("cd {}\n", shell_quote_fish(&path_str)));
        }
    }

    let Some(cfg) = config else {
        return script;
    };

    if let Some(env_file) = &cfg.env.env_file {
        let env_path = project_path.join(env_file);
        if env_path.exists() {
            match crate::config::parse_env_file(&env_path) {
                Ok(pairs) => script.push_str(&render_env(shell, &pairs)),
                Err(e) => script.push_str(&format!(
                    "{} pj: failed to read env file: {}\n",
                    comment(shell),
                    e
                )),
            }
        }
    }

    for cmd in &cfg.setup.commands {
        script.push_str(cmd);
        script.push('\n');
    }

    script
}

fn render_env(shell: ShellKind, pairs: &[(String, String)]) -> String {
    let mut out = String::new();
    for (k, v) in pairs {
        match shell {
            ShellKind::Bash | ShellKind::Zsh => {
                out.push_str(&format!("export {}={}\n", k, shell_quote_posix(v)));
            }
            ShellKind::Fish => {
                out.push_str(&format!("set -gx {} {}\n", k, shell_quote_fish(v)));
            }
        }
    }
    out
}

fn comment(shell: ShellKind) -> &'static str {
    match shell {
        ShellKind::Bash | ShellKind::Zsh | ShellKind::Fish => "#",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quote_posix_passes_safe_strings_through() {
        assert_eq!(shell_quote_posix("simple"), "simple");
        assert_eq!(shell_quote_posix("/usr/local/bin"), "/usr/local/bin");
        assert_eq!(shell_quote_posix("a-b_c.d"), "a-b_c.d");
    }

    #[test]
    fn quote_posix_wraps_strings_with_spaces() {
        assert_eq!(shell_quote_posix("hello world"), "'hello world'");
    }

    #[test]
    fn quote_posix_escapes_single_quotes() {
        assert_eq!(shell_quote_posix("it's"), r#"'it'\''s'"#);
    }

    #[test]
    fn quote_posix_handles_empty() {
        assert_eq!(shell_quote_posix(""), "''");
    }

    #[test]
    fn quote_fish_escapes_backslash_and_quote() {
        assert_eq!(shell_quote_fish(r"a\b"), r"'a\\b'");
        assert_eq!(shell_quote_fish("it's"), r"'it\'s'");
    }

    #[test]
    fn switch_script_emits_cd_for_bash() {
        let s = build_switch_script(ShellKind::Bash, Path::new("/tmp/x"), None);
        assert!(s.contains("cd -- /tmp/x"));
    }

    #[test]
    fn switch_script_quotes_paths_with_spaces() {
        let s = build_switch_script(ShellKind::Bash, Path::new("/tmp/has space"), None);
        assert!(s.contains("'/tmp/has space'"));
    }

    #[test]
    fn switch_script_emits_setup_commands() {
        let cfg = ProjectConfig {
            project: crate::config::ProjectMeta {
                name: "x".into(),
                description: None,
            },
            env: Default::default(),
            setup: crate::config::SetupConfig {
                commands: vec!["echo hi".into()],
            },
        };
        let s = build_switch_script(ShellKind::Bash, Path::new("/tmp/x"), Some(&cfg));
        assert!(s.contains("echo hi"));
    }

    #[test]
    fn init_snippet_includes_command_passthrough() {
        let s = init_snippet(ShellKind::Bash);
        assert!(s.contains("register|list|remove|init"));
        assert!(s.contains("__shell"));
    }
}
