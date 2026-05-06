use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};

use pier::commands;
use pier::shell::ShellKind;

#[derive(Parser)]
#[command(
    name = "pj",
    version,
    about = "pier — dock at any project in one command",
    long_about = "pier is a CLI for registering and switching between dev projects.\n\
                  Use `pj init <shell>` and source the output to enable interactive \
                  switching from your shell.",
    arg_required_else_help = true
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Register a directory as a project
    Register {
        /// Path to register (defaults to the current directory)
        path: Option<PathBuf>,
        /// Override the auto-detected project name
        #[arg(long)]
        name: Option<String>,
    },
    /// List all registered projects
    List,
    /// Remove a project from the registry
    Remove {
        /// Project name to remove
        name: String,
    },
    /// Print shell integration code (eval this in your shell init file)
    Init {
        /// Shell to generate init code for
        #[arg(value_enum)]
        shell: ShellKind,
    },
    /// Internal: emit shell snippet to switch to a project. Used by the shell
    /// wrapper produced by `pj init`. Not intended for direct use.
    #[command(name = "__shell", hide = true)]
    Shell {
        /// Shell flavor producing this call
        #[arg(long, value_enum, default_value_t = ShellKind::Bash)]
        shell: ShellKind,
        /// Project name (omit to launch the interactive picker)
        name: Option<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Register { path, name } => commands::register::run(path, name),
        Commands::List => commands::list::run(),
        Commands::Remove { name } => commands::remove::run(&name),
        Commands::Init { shell } => commands::init::run(shell),
        Commands::Shell { shell, name } => commands::switch::run(shell, name.as_deref()),
    }
}
