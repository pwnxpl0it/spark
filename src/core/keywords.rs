use crate::Keywords;
use chrono::{Datelike, Local, Utc};
use regex::Regex;
use std::sync::OnceLock;
use std::{collections::HashMap, env};

impl Keywords {
    pub fn from(name: &str, function: Option<&str>) -> String {
        if let Some(func) = function {
            format!("{{{{${}:{}}}}}", name, func)
        } else {
            format!("{{{{${}}}}}", name)
        }
    }

    pub fn strip(keyword: &str) -> String {
        keyword
            .trim_matches(|c| c == '{' || c == '$' || c == '}')
            .to_string()
    }

    pub fn init() -> HashMap<String, String> {
        let mut keywords = HashMap::new();

        if let Ok(home) = env::var("HOME") {
            keywords.insert(Self::from("HOME", None), home);
        }

        keywords.insert(Self::from("PROJECTNAME", None), String::new());

        if let Ok(current_dir) = env::current_dir() {
            if let Some(dir_name) = current_dir.file_name().and_then(|n| n.to_str()) {
                keywords.insert(Self::from("CURRENTDIR", None), dir_name.to_string());
            }
        }

        keywords.insert(Self::from("NOW_UTC", None), Utc::now().to_string());
        keywords.insert(Self::from("NOW", None), Local::now().to_string());
        keywords.insert(Self::from("YYYY", None), Local::now().year().to_string());
        keywords.insert(
            Self::from("YY", None),
            Local::now().format("%y").to_string(),
        );
        keywords.insert(Self::from("MM", None), Local::now().month().to_string());
        keywords.insert(Self::from("DD", None), Local::now().day().to_string());

        for (key, value) in env::vars() {
            keywords.insert(Self::from(&key, None), value);
        }

        keywords
    }

    /// Compiled `{{$...}}` matcher, built once per process.
    pub fn regex() -> &'static Regex {
        static RE: OnceLock<Regex> = OnceLock::new();
        RE.get_or_init(|| Regex::new(crate::templates::KEYWORDS_REGEX).expect("valid regex"))
    }

    /// Prefix of an escaped placeholder. `{{$$NAME}}` renders as the literal
    /// text `{{$NAME}}` and is never resolved or substituted.
    pub const ESCAPE_PREFIX: &'static str = "{{$$";

    /// Returns `true` if `token` is an escaped placeholder (`{{$$...}}`).
    pub fn is_escaped(token: &str) -> bool {
        token.starts_with(Self::ESCAPE_PREFIX)
    }

    /// Turns every `{{$` in `data` into `{{$$` so the text survives rendering
    /// unchanged. Used by `spark init` when capturing existing files.
    pub fn escape(data: &str) -> String {
        data.replace("{{$", Self::ESCAPE_PREFIX)
    }

    /// Turns every `{{$$` back into `{{$`. Run once, after all substitution.
    pub fn unescape(data: &str) -> String {
        data.replace(Self::ESCAPE_PREFIX, "{{$")
    }

    /// Replaces every `{{$...}}` token in `data` with its value from `keywords`
    /// in a single pass. Tokens without a value and escaped tokens are left
    /// untouched.
    pub fn replace_keywords(keywords: &HashMap<String, String>, data: &str) -> String {
        Self::regex()
            .replace_all(data, |caps: &regex::Captures| {
                let token = &caps[0];
                if Self::is_escaped(token) {
                    return token.to_string();
                }
                keywords
                    .get(token)
                    .cloned()
                    .unwrap_or_else(|| token.to_string())
            })
            .into_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::Keywords;
    use std::collections::HashMap;

    #[test]
    fn from_without_function() {
        assert_eq!(Keywords::from("TEST", None), "{{$TEST}}");
    }

    #[test]
    fn from_with_function() {
        assert_eq!(Keywords::from("USER", Some("read")), "{{$USER:read}}");
    }

    #[test]
    fn strip_removes_keyword_delimiters() {
        assert_eq!(Keywords::strip("{{$TEST}}"), "TEST");
        assert_eq!(Keywords::strip("{{$USER:read}}"), "USER:read");
    }

    #[test]
    fn init_includes_builtin_keywords() {
        let keywords = Keywords::init();
        assert!(keywords.contains_key("{{$PROJECTNAME}}"));
        assert!(keywords.contains_key("{{$NOW}}"));
        assert!(keywords.contains_key("{{$YYYY}}"));
        assert!(keywords.contains_key("{{$MM}}"));
        assert!(keywords.contains_key("{{$DD}}"));
        if std::env::var("HOME").is_ok() {
            assert!(keywords.contains_key("{{$HOME}}"));
        }
    }

    #[test]
    fn replace_keywords_substitutes_all_matches() {
        let mut map = HashMap::new();
        map.insert("{{$NAME}}".to_string(), "spark".to_string());
        map.insert("{{$AUTHOR}}".to_string(), "pwnxpl0it".to_string());

        let result =
            Keywords::replace_keywords(&map, "Project {{$NAME}} by {{$AUTHOR}} ({{$NAME}})");
        assert_eq!(result, "Project spark by pwnxpl0it (spark)");
    }

    #[test]
    fn replace_keywords_leaves_unknown_placeholders() {
        let map = HashMap::new();
        let input = "Hello {{$MISSING}}";
        assert_eq!(Keywords::replace_keywords(&map, input), input);
    }

    #[test]
    fn replace_keywords_is_single_pass() {
        // A value that itself looks like a placeholder must not be re-expanded.
        let mut map = HashMap::new();
        map.insert("{{$A}}".to_string(), "{{$B}}".to_string());
        map.insert("{{$B}}".to_string(), "leaked".to_string());
        assert_eq!(Keywords::replace_keywords(&map, "{{$A}}"), "{{$B}}");
    }

    #[test]
    fn escaped_tokens_are_not_replaced_and_unescape_restores_them() {
        let mut map = HashMap::new();
        map.insert("{{$NAME}}".to_string(), "spark".to_string());
        let out = Keywords::replace_keywords(&map, "{{$NAME}} {{$$NAME}}");
        assert_eq!(out, "spark {{$$NAME}}");
        assert_eq!(Keywords::unescape(&out), "spark {{$NAME}}");
        assert_eq!(Keywords::escape("a {{$X}} b"), "a {{$$X}} b");
    }
}
