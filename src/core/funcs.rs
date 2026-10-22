use crate::context::MissingPolicy;
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
            Self::Read(None) => write!(f, "read"),
            Self::Read(Some(label)) => write!(f, "read(\"{}\")", label),
            Self::None => write!(f, ""),
        }
    }
}

impl Fns {
    /// Parses the text after the `:` in a placeholder into a function.
    /// Accepts `read`, `read(Label)`, `read("Label")`, and `read('Label')`.
    pub fn parse(spec: &str) -> Option<Self> {
        let spec = spec.trim();
        if spec == "read" {
            return Some(Self::Read(None));
        }
        let inner = spec.strip_prefix("read(")?.strip_suffix(')')?.trim();
        let label = inner
            .strip_prefix('"')
            .and_then(|l| l.strip_suffix('"'))
            .or_else(|| inner.strip_prefix('\'').and_then(|l| l.strip_suffix('\'')))
            .unwrap_or(inner)
            .trim();
        Some(Self::Read(if label.is_empty() {
            None
        } else {
            Some(label.to_string())
        }))
    }

    /// Strips the `:function` suffix from a placeholder, giving the plain
    /// `{{$NAME}}` token.
    pub fn remove_fn_name(keyword: &str, _func_name: &Self) -> String {
        let stripped = Keywords::strip(keyword);
        let name = stripped
            .split_once(':')
            .map_or(stripped.as_str(), |(n, _)| n);
        Keywords::from(name.trim(), None)
    }

    pub fn find(
        txt: &str,
        keywords: &HashMap<String, String>,
        re: &Regex,
    ) -> crate::Result<IndexMap<String, (String, Self)>> {
        let mut found = IndexMap::new();
        for cap in re.captures_iter(txt) {
            if let Some(key_match) = cap.get(0) {
                let keyword = key_match.as_str().to_string();
                if Keywords::is_escaped(&keyword) {
                    continue;
                }
                // need to compare the overall arms not just checking if it's inserted or not
                // if lhs function is the same as rhs function then no need to override in the IndexMap
                if !keywords.contains_key(&keyword) {
                    let stripped_keyword = Keywords::strip(&keyword);
                    if let Some((name, spec)) = stripped_keyword.split_once(':') {
                        let name = name.trim();
                        let parsed_func =
                            Self::parse(spec).ok_or_else(|| crate::Error::InvalidFunction {
                                placeholder: keyword.clone(),
                                function: spec.trim().to_string(),
                            })?;

                        // A function form wins over a plain form of the same
                        // name; the first function form seen is kept.
                        match found.get(name) {
                            Some((_, Fns::None)) | None => {
                                found.insert(name.to_string(), (keyword, parsed_func));
                            }
                            _ => continue,
                        }
                    } else {
                        if found.contains_key(stripped_keyword.as_str()) {
                            continue;
                        }
                        found.insert(stripped_keyword.clone(), (keyword, Self::None));
                    }
                }
            }
        }
        Ok(found)
    }

    pub fn exec(func: Self, keyword_name: &str) -> Result<String, String> {
        match func {
            Self::Read(label) => prompt(label.as_deref().unwrap_or(keyword_name))
                .map_err(|_| "Failed to read input".to_string()),
            Self::None => Ok(keyword_name.to_string()),
        }
    }

    /// Legacy, always-interactive variant of [`Fns::find_and_resolve`].
    /// Errors are printed to stderr instead of being returned.
    pub fn find_and_exec(
        txt: &str,
        keywords: &mut HashMap<String, String>,
        re: &Regex,
        json_data: &serde_json::Value,
    ) {
        if let Err(e) =
            Self::find_and_resolve(txt, keywords, re, json_data, true, MissingPolicy::Empty)
        {
            eprintln!("\n{}: {}", "error".red(), e);
        }
    }

