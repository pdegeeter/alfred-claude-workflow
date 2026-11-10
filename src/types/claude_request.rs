use super::EffortLevel;

/// One single-turn question to Claude.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClaudeRequest {
    pub effort: EffortLevel,
    /// Model alias (`haiku`, `sonnet`, `opus`) or full model name, as `claude --model` accepts it.
    pub model: String,
    /// Replaces the Claude Code system prompt, written for coding sessions.
    pub system_prompt: String,
}
