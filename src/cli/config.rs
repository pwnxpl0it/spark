use colored::Colorize;
use spark::Keywords;
use spark::Template;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use toml::Value;

#[derive(Debug, Clone)]
pub struct Config {
    pub path: String,
    pub templates_path: String,
}

impl Config {
    pub fn new(path: &str) -> Self {
        let config_path = shellexpand::tilde(path).to_string();
        let config_dir = Path::new(path).parent().unwrap();

        let templates = Path::new(config_dir).join("templates");

        Config {
            path: config_path,
            templates_path: shellexpand::tilde(&templates.to_str().unwrap()).to_string(),
        }
    }

    pub fn init(self, keywords: &mut HashMap<String, String>) {
        // this sample is just a template that create config.toml and the new.toml template for the
        // first time, Now something maybe confusing is the "initPJNAME" wtf is it ?
        // That's just a way to workaround auto replacing PROJECTNAME in templates
        let conf_template = r#"
[[files]]
path = 'TEMPLATES_PATH/new.toml'
content = '''
[info]
name = "Spark Template"
description = "A Template for making a template"
author = "Mohamed Tarek @pwnxpl0it"

[[files]]
path="TEMPLATES_PATH/initPJNAME.toml"
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
path = 'CONFIGPATH'
content = '''
[Keywords]
'''
            "#
        .replace("CONFIGPATH", &self.path)
        .replace("TEMPLATES_PATH", &self.templates_path);

        let template: Template = toml::from_str(&conf_template).unwrap();

        Template::extract(template, &mut keywords.clone());
    }

    pub fn get_keywords(self) -> HashMap<String, String> {
        let mut keywords = HashMap::new();
        if let Ok(toml_str) = fs::read_to_string(&self.path) {
            let toml_val: Value = toml::from_str(&toml_str).unwrap();

            let keywords_table = toml_val.get("Keywords").unwrap().as_table().unwrap();

            for (key, value) in keywords_table.iter() {
                let value_str = match value {
                    Value::String(s) => s.clone(),
                    _ => value.to_string(),
                };
                keywords.insert(Keywords::from(key.to_string(), None), value_str);
            }
        } else {
            eprintln!(
                "\n[{}] {}\n",
                "INFO".bold().blue(),
                "Looks like it's your first time running spark, creating config files and templates for you".green()
            );
            Self::init(self, &mut keywords);
            println!("");
        }

        keywords
    }
}
