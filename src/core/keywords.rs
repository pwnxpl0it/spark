use crate::Keywords;
use chrono::{Datelike, Local, Utc};
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

    pub fn replace_keywords(keywords: &HashMap<String, String>, data: &str) -> String {
        let mut output = data.to_string();
        for (key, value) in keywords.iter() {
            output = output.replace(key, value);
        }
        output
    }

}
