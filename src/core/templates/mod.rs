use crate::utils::*;
use crate::*;
use colored::Colorize;
use promptly::prompt;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::Path};
pub mod options;

pub const KEYWORDS_REGEX: &str = r"\{\{\$.*?\}\}";

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Options {
    pub git: bool,
    pub use_liquid: Option<bool>,
    pub json_data: Option<serde_json::Value>,
    pub project_root: String,
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

    pub fn dump_options(&self) -> Option<Options> {
        self.options.clone()
    }

    pub fn generate(dest: &str) -> Result<(), String> {
        let files: Vec<File> = list_files(Path::new("./"))
            .unwrap_or_default()
            .into_iter()
            .filter(|file| !file.contains(".git"))
            .map(|file| {
                File::new(
                    file.replace("./", ""),
                    fs::read_to_string(&file).unwrap_or_default(),
                )
            })
            .collect();

        let template = Self {
            info: None,
            files: Some(files),
            options: None,
        };

        let toml_string = toml::to_string_pretty(&template)
            .map_err(|e| format!("Failed to serialize template: {}", e))?;

        fs::write(dest, toml_string)
            .map_err(|e| format!("Failed to write template to file: {}", e))?;

        println!(
            "{}: Template successfully generated at {}",
            "Success".green().bold().blink(),
            dest
        );
        Ok(())
    }

    pub fn liquify(string: &str) -> Result<String, liquid::Error> {
        let parser = liquid::ParserBuilder::with_stdlib().build()?;
        let empty_globals = liquid::Object::new();

        parser.parse(string)?.render(&empty_globals)
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

    fn process_file(
        file: &File,
        keywords: &mut HashMap<String, String>,
        re: &Regex,
        json_data: &serde_json::Value,
        options: &mut Options,
    ) -> Result<String, String> {
        Fns::find_and_exec(&file.content, keywords, re, json_data);
        Fns::find_and_exec(&file.path, keywords, re, json_data);

        let project = Self::handle_project_name(keywords, options, file)
            .map_err(|e| format!("Error handling project name: {}", e))?;

        let dir_path = Path::new(&file.path)
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();

        if !dir_path.is_empty() {
            create_dirs(&Keywords::replace_keywords(keywords, &dir_path));
        }

        Ok(project)
    }

    fn prepare_file_content(
        file_content: &str,
        file_path: &str,
        keywords: &HashMap<String, String>,
        options: &Options,
    ) -> Result<(String, String), String> {
        let output = Keywords::replace_keywords(keywords, file_content);
        let path = Keywords::replace_keywords(keywords, file_path);

        let final_output = if options.use_liquid.unwrap_or(false) {
            Self::liquify(&output).map_err(|e| format!("Liquid error: {}", e))?
        } else {
            output
        };

        Ok((path, final_output))
    }

    pub fn extract(&mut self, keywords: &mut HashMap<String, String>) -> Result<(), String> {
        let re = Regex::new(KEYWORDS_REGEX).map_err(|e| format!("Invalid regex: {}", e))?;
        let mut options = self.options.take().unwrap_or_default();
        let json_data = options.json_data.clone().unwrap_or(serde_json::Value::Null);
        let files = self.files.take().unwrap_or_default();
        let mut project = String::new();

        for file in files {
            if project.is_empty() {
                project = Self::process_file(&file, keywords, &re, &json_data, &mut options)?;
            }

            let (path, final_output) =
                Self::prepare_file_content(&file.content, &file.path, keywords, &options)?;

            write_content(&path, &final_output).map_err(|e| format!("File write error: {}", e))?;
        }

        options.handle();

        Ok(())
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
