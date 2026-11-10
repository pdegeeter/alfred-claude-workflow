use std::collections::HashMap;

use crate::client::ClaudeCliClient;
use crate::types::{ClaudeRequest, EffortLevel};

/// Sends a prompt to the model selected in the workflow variables.
pub struct CommandRunner {
    variables: HashMap<String, String>,
}

impl CommandRunner {
    const DEFAULT_MODEL: &str = "haiku";
    /// Short answers: spelling, translation, quick questions.
    const DEFAULT_LEVEL: &str = "low";

    pub fn new(variables: HashMap<String, String>) -> Self {
        Self { variables }
    }

    /// Workflow variable, `None` when unset or empty.
    pub fn variable(&self, name: &str) -> Option<&str> {
        self.variables
            .get(name)
            .map(String::as_str)
            .filter(|value| !value.is_empty())
    }

    /// # Errors
    /// When the configuration is invalid or the Claude Code call fails.
    pub fn ask(&self, system_prompt: &str, message: &str) -> anyhow::Result<String> {
        let request = ClaudeRequest {
            effort: EffortLevel::parse(self.variable("level").unwrap_or(Self::DEFAULT_LEVEL))?,
            model: self.variable("model").unwrap_or(Self::DEFAULT_MODEL).to_owned(),
            system_prompt: system_prompt.to_owned(),
        };

        ClaudeCliClient::locate(self.variable("claude_path"), self.variable("HOME")).ask(&request, message)
    }
}
