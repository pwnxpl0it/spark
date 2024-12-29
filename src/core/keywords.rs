use crate::Keywords;
use chrono::Datelike;
use std::{collections::HashMap, env};

pub const KEYWORDS_FORMAT: &str = "{{$%s:f}}";

impl Keywords {
    pub fn from(name: String, function: Option<String>) -> String {
        if let Some(function) = function {
            KEYWORDS_FORMAT
                .to_string()
                .replace("%s", &name)
                .replace('f', &function)
        } else {
            KEYWORDS_FORMAT
                .to_string()
                .replace("%s", &name)
                .replace(":f", "")
        }
    }

    pub fn strip(keyword: &String) -> String {
        keyword.replace("{{$", "").replace("}}", "")
    }

    pub fn init() -> HashMap<String, String> {
        let mut keywords = HashMap::new();
        keywords.insert(
            Self::from(String::from("HOME"), None),
            env::var("HOME").unwrap(),
        );
        keywords.insert(
            Self::from(String::from("PROJECTNAME"), None),
            "".to_string(),
        );
        keywords.insert(
            Self::from(String::from("CURRENTDIR"), None),
            env::current_dir()
                .unwrap()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string(),
        );

        keywords.insert(
            Self::from(String::from("NOW_UTC"), None),
            chrono::Utc::now().to_string(),
        );

        keywords.insert(
            Self::from(String::from("NOW"), None),
            chrono::Local::now().to_string(),
        );

        keywords.insert(
            Self::from(String::from("YYYY"), None),
            chrono::Local::now().year().to_string(),
        );

        keywords.insert(
            Self::from(String::from("YY"), None),
            chrono::Local::now().format("%y").to_string(),
        );

        keywords.insert(
            Self::from(String::from("MM"), None),
            chrono::Local::now().month().to_string(),
        );

        keywords.insert(
            Self::from(String::from("DD"), None),
            chrono::Local::now().day().to_string(),
        );

        keywords
    }

    pub fn replace_keywords(keywords: &HashMap<String, String>, mut data: String) -> String {
        for (key, value) in keywords.iter() {
            data = data.replace(key, value);
        }
        data
    }
}
