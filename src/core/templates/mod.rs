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

        write_content(dest, &toml_string)
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
        
        Self::handle_project_name(keywords, options, file)
            .map_err(|e| format!("Error handling project name: {}", e))
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

        if let Some(dir) = Path::new(&path).parent() {
            let dir_str = dir.to_string_lossy();
            if !dir_str.is_empty() {
                create_dirs(&dir_str);
            }
        }

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::fs;

    #[test]
    fn liquify_errors_on_unknown_variable() {
        let result = Template::liquify("Hello {{ name }}!");
        assert!(result.is_err());
    }

    #[test]
    fn liquify_renders_literal_text() {
        let output = Template::liquify("Hello spark!").unwrap();
        assert_eq!(output, "Hello spark!");
    }

    #[test]
    fn liquify_applies_stdlib_filters() {
        let output = Template::liquify("{{ 'hello' | upcase }}").unwrap();
        assert_eq!(output, "HELLO");
    }

    #[test]
    fn set_info_files_and_options() {
        let mut template = Template {
            info: None,
            options: None,
            files: None,
        };

        template.set_info(Information {
            name: Some("demo".into()),
            author: Some("author".into()),
            description: Some("desc".into()),
        });
        template.set_files(vec![File::new("a.txt".into(), "hi".into())]);
        template.set_options(Options {
            git: true,
            use_liquid: Some(false),
            json_data: None,
            project_root: "proj".into(),
        });

        assert_eq!(template.info.as_ref().unwrap().name.as_deref(), Some("demo"));
        assert_eq!(template.files.as_ref().unwrap().len(), 1);
        let options = template.dump_options().unwrap();
        assert!(options.git);
        assert_eq!(options.project_root, "proj");
    }

    #[test]
    fn handle_project_name_without_placeholder_is_noop() {
        let mut keywords = HashMap::new();
        let mut options = Options::default();
        let file = File::new("test.txt".into(), "hello world".into());

        let result = Template::handle_project_name(&mut keywords, &mut options, &file).unwrap();
        assert!(result.is_empty());
        assert!(!keywords.contains_key("{{$PROJECTNAME}}"));
    }

    #[test]
    fn handle_project_name_errors_when_prompt_unavailable() {
        let mut keywords = HashMap::new();
        let mut options = Options::default();
        let file = File::new(
            "test.txt".into(),
            "Hello {{$PROJECTNAME}}".into(),
        );

        let result = Template::handle_project_name(&mut keywords, &mut options, &file);
        assert!(result.is_err());
    }

    #[test]
    fn prepare_file_content_replaces_keywords_without_liquid() {
        let mut keywords = HashMap::new();
        keywords.insert("{{$TEST}}".to_string(), "value".to_string());
        let options = Options {
            git: false,
            use_liquid: None,
            json_data: None,
            project_root: String::new(),
        };

        let (path, content) =
            Template::prepare_file_content("Hello {{$TEST}}", "out.txt", &keywords, &options)
                .unwrap();

        assert_eq!(path, "out.txt");
        assert_eq!(content, "Hello value");
    }

    #[test]
    fn prepare_file_content_applies_liquid_when_enabled() {
        let keywords = HashMap::new();
        let options = Options {
            git: false,
            use_liquid: Some(true),
            json_data: None,
            project_root: String::new(),
        };

        let (_path, content) = Template::prepare_file_content(
            "{{ 'spark' | upcase }}",
            "out.txt",
            &keywords,
            &options,
        )
        .unwrap();

        assert_eq!(content, "SPARK");
    }

    #[test]
    fn prepare_file_content_creates_parent_directories() {
        let sub_dir = std::env::temp_dir().join("spark_test_prepare_dirs");
        let file_path = sub_dir.join("nested").join("test.txt");
        let _ = fs::remove_dir_all(&sub_dir);

        let keywords = HashMap::new();
        let options = Options {
            git: false,
            use_liquid: None,
            json_data: None,
            project_root: String::new(),
        };

        let (path, content) = Template::prepare_file_content(
            "hi",
            &file_path.to_string_lossy(),
            &keywords,
            &options,
        )
        .unwrap();

        assert_eq!(path, file_path.to_string_lossy());
        assert_eq!(content, "hi");
        assert!(file_path.parent().unwrap().is_dir());

        let _ = fs::remove_dir_all(&sub_dir);
    }

    #[test]
    fn extract_with_no_files_succeeds() {
        let mut template = Template {
            info: None,
            options: Some(Options {
                git: false,
                use_liquid: None,
                json_data: None,
                project_root: String::new(),
            }),
            files: Some(vec![]),
        };
        let mut keywords = HashMap::new();
        assert!(template.extract(&mut keywords).is_ok());
    }

    #[test]
    fn extract_writes_files_with_keyword_replacement() {
        let out_dir = std::env::temp_dir().join("spark_test_extract");
        let _ = fs::remove_dir_all(&out_dir);
        fs::create_dir_all(&out_dir).unwrap();
        let out_file = out_dir.join("hello.txt");

        let mut template = Template {
            info: None,
            options: Some(Options {
                git: false,
                use_liquid: None,
                json_data: None,
                project_root: String::new(),
            }),
            files: Some(vec![File::new(
                out_file.to_string_lossy().to_string(),
                "Hello {{$NAME}}".into(),
            )]),
        };

        let mut keywords = HashMap::new();
        keywords.insert("{{$NAME}}".to_string(), "spark".to_string());

        template.extract(&mut keywords).unwrap();

        let written = fs::read_to_string(&out_file).unwrap();
        assert_eq!(written, "Hello spark");

        let _ = fs::remove_dir_all(&out_dir);
    }
}
