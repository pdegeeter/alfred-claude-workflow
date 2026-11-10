use serde_json::{Value, json};

/// Alfred Script Filter JSON responses.
pub struct AlfredOutput;

impl AlfredOutput {
    const TITLE_MAX_CHARS: usize = 100;

    /// Single result carrying the response text, viewable, copyable or pastable.
    pub fn response(text: &str) -> Value {
        json!({
            "items": [{
                "arg": text,
                "mods": {
                    "alt": { "arg": text, "subtitle": "Paste response", "valid": true },
                    "cmd": { "arg": text, "subtitle": "Copy response to clipboard", "valid": true },
                },
                "subtitle": "↵ View in Text View | ⌘↵ Copy | ⌥↵ Paste",
                "title": Self::title_of(text),
                "uid": "claude-response",
                "valid": true,
            }],
            "variables": { "response": text },
        })
    }

    pub fn error(message: &str) -> Value {
        json!({
            "items": [{
                "subtitle": "Press Enter to dismiss",
                "title": message,
                "uid": "claude-error",
                "valid": false,
            }],
        })
    }

    fn title_of(text: &str) -> String {
        let first_line = text.lines().next().unwrap_or_default();

        if first_line.chars().count() > Self::TITLE_MAX_CHARS {
            let truncated: String = first_line.chars().take(Self::TITLE_MAX_CHARS).collect();

            return format!("{truncated}...");
        }

        first_line.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::AlfredOutput;

    #[test]
    fn response_has_a_single_item_titled_with_the_first_line() {
        let output = AlfredOutput::response("Hello World\nSecond line");

        assert_eq!(output["items"].as_array().unwrap().len(), 1);
        assert_eq!(output["items"][0]["title"], "Hello World");
    }

    #[test]
    fn response_truncates_long_titles() {
        let output = AlfredOutput::response(&"é".repeat(150));
        let title = output["items"][0]["title"].as_str().unwrap();

        assert_eq!(title.chars().count(), 103);
        assert!(title.ends_with("..."));
    }

    #[test]
    fn response_passes_the_text_to_the_next_steps() {
        let output = AlfredOutput::response("Hello");

        assert_eq!(output["items"][0]["arg"], "Hello");
        assert_eq!(output["items"][0]["mods"]["cmd"]["arg"], "Hello");
        assert_eq!(output["items"][0]["mods"]["alt"]["arg"], "Hello");
        assert_eq!(output["variables"]["response"], "Hello");
    }

    #[test]
    fn error_is_not_actionable() {
        let output = AlfredOutput::error("Error message");

        assert_eq!(output["items"][0]["title"], "Error message");
        assert_eq!(output["items"][0]["valid"], false);
    }
}
