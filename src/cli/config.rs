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

#[cfg(test)]
mod tests {
    use super::Config;
    use std::fs;

    #[test]
    fn new_expands_tilde_and_derives_templates_path() {
        let cfg = Config::new("~/.config/spark/config.toml");
        let expected_path = shellexpand::tilde("~/.config/spark/config.toml").to_string();
        let expected_templates = shellexpand::tilde("~/.config/spark/templates").to_string();

        assert_eq!(cfg.path, expected_path);
        assert_eq!(cfg.templates_path, expected_templates);
    }

    #[test]
    fn new_with_absolute_path() {
        let cfg = Config::new("/tmp/spark_cfg/config.toml");
        assert_eq!(cfg.path, "/tmp/spark_cfg/config.toml");
        assert_eq!(cfg.templates_path, "/tmp/spark_cfg/templates");
    }

    #[test]
    fn get_keywords_reads_keywords_section() {
        let dir = std::env::temp_dir().join("spark_test_config_keywords");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let config_path = dir.join("config.toml");
        fs::write(
            &config_path,
            r#"
[Keywords]
AUTHOR = "pwnxpl0it"
GITHUB = "https://github.com/pwnxpl0it"
"#,
        )
        .unwrap();

        let cfg = Config::new(&config_path.to_string_lossy());
        let keywords = cfg.get_keywords();

        assert_eq!(
            keywords.get("{$AUTHOR}").map(String::as_str),
            Some("pwnxpl0it")
        );
        assert_eq!(
            keywords.get("{$GITHUB}").map(String::as_str),
            Some("https://github.com/pwnxpl0it")
        );

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn get_keywords_returns_empty_when_section_missing() {
        let dir = std::env::temp_dir().join("spark_test_config_empty");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let config_path = dir.join("config.toml");
        fs::write(&config_path, "").unwrap();

        let cfg = Config::new(&config_path.to_string_lossy());
        let keywords = cfg.get_keywords();
        assert!(keywords.is_empty());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn init_creates_config_file() {
        let dir = std::env::temp_dir().join("spark_test_config_init");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let config_path = dir.join("config.toml");
        let cfg = Config::new(&config_path.to_string_lossy());
        cfg.init();

        assert!(config_path.exists());
        let content = fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("[Keywords]"));

        let _ = fs::remove_dir_all(&dir);
    }
}
