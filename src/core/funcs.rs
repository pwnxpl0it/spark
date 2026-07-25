use crate::Fns;
use crate::Keywords;
use colored::*;
use indexmap::IndexMap;
use promptly::prompt;
use regex::Regex;
use std::collections::HashMap;

impl std::fmt::Display for Fns {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Read => write!(f, "read"),
            Self::None => write!(f, ""),
        }
    }
}

impl Fns {
    pub fn remove_fn_name(keyword: &str, func_name: Self) -> String {
        keyword.replace(&format!(":{}", func_name), "")
    }

    pub fn find(
        txt: &str,
        keywords: &HashMap<String, String>,
        re: &Regex,
    ) -> Option<IndexMap<String, (String, Self)>> {
        let mut found = IndexMap::new();
        for cap in re.captures_iter(txt) {
            if let Some(key_match) = cap.get(0) {
                let keyword = key_match.as_str().to_string();
                if !keywords.contains_key(&keyword) {
                    let stripped_keyword = Keywords::strip(&keyword);
                    let parts: Vec<&str> = stripped_keyword.split(':').collect();
                    if parts.len() == 2 {
                        match parts[1].trim() {
                            "read" => {
                                found.insert(parts[0].to_string(), (keyword, Self::Read));
                            }
                            _ => {
                                eprintln!(
                                    "\n{}: '{}' is not a valid function",
                                    "error".red(),
                                    parts[1].yellow()
                                );
                                return None;
                            }
                        }
                    } else {
                        found.insert(stripped_keyword.clone(), (keyword, Self::None));
                    }
                }
            }
        }
        Some(found)
    }

    pub fn exec(func: Self, keyword_name: &str) -> Result<String, String> {
        match func {
            Self::Read => prompt(keyword_name).map_err(|_| "Failed to read input".to_string()),
            Self::None => Ok(keyword_name.to_string()),
        }
    }

    pub fn find_and_exec(
        txt: &str,
        keywords: &mut HashMap<String, String>,
        re: &Regex,
        json_data: &serde_json::Value,
    ) {
        if let Some(found) = Self::find(txt, keywords, re) {
            for (keyword_name, (keyword, function)) in found {
                let final_keyword = Self::remove_fn_name(&keyword, function);
                if keywords.contains_key(&final_keyword) {
                    keywords.insert(keyword, keywords[&final_keyword].clone());
                    continue;
                }

                if !json_data.is_null() && keyword_name.contains('.') {
                    if let Ok(value) = jq_rs::run(&keyword_name, &json_data.to_string()) {
                        // Remove quotes from the value
                        keywords.insert(keyword, value.replace('"', ""));
                    }
                    continue;
                }
                if let Ok(value) = Self::exec(function, &keyword_name) {
                    match function {
                        Self::None => {
                            eprintln!(
                                "\n[{}] {}: {}",
                                "WRN".yellow(),
                                "Value not found".yellow(),
                                keyword.green()
                            );
                            keywords.insert(keyword, String::new());
                        }
                        _ => {
                            keywords.insert(keyword.clone(), value.clone());
                            keywords.insert(final_keyword, value);
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::templates::KEYWORDS_REGEX;
    use std::collections::HashMap;

    fn keyword_re() -> Regex {
        Regex::new(KEYWORDS_REGEX).unwrap()
    }

    #[test]
    fn remove_fn_name_strips_read_suffix() {
        let result = Fns::remove_fn_name("{{$TEST:read}}", Fns::Read);
        assert_eq!(result, "{{$TEST}}");
    }

    #[test]
    fn remove_fn_name_with_none_leaves_colon_suffix() {
        // Display for None is empty, so the replaced pattern is just ":"
        let result = Fns::remove_fn_name("{{$TEST:read}}", Fns::None);
        assert_eq!(result, "{{$TESTread}}");
    }

    #[test]
    fn display_formats_variants() {
        assert_eq!(Fns::Read.to_string(), "read");
        assert_eq!(Fns::None.to_string(), "");
    }

    #[test]
    fn find_skips_keywords_already_in_map() {
        let re = keyword_re();
        let mut keywords = HashMap::new();
        keywords.insert("{{$TEST}}".to_string(), "value".to_string());

        let found = Fns::find("Hello {{$TEST}} world", &keywords, &re).unwrap();
        assert!(found.is_empty());
    }

    #[test]
    fn find_detects_read_function() {
        let re = keyword_re();
        let keywords = HashMap::new();

        let found = Fns::find("Give me {{$NAME:read}}", &keywords, &re).unwrap();
        assert_eq!(found.len(), 1);

        let (keyword, func) = found.get("NAME").expect("NAME should be found");
        assert_eq!(keyword, "{{$NAME:read}}");
        assert!(matches!(func, Fns::Read));
    }

    #[test]
    fn find_detects_plain_keyword_as_none() {
        let re = keyword_re();
        let keywords = HashMap::new();

        let found = Fns::find("Value {{$AUTHOR}}", &keywords, &re).unwrap();
        assert_eq!(found.len(), 1);

        let (keyword, func) = found.get("AUTHOR").expect("AUTHOR should be found");
        assert_eq!(keyword, "{{$AUTHOR}}");
        assert!(matches!(func, Fns::None));
    }

    #[test]
    fn find_returns_none_for_invalid_function() {
        let re = keyword_re();
        let keywords = HashMap::new();

        let found = Fns::find("Bad {{$NAME:upper}}", &keywords, &re);
        assert!(found.is_none());
    }

    #[test]
    fn exec_none_returns_keyword_name() {
        let res = Fns::exec(Fns::None, "hello").unwrap();
        assert_eq!(res, "hello");
    }

    #[test]
    fn find_and_exec_copies_existing_base_keyword() {
        let re = keyword_re();
        let mut keywords = HashMap::new();
        keywords.insert("{{$NAME}}".to_string(), "spark".to_string());

        Fns::find_and_exec(
            "Hello {{$NAME:read}}",
            &mut keywords,
            &re,
            &serde_json::Value::Null,
        );

        assert_eq!(keywords.get("{{$NAME:read}}").map(String::as_str), Some("spark"));
    }

    #[test]
    fn find_and_exec_resolves_json_path() {
        let re = keyword_re();
        let mut keywords = HashMap::new();
        let json_data = serde_json::json!({ "user": { "name": "spark" } });

        // jq filters need a leading '.', so the placeholder is {{$.user.name}}
        Fns::find_and_exec("Hello {{$.user.name}}", &mut keywords, &re, &json_data);

        assert_eq!(
            keywords
                .get("{{$.user.name}}")
                .map(|v| v.trim())
                .unwrap_or_default(),
            "spark"
        );
    }
}
