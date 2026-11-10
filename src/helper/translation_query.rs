/// Translate command input: an optional target language prefix, then the text.
/// `:xx` accepts any two-letter code; a bare `xx` only a known one, so text starting
/// with a short word ("Le chat", "Un jour") is not mistaken for a language.
#[derive(Debug, Eq, PartialEq)]
pub struct TranslationQuery {
    pub lang: String,
    /// Full language name when known, the code otherwise.
    pub language_name: String,
    pub text: String,
}

impl TranslationQuery {
    pub fn new(query: &str, default_lang: &str) -> Self {
        let (lang, text) = match Self::split_prefix(query) {
            Some((explicit, lang, text)) if explicit || Self::name_of(&lang).is_some() => (lang, text),
            _ => (default_lang.to_owned(), query),
        };

        Self {
            language_name: Self::name_of(&lang).map_or_else(|| lang.clone(), str::to_owned),
            lang,
            text: text.to_owned(),
        }
    }

    /// Whether the colon was present, the lowercased code and the text, for `[:]xx <text>`.
    fn split_prefix(query: &str) -> Option<(bool, String, &str)> {
        let (explicit, rest) = query.strip_prefix(':').map_or((false, query), |rest| (true, rest));
        let code = rest
            .get(..2)
            .filter(|code| code.bytes().all(|byte| byte.is_ascii_alphabetic()))?;
        let after = &rest[2..];

        if !after.starts_with(char::is_whitespace) {
            return None;
        }

        let text = after.trim_start();

        (!text.is_empty()).then(|| (explicit, code.to_ascii_lowercase(), text))
    }

    fn name_of(lang: &str) -> Option<&'static str> {
        match lang {
            "de" => Some("German"),
            "en" => Some("English"),
            "es" => Some("Spanish"),
            "fr" => Some("French"),
            "it" => Some("Italian"),
            "ja" => Some("Japanese"),
            "ko" => Some("Korean"),
            "nl" => Some("Dutch"),
            "pl" => Some("Polish"),
            "pt" => Some("Portuguese"),
            "ru" => Some("Russian"),
            "zh" => Some("Chinese"),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TranslationQuery;

    fn parse(query: &str) -> TranslationQuery {
        TranslationQuery::new(query, "en")
    }

    #[test]
    fn uses_the_default_language_without_prefix() {
        let query = parse("Hello world");

        assert_eq!(query.lang, "en");
        assert_eq!(query.language_name, "English");
        assert_eq!(query.text, "Hello world");
    }

    #[test]
    fn extracts_the_language_from_a_colon_prefix() {
        let query = parse(":fr Hello world");

        assert_eq!(query.lang, "fr");
        assert_eq!(query.language_name, "French");
        assert_eq!(query.text, "Hello world");
    }

    #[test]
    fn extracts_a_known_language_without_colon() {
        let query = parse("fr Hello world");

        assert_eq!(query.lang, "fr");
        assert_eq!(query.text, "Hello world");
    }

    #[test]
    fn keeps_a_leading_short_word_that_is_not_a_known_language() {
        for text in ["Le chat dort", "La maison", "Un jour", "Il pleut"] {
            let query = parse(text);

            assert_eq!(query.lang, "en", "{text}");
            assert_eq!(query.text, text);
        }
    }

    #[test]
    fn accepts_an_unknown_code_made_explicit_by_the_colon() {
        let query = parse(":sv Hello world");

        assert_eq!(query.lang, "sv");
        assert_eq!(query.language_name, "sv");
        assert_eq!(query.text, "Hello world");
    }

    #[test]
    fn lowercases_the_language_code() {
        assert_eq!(parse(":FR Hello world").lang, "fr");
    }

    #[test]
    fn preserves_multiline_text() {
        let query = parse(":de Line 1\nLine 2");

        assert_eq!(query.lang, "de");
        assert_eq!(query.text, "Line 1\nLine 2");
    }

    #[test]
    fn ignores_codes_longer_than_two_letters() {
        let query = parse(":abc Hello world");

        assert_eq!(query.lang, "en");
        assert_eq!(query.text, ":abc Hello world");
    }
}
