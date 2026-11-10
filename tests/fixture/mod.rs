//! Test doubles shared by the integration tests; each test crate uses a subset of them.
#![allow(dead_code, unused_imports)]

mod fake_claude_cli;

pub use fake_claude_cli::FakeClaudeCli;
