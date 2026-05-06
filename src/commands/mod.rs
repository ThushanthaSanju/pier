//! One module per subcommand. Each module exposes a single `run` function
//! invoked from `main.rs`. Keeping them small and isolated keeps the
//! dispatch table in `main.rs` readable.

pub mod init;
pub mod list;
pub mod register;
pub mod remove;
pub mod switch;
