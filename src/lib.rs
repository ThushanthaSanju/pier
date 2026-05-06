//! pier — dock at any project in one command
//!
//! Library crate exposing the core modules so they can be exercised by
//! integration tests and reused if pier is ever embedded in another tool.
//! The user-facing binary is named `pj`.

pub mod commands;
pub mod config;
pub mod paths;
pub mod registry;
pub mod shell;
pub mod ui;
