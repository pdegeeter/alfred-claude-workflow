//! Alfred workflow for Claude: one binary serving the `aspell`, `claude` and `translate` commands
//! through `claude -p`, the non-interactive mode of the Claude Code CLI.

pub mod client;
pub mod command;
pub mod helper;
pub mod types;