    pub fn find_and_resolve(
        txt: &str,
        keywords: &mut HashMap<String, String>,
        re: &Regex,
        json_data: &serde_json::Value,
        interactive: bool,
        on_missing: MissingPolicy,
    ) -> Result<(), crate::Error> {
        let found = Self::find(txt, keywords, re)?;
        {
            for (keyword_name, (keyword, function)) in found {
                let final_keyword = Self::remove_fn_name(&keyword, &function);
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
                    Self::Read(label) => {
                        if !interactive {
                            return Err(crate::Error::MissingVariable(keyword_name));
                        }
                        let value: String = prompt(label.as_deref().unwrap_or(&keyword_name))
                            .map_err(|e| crate::Error::Prompt(e.to_string()))?;
                        keywords.insert(keyword.clone(), value.clone());
                        keywords.insert(final_keyword, value);
                    }
                    Self::None => match on_missing {
                        MissingPolicy::Error => {
                            return Err(crate::Error::MissingVariable(keyword_name));
                        }
                        MissingPolicy::Keep => {
                            keywords.insert(keyword.clone(), keyword);
                        }
                        MissingPolicy::Empty => {
                            eprintln!(
                                "\n[{}] {}: {}",
                                "WRN".yellow(),
                                "Value not found".yellow(),
                                keyword.green()
                            );
                            keywords.insert(keyword, String::new());
                        }
                    },
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
            return Err(format!(
                "Filter compilation failed: {}",
                err_msgs.join(", ")
            ));
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
        let result = Fns::remove_fn_name("{{$TEST:read}}", &Fns::Read(None));
        assert_eq!(result, "{{$TEST}}");
        let result = Fns::remove_fn_name(
            "{{$TEST:read(\"Label: with colon\")}}",
            &Fns::Read(Some("Label: with colon".into())),
        );
        assert_eq!(result, "{{$TEST}}");
        assert_eq!(Fns::remove_fn_name("{{$PLAIN}}", &Fns::None), "{{$PLAIN}}");
    }

    #[test]
    fn display_formats_variants() {
        assert_eq!(Fns::Read(None).to_string(), "read");
        assert_eq!(Fns::Read(Some("Name".into())).to_string(), "read(\"Name\")");
        assert_eq!(Fns::None.to_string(), "");
    }

    #[test]
    fn parse_accepts_read_with_optional_label() {
        assert_eq!(Fns::parse("read"), Some(Fns::Read(None)));
        assert_eq!(Fns::parse(" read "), Some(Fns::Read(None)));
        assert_eq!(Fns::parse("read()"), Some(Fns::Read(None)));
        assert_eq!(
            Fns::parse("read(\"Project name\")"),
            Some(Fns::Read(Some("Project name".into())))
        );
        assert_eq!(Fns::parse("read('x')"), Some(Fns::Read(Some("x".into()))));
        assert_eq!(
            Fns::parse("read(bare)"),
            Some(Fns::Read(Some("bare".into())))
        );
        assert_eq!(Fns::parse("upper"), None);
        assert_eq!(Fns::parse("read(unclosed"), None);
    }

    #[test]
    fn find_parses_labelled_read_with_colon_in_label() {
        let re = keyword_re();
        let keywords = HashMap::new();
        let found = Fns::find("{{$NAME:read(\"Name: please\")}}", &keywords, &re).unwrap();
        let (keyword, func) = found.get("NAME").unwrap();
        assert_eq!(keyword, "{{$NAME:read(\"Name: please\")}}");
        assert_eq!(func, &Fns::Read(Some("Name: please".into())));
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
        assert!(matches!(func, Fns::Read(None)));
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
    fn find_errors_for_invalid_function() {
        let re = keyword_re();
        let keywords = HashMap::new();

        let err = Fns::find("Bad {{$NAME:upper}}", &keywords, &re).unwrap_err();
        assert!(matches!(
            err,
            crate::Error::InvalidFunction { ref function, .. } if function == "upper"
        ));
    }

    #[test]
    fn find_errors_for_read_followed_by_invalid_function() {
        let re = keyword_re();
        let keywords = HashMap::new();

        let found = Fns::find("{{$NAME:read}} then {{$NAME:write}}", &keywords, &re);
        assert!(found.is_err());
    }

    #[test]
    fn find_and_resolve_propagates_invalid_function() {
        let re = keyword_re();
        let mut keywords = HashMap::new();
        let err = Fns::find_and_resolve(
            "{{$A:upper}}",
            &mut keywords,
            &re,
            &serde_json::Value::Null,
            false,
            MissingPolicy::Empty,
        )
        .unwrap_err();
        assert!(matches!(err, crate::Error::InvalidFunction { .. }));
    }

    fn resolve_missing(policy: MissingPolicy) -> crate::Result<HashMap<String, String>> {
        let re = keyword_re();
        let mut keywords = HashMap::new();
        Fns::find_and_resolve(
            "{{$NOPE}}",
            &mut keywords,
            &re,
            &serde_json::Value::Null,
            false,
            policy,
        )?;
        Ok(keywords)
    }

    #[test]
    fn missing_policy_empty_inserts_empty_string() {
        let k = resolve_missing(MissingPolicy::Empty).unwrap();
        assert_eq!(k.get("{{$NOPE}}").map(String::as_str), Some(""));
    }

    #[test]
    fn missing_policy_keep_preserves_token() {
        let k = resolve_missing(MissingPolicy::Keep).unwrap();
        assert_eq!(k.get("{{$NOPE}}").map(String::as_str), Some("{{$NOPE}}"));
    }

    #[test]
    fn missing_policy_error_fails() {
        let err = resolve_missing(MissingPolicy::Error).unwrap_err();
        assert!(matches!(err, crate::Error::MissingVariable(v) if v == "NOPE"));
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

    #[test]
    fn find_ignores_escaped_placeholders() {
        let re = keyword_re();
        let keywords = HashMap::new();
        let found = Fns::find("{{$$NAME:read}} {{$$OTHER}}", &keywords, &re).unwrap();
        assert!(found.is_empty());
    }
}
