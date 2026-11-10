//! Claude Code CLI wrapper, running a fake `claude` executable.

mod fixture;

use std::env;
use std::fs;

use alfred_claude::client::ClaudeCliClient;
use alfred_claude::types::{ClaudeRequest, EffortLevel};
use fixture::FakeClaudeCli;
use tempfile::TempDir;

fn request() -> ClaudeRequest {
    ClaudeRequest {
        effort: EffortLevel::Low,
        model: "haiku".to_owned(),
        system_prompt: "Be brief.".to_owned(),
    }
}

#[test]
fn returns_the_answer() {
    let claude = FakeClaudeCli::answering("Hello");

    let answer = ClaudeCliClient::locate(Some(&claude.path()), None)
        .ask(&request(), "Bonjour")
        .unwrap();

    assert_eq!(answer, "Hello");
}

#[test]
fn sends_a_single_turn_without_tools_settings_mcp_servers_nor_session() {
    let claude = FakeClaudeCli::answering("Hello");

    ClaudeCliClient::locate(Some(&claude.path()), None)
        .ask(&request(), "Bonjour")
        .unwrap();

    assert_eq!(
        claude.args(),
        [
            "--print",
            "--output-format",
            "json",
            "--model",
            "haiku",
            "--effort",
            "low",
            "--system-prompt",
            "Be brief.",
            "--tools",
            "",
            "--strict-mcp-config",
            "--setting-sources",
            "",
            "--disable-slash-commands",
            "--no-session-persistence",
        ]
    );
}

#[test]
fn sends_the_prompt_on_stdin_so_it_is_never_read_as_an_option() {
    let claude = FakeClaudeCli::answering("Hello");

    ClaudeCliClient::locate(Some(&claude.path()), None)
        .ask(&request(), "--help me\nsecond line")
        .unwrap();

    assert_eq!(claude.stdin(), "--help me\nsecond line");
    assert!(!claude.args().iter().any(|arg| arg.contains("help me")));
}

#[test]
fn runs_outside_any_project_directory() {
    let claude = FakeClaudeCli::answering("Hello");

    ClaudeCliClient::locate(Some(&claude.path()), None)
        .ask(&request(), "Bonjour")
        .unwrap();

    assert_eq!(
        claude.last_cwd(),
        fs::canonicalize(env::temp_dir()).unwrap().display().to_string()
    );
}

#[test]
fn puts_its_own_and_homebrew_directories_first_on_the_path() {
    let claude = FakeClaudeCli::answering("Hello");

    ClaudeCliClient::locate(Some(&claude.path()), None)
        .ask(&request(), "Bonjour")
        .unwrap();

    let expected_prefix = format!("{}:/opt/homebrew/bin:/usr/local/bin:", claude.dir().display());
    assert!(
        claude.last_path().starts_with(&expected_prefix),
        "{}",
        claude.last_path()
    );
}

#[test]
fn disables_nonessential_traffic_which_delays_the_exit() {
    let claude = FakeClaudeCli::answering("Hello");

    ClaudeCliClient::locate(Some(&claude.path()), None)
        .ask(&request(), "Bonjour")
        .unwrap();

    assert_eq!(
        claude.env("CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC").as_deref(),
        Some("1")
    );
}

#[test]
fn surfaces_the_error_reported_by_the_cli() {
    let claude = FakeClaudeCli::answering("");
    claude.respond(r#"{"is_error":true,"result":"API Error: 529 Overloaded"}"#, "", 1);

    let error = ClaudeCliClient::locate(Some(&claude.path()), None)
        .ask(&request(), "Bonjour")
        .unwrap_err();

    assert_eq!(error.to_string(), "claude failed: API Error: 529 Overloaded");
}

#[test]
fn explains_how_to_log_in() {
    let claude = FakeClaudeCli::answering("");
    claude.respond(
        r#"{"is_error":true,"result":"Not logged in · Please run /login"}"#,
        "",
        1,
    );

    let error = ClaudeCliClient::locate(Some(&claude.path()), None)
        .ask(&request(), "Bonjour")
        .unwrap_err();

    assert_eq!(
        error.to_string(),
        "Claude Code is not logged in: run `claude` in a terminal, then /login"
    );
}

#[test]
fn surfaces_stderr_of_a_rejected_invocation() {
    let claude = FakeClaudeCli::answering("");
    claude.respond("", "error: unknown option '--effort'\n", 1);

    let error = ClaudeCliClient::locate(Some(&claude.path()), None)
        .ask(&request(), "Bonjour")
        .unwrap_err();

    assert_eq!(error.to_string(), "claude failed: error: unknown option '--effort'");
}

#[test]
fn reports_a_missing_cli() {
    let error = ClaudeCliClient::locate(Some("/nonexistent/claude"), None)
        .ask(&request(), "Bonjour")
        .unwrap_err();

    assert_eq!(
        error.to_string(),
        "Claude Code not found at /nonexistent/claude: install it or set claude_path"
    );
}

#[test]
fn prefers_the_native_installer_location_in_the_home_directory() {
    let home = TempDir::new().unwrap();
    let bin = home.path().join(".local/bin");
    fs::create_dir_all(&bin).unwrap();
    fs::write(bin.join("claude"), "").unwrap();

    let client = ClaudeCliClient::locate(None, Some(&home.path().display().to_string()));

    assert_eq!(client.path(), bin.join("claude"));
}

#[test]
fn uses_the_configured_path_first() {
    let client = ClaudeCliClient::locate(Some("/custom/claude"), Some("/Users/someone"));

    assert_eq!(client.path().display().to_string(), "/custom/claude");
}
