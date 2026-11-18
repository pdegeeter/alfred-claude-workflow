//! Alfred commands end to end, configured by workflow variables and answered by a fake `claude`.

mod fixture;

use std::collections::HashMap;

use alfred_claude::command::{AspellCommand, ClaudeCommand, CommandRunner, TranslateCommand};
use fixture::FakeClaudeCli;

fn runner(claude: &FakeClaudeCli, variables: &[(&str, &str)]) -> CommandRunner {
    let mut all: HashMap<String, String> = variables
        .iter()
        .map(|(name, value)| ((*name).to_owned(), (*value).to_owned()))
        .collect();
    all.insert("claude_path".to_owned(), claude.path());

    CommandRunner::new(all)
}

#[test]
fn claude_returns_the_answer_as_an_alfred_item() {
    let claude = FakeClaudeCli::answering("Paris\n\nCapital of France.");

    let output = ClaudeCommand::run(&runner(&claude, &[]), "Capital of France?").unwrap();

    assert_eq!(output["items"][0]["title"], "Paris");
    assert_eq!(output["variables"]["response"], "Paris\n\nCapital of France.");
    assert_eq!(claude.stdin(), "Capital of France?");
}

#[test]
fn defaults_to_haiku_at_low_effort() {
    let claude = FakeClaudeCli::answering("Hi");

    ClaudeCommand::run(&runner(&claude, &[]), "Hello").unwrap();

    assert_eq!(claude.option("--model").as_deref(), Some("haiku"));
    assert_eq!(claude.option("--effort").as_deref(), Some("low"));
}

#[test]
fn uses_the_configured_model_and_level() {
    let claude = FakeClaudeCli::answering("Hi");

    ClaudeCommand::run(&runner(&claude, &[("model", "opus"), ("level", "high")]), "Hello").unwrap();

    assert_eq!(claude.option("--model").as_deref(), Some("opus"));
    assert_eq!(claude.option("--effort").as_deref(), Some("high"));
}

#[test]
fn every_command_replaces_the_coding_system_prompt() {
    let claude = FakeClaudeCli::answering("Hi");
    let runner = runner(&claude, &[]);

    ClaudeCommand::run(&runner, "Hello").unwrap();
    assert!(claude.option("--system-prompt").unwrap().contains("macOS launcher"));

    AspellCommand::run(&runner, "Helo").unwrap();
    assert!(
        claude
            .option("--system-prompt")
            .unwrap()
            .contains("spelling and grammar checker")
    );

    TranslateCommand::run(&runner, "Bonjour").unwrap();
    assert!(claude.option("--system-prompt").unwrap().contains("silent translator"));
}

#[test]
fn rejects_an_unknown_level_without_calling_claude() {
    let claude = FakeClaudeCli::answering("Hi");

    let error = ClaudeCommand::run(&runner(&claude, &[("level", "extreme")]), "Hello").unwrap_err();

    assert_eq!(
        error.to_string(),
        "Unknown level \"extreme\": use one of low, medium, high, xhigh, max"
    );
    assert_eq!(claude.args(), Vec::<String>::new());
}

#[test]
fn aspell_sends_the_text_untouched() {
    let claude = FakeClaudeCli::answering("Hello world");

    let output = AspellCommand::run(&runner(&claude, &[]), "Helo wrld").unwrap();

    assert_eq!(claude.stdin(), "Helo wrld");
    assert_eq!(output["variables"]["response"], "Hello world");
}

#[test]
fn translate_targets_the_default_language() {
    let claude = FakeClaudeCli::answering("Hallo Welt");

    TranslateCommand::run(&runner(&claude, &[("default_lang", "de")]), "Hello world").unwrap();

    assert_eq!(claude.stdin(), "Target language: German\n<text>\nHello world\n</text>");
}

#[test]
fn translate_targets_the_language_of_the_prefix() {
    let claude = FakeClaudeCli::answering("Bonjour le monde");

    TranslateCommand::run(&runner(&claude, &[("default_lang", "de")]), ":fr Hello world").unwrap();

    assert_eq!(claude.stdin(), "Target language: French\n<text>\nHello world\n</text>");
}

#[test]
fn translate_reads_a_bare_known_code_and_delimits_text_that_asks_questions() {
    let claude = FakeClaudeCli::answering("Ton avis ?");
    let query = "fr just following up: if you have an opinion?\n\nshould we do it now, or later?";

    TranslateCommand::run(&runner(&claude, &[("default_lang", "en")]), query).unwrap();

    assert_eq!(
        claude.stdin(),
        "Target language: French\n<text>\njust following up: if you have an opinion?\n\nshould we do it now, or later?\n</text>"
    );
}

#[test]
fn empty_queries_ask_for_input_without_calling_claude() {
    let claude = FakeClaudeCli::answering("Hi");
    let runner = runner(&claude, &[]);

    for output in [
        ClaudeCommand::run(&runner, "").unwrap(),
        AspellCommand::run(&runner, "").unwrap(),
        TranslateCommand::run(&runner, "").unwrap(),
    ] {
        assert_eq!(output["items"][0]["valid"], false);
    }
    assert_eq!(claude.args(), Vec::<String>::new());
}
