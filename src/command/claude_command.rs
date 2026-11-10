use serde_json::Value;

use super::CommandRunner;
use crate::helper::AlfredOutput;

/// `claude`: free-form question.
pub struct ClaudeCommand;

impl ClaudeCommand {
    const SYSTEM_PROMPT: &str = "You are Claude, answering a question typed in a macOS launcher.
- Answer directly, without preamble; be concise unless the question asks for detail
- Format with Markdown: the answer is shown in a Markdown viewer
- Answer in the language of the question
- You have no tools and no access to files or the web: answer from your knowledge";

    /// # Errors
    /// When the model cannot be called.
    pub fn run(runner: &CommandRunner, query: &str) -> anyhow::Result<Value> {
        if query.is_empty() {
            return Ok(AlfredOutput::error("Please enter a message"));
        }

        Ok(AlfredOutput::response(&runner.ask(Self::SYSTEM_PROMPT, query)?))
    }
}
