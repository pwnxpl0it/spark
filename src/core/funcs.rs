use crate::{Fns, Keywords};
use colored::*;
use core::fmt;
use indexmap::IndexMap;
use promptly::prompt;
use regex::Regex;
use std::collections::HashMap;

impl std::fmt::Display for Fns {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Read => write!(f, "read"),
            //Self::Env => write!(f, "env"),
            Self::None => write!(f, ""),
        }
    }
}

impl Fns {
    pub fn remove_fn_name(keyword: String, func_name: Self) -> String {
        keyword.replace(&format!(":{}", func_name), "")
    }

    /// This method finds `functions` in a string based on a Regex pattern that matches keywords
    pub fn find(
        txt: &str,
        keywords: &HashMap<String, String>,
        re: &Regex,
    ) -> Option<IndexMap<String, (String, Self)>> {
        let mut found = IndexMap::new();

        for key in re.captures_iter(&txt) {
            if let Some(key) = key.get(0) {
                let keyword = key.as_str().to_string();

                if !keywords.contains_key(&keyword) {
                    let new_keyword = Keywords::strip(&keyword).trim().to_string();
                    let data = new_keyword.split(':').collect::<Vec<_>>();

                    if data.len() == 2 {
                        let keyword_name = Keywords::strip(&data[0].to_string());
                        let func = data[1].trim();

                        match func {
                            "read" => {
                                found.insert(keyword_name, (keyword, Self::Read));
                            }
                            //"env" => {
                                //found.insert(keyword_name, (keyword, Self::Env));
                            //}
                            _ => {
                                eprintln!(
                                    "\n{}: '{}' is not a valid function",
                                    "error".to_string().red(),
                                    func.yellow()
                                );
                                return None;
                            }
                        }
                    } else {
                        let keyword_name = Keywords::strip(&keyword);
                        found.insert(keyword_name, (keyword, Self::None));
                        continue;
                    }
                }
            }
        }

        Some(found)
    }

    /// This method executes allowed functions such as: read,env
    pub fn exec(func: Self, keyword_name: String) -> Result<String, String> {
        match func {
            Self::Read => Ok(prompt(keyword_name).unwrap()),
            //Self::Env => Ok(Self::env(keyword_name)),
            Self::None => Ok(Keywords::from(keyword_name, None)),
        }
    }

    /// NOTE: Why do I need this when I can access it directly ?
    /// This function reads from environment variables and returns the value as a string
    //fn env(name: String) -> String {
        //env::var(name).unwrap()
    //}

    pub fn find_and_exec(
        txt: &str,
        keywords: &mut HashMap<String, String>,
        re: &Regex,
        json_data: &serde_json::Value,
    ) {
        if let Some(found) = Self::find(txt, &keywords, &re) {
            for (keyword_name, (keyword, function)) in found {
                //HACK: Just a bit of optimization, if the json_data is null then it doesn't make sense to run jq
                // because doing so is every expensive and here we are dealing with dynamic queries
                if !json_data.is_null() && keyword_name.contains(".") {
                    if let Ok(value) = jq_rs::run(&keyword_name, &json_data.to_string()) {
                        keywords.insert(keyword, value.replace("\"", ""));
                    }
                    continue;
                }

                if let Ok(value) = Self::exec(function, keyword_name) {
                    match function {
                        Self::None => {
                            eprintln!(
                                "\n[{}] {}: {}",
                                "WRN".yellow(),
                                "Value not found".yellow(),
                                keyword.green()
                            );
                            keywords.insert(keyword, "".to_string());
                        }
                        _ => {
                            keywords.insert(keyword.clone(), value.clone());
                            keywords.insert(Self::remove_fn_name(keyword, function), value);
                        }
                    }
                }
            }
        }
    }
}
