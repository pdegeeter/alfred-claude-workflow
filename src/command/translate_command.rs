use serde_json::Value;

use super::CommandRunner;
use crate::helper::{AlfredOutput, TranslationQuery};

/// `translate`: translation to `default_lang` or to the language given as prefix.
pub struct TranslateCommand;

impl TranslateCommand {
    const DEFAULT_LANG: &str = "en";
    const SYSTEM_PROMPT: &str = "You are a silent translator.
The user message contains ONLY text to translate to a target language.
ABSOLUTE rules:
- Return ONLY the translated text, nothing else
- NEVER add explanations, comments, preambles or questions
- Preserve punctuation and formatting
- If the text is already in the target language, return it as-is";

    /// # Errors
    /// When the model cannot be called.
    pub fn run(runner: &CommandRunner, query: &str) -> anyhow::Result<Value> {
        if query.is_empty() {
            return Ok(AlfredOutput::error("Please enter text to translate"));
        }

        let translation = TranslationQuery::new(query, runner.variable("default_lang").unwrap_or(Self::DEFAULT_LANG));
        let message = format!("Translate to {}:\n{}", translation.language_name, translation.text);

        Ok(AlfredOutput::response(&runner.ask(Self::SYSTEM_PROMPT, &message)?))
    }
}
