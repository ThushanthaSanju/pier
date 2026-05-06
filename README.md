<div align="center">

# ⚓ pier

**Dock at any project in one command.**

A tiny, fast project switcher written in Rust. Replace the morning ritual of
`cd`, `source venv/bin/activate`, `set -a; source .env; set +a`, and
`docker compose up -d` with a single keystroke.

[![CI](https://github.com/ThushanthaSanju/pier/actions/workflows/ci.yml/badge.svg)](https://github.com/ThushanthaSanju/pier/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org)

</div>

---

```text
$ pj backend
✓ acme-backend
  loaded .env (12 vars)
  starting docker compose
~/code/acme-backend $ _
```

> The CLI you type is `pj` (project jumper). The project itself is `pier` —
> the place you dock and depart from.

<!-- Replace with a real terminal recording before publishing. -->
<p align="center">
  <img src="docs/demo.gif" alt="pier demo" width="720" />
</p>

## Table of contents

- [Why pier](#why-pier)
- [Install](#install)
- [Shell setup](#shell-setup)
- [5-minute walkthrough](#5-minute-walkthrough)
- [Commands](#commands)
- [Per-project config](#per-project-config)
- [Where things live](#where-things-live)
- [How it works](#how-it-works)
- [FAQ](#faq)
- [Comparison](#comparison)
- [Roadmap](#roadmap)
- [Contributing](#contributing)
- [License](#license)

## Why pier

- **One command, not five.** `cd`, env vars, setup commands — all fused into a single jump.
- **Stays in your shell.** No daemon. No subshell. No surprises. Like `zoxide`, pier emits a tiny shell snippet your shell `eval`s — your existing prompt, history, and aliases keep working.
- **Cross-shell.** Bash, zsh, and fish are all first-class. The same `.env` file works across them because pier parses it itself.
- **Explicit registry.** Projects are added by you, not magically detected. No directory crawling, no surprise reindexing, no `~/code` assumptions.
- **Small surface area.** Six commands. One config file per project — and the config file is optional.
- **Fast.** Native Rust binary. Cold start is single-digit milliseconds.

## Install

### Via Cargo (recommended for now)

```bash
cargo install --git https://github.com/ThushanthaSanju/pier
```

This puts the `pj` binary on your `PATH` (typically `~/.cargo/bin`).

### From source

```bash
git clone https://github.com/ThushanthaSanju/pier
cd pier
cargo install --path .
```

### Verify

```bash
$ pj --version
pj 0.1.0
```

> Pre-built binaries and Homebrew/apt/winget packages are on the [roadmap](#roadmap). Until then, `cargo install` is the way.

## Shell setup

pier needs a tiny wrapper function in your shell so it can change directories
in your current shell (the [cd problem](#how-it-works), explained below).
Pick your shell:

### bash

```bash
echo 'eval "$(pj init bash)"' >> ~/.bashrc
exec bash
```

### zsh

```bash
echo 'eval "$(pj init zsh)"' >> ~/.zshrc
exec zsh
```

### fish

```fish
echo 'pj init fish | source' >> ~/.config/fish/config.fish
exec fish
```

That's it. You now have a `pj` shell function ready to go.

## 5-minute walkthrough

```bash
# Step 1: register a project (run from inside it)
$ cd ~/code/acme-backend
$ pj register
✓ registered 'acme-backend' → /Users/me/code/acme-backend

# Step 2: register a few more
$ pj register ~/code/acme-web
$ pj register ~/code/blog --name personal-blog

# Step 3: list them
$ pj list
 NAME            PATH                       BRANCH   LAST ACCESSED
 acme-backend    ~/code/acme-backend        main     never
 acme-web        ~/code/acme-web            main     never
 personal-blog   ~/code/blog                draft    never

# Step 4: jump to one by name
$ pj acme-backend
~/code/acme-backend $

# Step 5: substring matching just works
$ pj backend
~/code/acme-backend $

# Step 6: no argument? interactive fuzzy picker
$ pj
? Switch to:
> acme-backend            ~/code/acme-backend
  acme-web                ~/code/acme-web
  personal-blog           ~/code/blog
  [type to filter · ↑↓ to move · enter to select]

# Step 7: drop a project.toml in the project root for env + setup
$ cat ~/code/acme-backend/project.toml
[project]
name = "acme-backend"
description = "Acme Corp main API"

[env]
env_file = ".env"

[setup]
commands = ["docker compose up -d"]

# Now jumping in loads .env and starts services automatically:
$ pj acme-backend
# .env vars exported, docker compose started, you're at the prompt.
```

## Commands

| Command                            | What it does                                                                |
| ---------------------------------- | --------------------------------------------------------------------------- |
| `pj register [PATH]`               | Register the current directory (or `PATH`) as a project.                    |
| `pj register [PATH] --name <NAME>` | Same, but use `<NAME>` instead of the inferred directory name.              |
| `pj list`                          | Print all registered projects, sorted by recency. Shows git branch if any.  |
| `pj <name>`                        | Switch to a project. Resolves exact → case-insensitive → unique substring.  |
| `pj` (no args)                     | Launch the interactive fuzzy picker.                                        |
| `pj remove <name>`                 | Remove a project from the registry. Doesn't touch the directory.            |
| `pj init <bash\|zsh\|fish>`        | Print the shell wrapper to source.                                          |

Run `pj --help` or `pj <command> --help` for full details.

## Per-project config

Drop a `project.toml` at any project root to wire up env vars and setup
commands. **Every section is optional** — projects without a `project.toml`
just `cd` and call it a day.

```toml
[project]
name = "acme-backend"
description = "Acme Corp main API"     # optional, shown in errors and listings later

[env]
# Path to a .env-style file, relative to the project root.
# Variables are exported into your shell on switch.
env_file = ".env"

[setup]
# Shell commands run after the cd, in order.
# Output goes straight to your terminal.
commands = [
  "docker compose up -d",
  "echo 'ready'",
]
```

### Supported `.env` syntax

pier parses `.env` itself (rather than `source`-ing it) so the same file
works in bash, zsh, and fish:

```bash
# Comments are ignored
KEY=value
QUOTED="value with spaces"
SINGLE='also fine'
export EXPORTED=allowed
EMPTY=
WITH_HASH=val # trailing comment ok
```

What's **not** supported (intentionally — that belongs in your shell rc):

- Variable expansion (`$OTHER`)
- Command substitution (`$(...)`)
- Multi-line values
- Conditionals

If you need any of that, run it from `[setup].commands` instead.

## Where things live

| File                              | Purpose                                                |
| --------------------------------- | ------------------------------------------------------ |
| `~/.config/pier/projects.toml`    | The project registry. Edit by hand if you want.        |
| `<project>/project.toml`          | Per-project env + setup. Optional.                     |
| `<project>/.env` (or any path)    | Env file referenced from `project.toml`. Optional.     |

The registry location can be overridden with the `PIER_CONFIG_DIR`
environment variable. (Useful for sandboxes, dotfile setups, and tests.)

The registry is a plain TOML file. It's safe to edit by hand, version-control,
or sync between machines:

```toml
[[projects]]
name = "acme-backend"
path = "/Users/me/code/acme-backend"
last_accessed = "2026-05-06T14:30:00Z"

[[projects]]
name = "personal-blog"
path = "/Users/me/code/blog"
```

## How it works

### The cd problem

A child process can't change its parent shell's working directory. So how
does `pj backend` actually `cd` you anywhere? The same way `zoxide` does:

1. `pj init <shell>` emits a small shell function called `pj`.
2. When you run `pj backend`, that function calls the binary as `pj __shell --shell <your-shell> backend`.
3. The binary writes a script to **stdout**:
   ```bash
   cd -- '/Users/me/code/acme-backend' || return
   export DATABASE_URL='postgres://...'
   docker compose up -d
   ```
4. The shell function `eval`s that script in your current shell.

Subcommands like `register`, `list`, `remove`, `init` skip the eval path
and run as a regular subprocess.

### Architecture

```
┌──────────────┐  pj backend   ┌──────────────────┐  __shell  ┌──────────────┐
│  your shell  │ ────────────▶ │  pj() function   │ ────────▶ │  pj binary   │
│              │               │  (from `pj init`)│           │              │
│              │ ◀──────────── │                  │ ◀──────── │              │
└──────────────┘    eval       └──────────────────┘   stdout  └──────────────┘
                                                              shell snippet
```

## FAQ

**Why not just use `alias` or shell functions?**
You can, and many people do. pier is for when you have ten of them, want a
fuzzy picker, want a shared registry across shells, and want per-project
env/setup tied to the directory rather than your dotfiles.

**Why a registry instead of auto-detecting projects?**
Auto-detection means a slow startup, false positives, and surprises when you
clone something into the wrong place. A 30-second `pj register` is a fair
trade for years of predictable behavior.

**Does pier touch my files?**
Only `~/.config/pier/projects.toml`. It never modifies your project
directories. `pj remove` only removes the registry entry.

**Will pier work over SSH / in a container?**
Yes. It's a single static-ish Rust binary; copy it where you need it. The
registry is just a TOML file.

**Can I version-control my registry?**
Yes. Symlink `~/.config/pier/projects.toml` into your dotfiles repo. The file
is plain TOML, so diffs are clean.

**Why is the binary called `pj` and the project `pier`?**
Same reason `ripgrep` calls its binary `rg` — the binary should be short
because you type it 100 times a day; the project name should be
memorable because you type it once.

**How do I uninstall?**

```bash
cargo uninstall pier
# remove the eval line from your shell rc
rm -rf ~/.config/pier
```

## Comparison

| Tool         | What it does                                | What it doesn't                              |
| ------------ | ------------------------------------------- | -------------------------------------------- |
| **pier**     | Named project switcher + env + setup        | Doesn't track every dir you visit            |
| `zoxide`     | Frecency-ranked `cd` to any visited dir     | No env loading, no setup commands, no names  |
| `direnv`     | Auto-loads env on `cd`                      | No registry, no fuzzy switch, no setup       |
| `autoenv`    | Auto-runs `.env` when entering a directory  | Same gap as direnv                           |
| `tmuxinator` | Spins up a tmux layout per project          | Tmux-only, no plain switching                |

pier is happy to coexist with all of them. `direnv` users typically don't
need `[env]` in `project.toml`; `zoxide` users typically use `pj` for the
named/setup case and `z` for everything else.

## Roadmap

The MVP is intentionally small. Planned next, roughly in priority order:

- [ ] tmux session per project (attach if exists, create if not)
- [ ] Pre-built binaries on the GitHub release page
- [ ] Homebrew formula
- [ ] Shell completions (`pj completion <shell>`)
- [ ] `pj edit` to open the current project's `project.toml` in `$EDITOR`
- [ ] Per-shell `[setup]` blocks for bash/zsh/fish-specific commands
- [ ] Language version manager hooks (asdf, mise, rtx)
- [ ] Project templates (`pj new <template>`)

If one of these matters to you, [open an issue](https://github.com/ThushanthaSanju/pier/issues) and say so — it bumps priority.

## Contributing

Bug reports, ideas, and PRs are all welcome. The codebase is small and
intentionally favors clarity over cleverness.

### Layout

```
pier/
├── Cargo.toml
├── src/
│   ├── main.rs           # clap entry point + dispatch
│   ├── lib.rs            # re-exports for tests
│   ├── paths.rs          # ~/.config/pier resolution
│   ├── registry.rs       # the project registry
│   ├── config.rs         # project.toml + .env parsing
│   ├── shell.rs          # init snippets + switch script generation
│   ├── ui.rs             # tables, colors, relative time, git branch
│   └── commands/
│       ├── register.rs
│       ├── list.rs
│       ├── remove.rs
│       ├── init.rs
│       └── switch.rs
└── tests/
    ├── registry.rs       # round-trip integration tests
    └── config.rs         # project.toml loading tests
```

### Develop

```bash
git clone https://github.com/ThushanthaSanju/pier
cd pier

# Run the test suite (36 tests across unit + integration)
cargo test

# Lint with clippy and warnings-as-errors
cargo clippy --all-targets -- -D warnings

# Format check
cargo fmt --check

# Run locally without installing
cargo run -- list
```

### Principles for PRs

- Keep modules small and focused. One subcommand per file in `src/commands/`.
- Library code returns `thiserror` enums; binary code uses `anyhow` with `.with_context(...)`.
- New behavior comes with a test. Aim for the easy 80% — full coverage isn't required, but "I tested it manually once" doesn't ship.
- No silent failures. Surface issues through `ui::print_warn` or return an error.
- No mutation-heavy APIs in the library. Prefer `&mut self` only at the registry boundary.

## License

MIT. See [LICENSE](LICENSE).
