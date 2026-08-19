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
                // need to compare the overall arms not just checking if it's inserted or not
                // if lhs function is the same as rhs function then no need to override in the IndexMap
                if !keywords.contains_key(&keyword) {
                    let stripped_keyword = Keywords::strip(&keyword);
                    let parts: Vec<&str> = stripped_keyword.split(':').collect();
                    if parts.len() == 2 {
                        let parsed_func = match parts[1].trim() {
                            "read" => Self::Read,
                            _ => {
                                eprintln!(
                                    "\n{}: '{}' is not a valid function",
                                    "error".red(),
                                    parts[1].yellow()
                                );
                                return None;
                            }
                        };

                        if let Some((_key, val)) = found.get(parts[0]) {
                            match (val, parsed_func) {
                                (&Fns::None, _) => {
                                    found.insert(parts[0].to_string(), (keyword, parsed_func));
                                }
                                _ => continue,
                            }
                        } else {
                            found.insert(parts[0].to_string(), (keyword, parsed_func));
                        }
                    } else {
                        if found.contains_key(parts[0]) {
                            continue;
                        }
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
                    match Self::eval_json_filter(&keyword_name, json_data) {
                        Ok(resolved) => {
                            keywords.insert(keyword, resolved);
                        }
                        Err(e) => {
                            eprintln!(
                                "\n[{}] {}: {} ({})",
                                "WRN".yellow(),
                                "jq lookup failed".yellow(),
                                keyword_name.green(),
                                e
                            );
                            keywords.insert(keyword, String::new());
                        }
                    }
                    continue;
                }

                match function {
                    Self::Read => {
                        let value: String =
                            prompt(&keyword_name).unwrap_or_default();
                        keywords.insert(keyword.clone(), value.clone());
                        keywords.insert(final_keyword, value);
                    }
                    Self::None => {
                        eprintln!(
                            "\n[{}] {}: {}",
                            "WRN".yellow(),
                            "Value not found".yellow(),
                            keyword.green()
                        );
                        keywords.insert(keyword, String::new());
                    }
                }
            }
        }
    }

    pub fn find_and_resolve(
        txt: &str,
        keywords: &mut HashMap<String, String>,
        re: &Regex,
        json_data: &serde_json::Value,
        interactive: bool,
    ) -> Result<(), crate::Error> {
        if let Some(found) = Self::find(txt, keywords, re) {
            for (keyword_name, (keyword, function)) in found {
                let final_keyword = Self::remove_fn_name(&keyword, function);
                if keywords.contains_key(&final_keyword) {
                    keywords.insert(keyword, keywords[&final_keyword].clone());
                    continue;
                }

                if !json_data.is_null() && keyword_name.contains('.') {
                    match Self::eval_json_filter(&keyword_name, json_data) {
                        Ok(resolved) => {
                            keywords.insert(keyword, resolved);
                        }
                        Err(e) => {
                            return Err(crate::Error::JsonFilter(format!(
                                "{}: {}",
                                keyword_name, e
                            )));
                        }
                    }
                    continue;
                }

                match function {
                    Self::Read => {
                        if !interactive {
                            return Err(crate::Error::MissingVariable(keyword_name));
                        }
                        let value: String = prompt(&keyword_name)
                            .map_err(|e| crate::Error::Prompt(e.to_string()))?;
                        keywords.insert(keyword.clone(), value.clone());
                        keywords.insert(final_keyword, value);
                    }
                    Self::None => {
                        eprintln!(
                            "\n[{}] {}: {}",
                            "WRN".yellow(),
                            "Value not found".yellow(),
                            keyword.green()
                        );
                        keywords.insert(keyword, String::new());
                    }
                }
            }
        }
        Ok(())
    }

    pub fn eval_json_filter(
        filter_str: &str,
        json_data: &serde_json::Value,
    ) -> Result<String, String> {
        use jaq_interpret::FilterT;

        let mut defs = jaq_interpret::ParseCtx::new(Vec::new());

        let (f, errs) = jaq_parse::parse(filter_str, jaq_parse::main());
        if !errs.is_empty() {
            return Err(format!("Parse error: {:?}", errs));
        }
        let f = match f {
            Some(f) => defs.compile(f),
            None => return Err("Failed to parse filter".to_string()),
        };
        if !defs.errs.is_empty() {
            let err_msgs: Vec<String> = defs
                .errs
                .into_iter()
                .map(|(e, _span)| e.to_string())
                .collect();
            return Err(format!("Filter compilation failed: {}", err_msgs.join(", ")));
        }
        let val = jaq_interpret::Val::from(json_data.clone());
        let inputs = jaq_interpret::RcIter::new(core::iter::empty());
        let mut out = f.run((jaq_interpret::Ctx::new([], &inputs), val));
        if let Some(item) = out.next() {
            match item {
                Ok(val) => match val {
                    jaq_interpret::Val::Str(s) => Ok((*s).clone()),
                    other => Ok(other.to_string()),
                },
                Err(e) => Err(format!("{}", e)),
            }
        } else {
            Err("No output from filter".to_string())
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
    fn find_returns_none_for_read_followed_by_invalid_function() {
        let re = keyword_re();
        let keywords = HashMap::new();

        let found = Fns::find("{{$NAME:read}} then {{$NAME:write}}", &keywords, &re);
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

        assert_eq!(
            keywords.get("{{$NAME:read}}").map(String::as_str),
            Some("spark")
        );
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
