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
                            keywords.insert(Self::remove_fn_name(&keyword, function), value);
                        }
                    }
                }
            }
        }
    }
}
