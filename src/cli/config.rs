use crate::Template;
use colored::Colorize;
use std::{collections::HashMap, fs, path::Path};
use toml::Value;

#[derive(Debug, Clone)]
pub struct Config {
    pub path: String,
    pub templates_path: String,
}

impl Config {
    pub fn new(path: &str) -> Self {
        let config_path = shellexpand::tilde(path).to_string();
        let config_dir = Path::new(&config_path)
            .parent()
            .unwrap_or_else(|| Path::new("."));
        let templates = config_dir.join("templates");

        Self {
            path: config_path,
            templates_path: shellexpand::tilde(templates.to_str().unwrap()).to_string(),
        }
    }

    pub fn init(self) {
        // "initPJNAME" wtf is it ?
        // That's just a way to workaround auto replacing PROJECTNAME in templates
        let conf_template = r#"
[[files]]
path = '{{$TEMPLATES_PATH}}/new.toml'
content = '''
[info]
name = "Spark Template"
description = "A Template for making a template"
author = "Mohamed Tarek @pwnxpl0it"

[[files]]
path="{{$TEMPLATES_PATH}}/initPJNAME.toml"
content="""
[info]
name = "initPJNAME"
description = ""
author = ""

[[files]]
path=""
content=\"\"\"

\"\"\"
"""
'''

[[files]]
path = '{{$CONFIGPATH}}'
content = '''
[Keywords]
'''
            "#;

        let mut keywords: HashMap<String, String> = HashMap::new();
        keywords.insert("{{$CONFIGPATH}}".to_string(), self.path);
        keywords.insert("{{$TEMPLATES_PATH}}".to_string(), self.templates_path);

        let mut template: Template = toml::from_str(conf_template).unwrap();

        Template::extract(&mut template, &mut keywords).unwrap();
    }

    pub fn get_keywords(&self) -> HashMap<String, String> {
        let mut keywords = HashMap::new();

        if let Ok(toml_str) = fs::read_to_string(&self.path) {
            if let Ok(toml_val) = toml::from_str::<Value>(&toml_str) {
                if let Some(keywords_table) = toml_val.get("Keywords").and_then(|v| v.as_table()) {
                    for (key, value) in keywords_table.iter() {
                        let value_str = value.as_str().unwrap_or(&value.to_string()).to_string();
                        keywords.insert(format!("{{${}}}", key), value_str);
                    }
                }
            }
        } else {
            eprintln!(
                "\n[{}] Creating config files and templates for first-time setup...",
                "INFO".bold().blue()
            );
            self.clone().init();
        }

        keywords
    }
}
