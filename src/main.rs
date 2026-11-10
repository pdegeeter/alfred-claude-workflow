use std::collections::HashMap;
use std::env;

use alfred_claude::command::{AspellCommand, ClaudeCommand, CommandRunner, TranslateCommand};
use alfred_claude::helper::AlfredOutput;
use anyhow::anyhow;

/// Usage: `alfred-claude <aspell|claude|translate> <query>`, configured by the Alfred workflow variables.
fn main() {
    let mut args = env::args_os().skip(1).map(|arg| arg.to_string_lossy().into_owned());
    let command = args.next().unwrap_or_default();
    let query = args.next().unwrap_or_default();
    let variables: HashMap<String, String> = env::vars_os()
        .filter_map(|(name, value)| Some((name.into_string().ok()?, value.into_string().ok()?)))
        .collect();
    let runner = CommandRunner::new(variables);

    let result = match command.as_str() {
        "aspell" => AspellCommand::run(&runner, &query),
        "claude" => ClaudeCommand::run(&runner, &query),
        "translate" => TranslateCommand::run(&runner, &query),
        other => Err(anyhow!("Unknown command \"{other}\": use aspell, claude or translate")),
    };
    let output = result.unwrap_or_else(|error| AlfredOutput::error(&format!("Error: {error:#}")));

    println!("{output}");
}
