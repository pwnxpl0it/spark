use crate::utils::*;
use colored::Colorize;
use promptly::prompt;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::Path};
pub mod file;
pub mod options;
use crate::*;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Options {
    pub git: bool,
    pub use_liquid: Option<bool>,
    pub json_data: Option<serde_json::Value>,
    pub project_root: String,
}

pub const KEYWORDS_REGEX: &str = r"\{\{\$.*?\}\}";

impl Default for Template {
    fn default() -> Self {
        Self {
            options: Some(Options::default()),
            info: None,
            files: None,
        }
    }
}

impl Template {
    pub fn set_info(&mut self, info: Information) {
        self.info = Some(info);
    }

    pub fn set_files(&mut self, files: Vec<File>) {
        self.files = Some(files);
    }

    pub fn set_options(&mut self, options: Options) {
        self.options = Some(options);
    }

    pub fn dump_options(&mut self) -> Option<Options> {
        self.options.as_ref()?;
        Some(self.options.clone().unwrap())
    }

    pub fn generate(dest: &str) {
        let mut files: Vec<File> = Vec::new();

        list_files(Path::new("./")).iter().for_each(|file| {
            //TODO: Add more to ignore list maybe adding a --ignore flag will be good
            if !file.contains(".git") {
                let file = File::from(file.to_string().replace("./", ""), {
                    match fs::read_to_string(file) {
                        Ok(content) => content,
                        Err(e) => panic!("{}:{}", file.red().bold(), e),
                    }
                });
                files.push(file); // Push to Files Vector
            }
        });

        let template = Template {
            info: None,
            files: Some(files),
            options: None,
        };

        let toml_string = toml::to_string_pretty(&template).expect("Failed to create toml string");
        fs::write(dest, toml_string).unwrap();
    }

    pub fn liquify(string: &str) -> String {
        let parser = liquid::ParserBuilder::with_stdlib().build().unwrap();
        let empty_globals = liquid::Object::new();

        match parser.parse(string) {
            Ok(template) => template.render(&empty_globals).unwrap(),
            Err(e) => {
                eprintln!("{}: parsing template: {}", "error".red().bold(), e);
                String::new()
            }
        }
    }

    fn handle_project_name(
        keywords: &mut HashMap<String, String>,
        options: &mut Options,
        file: &File,
    ) -> Result<String, String> {
        let trimmed_content = file.content.trim();
        let trimmed_path = file.path.trim();
        let trimmed_project_root = options.project_root.trim();

        if trimmed_content.contains("{{$PROJECTNAME}}")
            || trimmed_path.contains("{{$PROJECTNAME}}")
            || trimmed_project_root.contains("{{$PROJECTNAME}}")
        {
            let project_name: String = prompt("Project name")
                .map_err(|_| format!("{}", "Project name not set.".red().bold()))?;

            keywords.insert("{{$PROJECTNAME}}".to_string(), project_name.clone());
            options.set_project_root(&project_name);
            Ok(project_name)
        } else {
            Ok(String::new())
        }
    }

    pub fn extract(mut self, keywords: &mut HashMap<String, String>) {
        let re = Regex::new(KEYWORDS_REGEX).expect("Invalid keywords regex");
        let mut options = self.dump_options().unwrap_or_default();
        let json_data = options.json_data.clone().unwrap_or(serde_json::Value::Null);
        let mut project = String::new();

        self.files
            .expect("No files table")
            .into_iter()
            .for_each(|file| {
                Fns::find_and_exec(&file.content, keywords, &re, &json_data);
                Fns::find_and_exec(&file.path, keywords, &re, &json_data);

                if project.is_empty() {
                    project = match Self::handle_project_name(keywords, &mut options, &file) {
                        Ok(name) => name,
                        Err(e) => {
                            eprintln!("Error handling project name: {}", e);
                            return;
                        }
                    };
                }

                let dir_path = file.path.split('/').collect::<Vec<_>>();
                if dir_path.len() > 1 {
                    create_dirs(&shellexpand::tilde(&Keywords::replace_keywords(
                        keywords,
                        file.path.replace(dir_path.last().unwrap(), ""),
                    )));
                }

                let output = Keywords::replace_keywords(keywords, file.content);
                let path = Keywords::replace_keywords(keywords, file.path);

                if options.use_liquid.is_some() {
                    let liquified = Self::liquify(&output);
                    write_content(&shellexpand::tilde(&path), liquified);
                } else {
                    write_content(&shellexpand::tilde(&path), output);
                }
            });

        options.handle();
    }

    pub fn show_info(template: &Self) {
        match &template.info {
            Some(information) => println!(
                "{}: {}\n{}: {}\n{}: {}\n",
                "Name".yellow(),
                information.name.as_ref().unwrap().bold().green(),
                "Description".yellow(),
                information.description.as_ref().unwrap().bold().green(),
                "Author".yellow(),
                information.author.as_ref().unwrap().bold().green()
            ),
            None => {}
        }
    }
}
