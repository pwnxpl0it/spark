use crate::config::*;
use crate::keywords::Keywords;
use crate::types::*;
use crate::utils::*;
use colored::Colorize;
use regex::Regex;
use std::{collections::HashMap, fs, io, path::Path};

impl Template {
    /// This method return a new Template instance, it takes `Information` and a vector of `File`.
    fn new(info_: Information, files_: Vec<File>) -> Self {
        Self {
            info: Some(info_),
            files: files_,
        }
    }

    /// This method basically generate a new Template and saves it
    /// It utilizes the Self::new() method, gives it an Empty `Information` Instance and the
    /// Vector it created by listing all files in the working current directory
    pub fn generate() {
        // get the current directory name by utilizing Keywords::init() -> a table of default
        // Keywords for idkmng, including current Directory
        let dest = format!("{}.toml", Keywords::init()["{{$CURRENTDIR}}"]);
        println!("{}: {}", "Creating Template".bold().green(), &dest.yellow());

        let mut files: Vec<File> = Vec::new(); // Create a new Vector of File

        list_files(Path::new("./")).iter().for_each(|file| {
            //TODO: Add more to ignore list maybe adding a --ignore flag will be good
            if !file.contains(".git") {
                let file = File::new(file.to_string().replace("./", ""), {
                    match fs::read_to_string(file) {
                        Ok(content) => content,
                        Err(e) => panic!("{}:{}", file.red(), e),
                    }
                });
                files.push(file); // Push to Files Vector
            }
        });

        let template = Self::new(
            Information {
                name: Some(String::from("")),
                author: Some(String::from("")),
                description: Some(String::from("")),
            },
            files,
        );

        let toml_string = toml::to_string_pretty(&template).expect("Failed to create toml string");
        fs::write(&dest, toml_string).unwrap();
    }

    /// This method "extracts" a template, means it takes a template and starts initializing files based that template
    pub fn extract(filename: String) {
        let mut keywords = Keywords::init();
        let template = Self::validate(filename, keywords.to_owned());
        let re = Regex::new(KEYWORDS_REGEX).unwrap();

        println!("{}: {}", "Using Template".blue(), &template.magenta());

        let sample = Self::parse(&template);
        Self::show_info(&sample);

        let files = sample.files;
        let mut project = String::from("");
        files.into_iter().for_each(|file| {
            keywords = find_and_exec_fns(file.content.clone(), keywords.clone(), re.clone());
            keywords = find_and_exec_fns(file.path.clone(), keywords.clone(), re.clone());

            if file.path.contains("{{$PROJECTNAME}}") || file.content.contains("{{$PROJECTNAME}}") {
                if project.is_empty() {
                    println!("Project name: ");
                    io::stdin().read_line(&mut project).unwrap();
                    project = project.trim().to_string();
                    keywords.insert("{{$PROJECTNAME}}".to_string(), project.to_owned());
                }
            }

            let dir = file.path.split('/').collect::<Vec<_>>();
            let path = Keywords::replace_keywords(keywords.to_owned(), file.path.to_owned());

            if dir.len() > 1 {
                create_dirs(
                    &Keywords::replace_keywords(
                        keywords.to_owned(),
                        file.path.to_owned().replace(dir[dir.len() - 1], ""),
                    )
                    .replace('~', &keywords["{{$HOME}}"]),
                )
            }

            write_content(
                &path.replace('~', &keywords["{{$HOME}}"]),
                Keywords::replace_keywords(keywords.to_owned(), file.content),
            )
        });
    }

    /// Parse a Template
    fn parse(template: &str) -> Self {
        let content =
            fs::read_to_string(template).unwrap_or_else(|_| panic!("Failed to Parse {}", template));
        toml::from_str(&content).unwrap()
    }

    /// This method validates Template path, in other words it just checks if the template is in
    /// the current working Directory,if not it uses the default templates directory, also automatically adds .toml
    fn validate(mut template: String, keywords: HashMap<String, String>) -> String {
        if template.contains(".toml") {
            //IGNORE
        } else {
            template += ".toml"
        }

        if fs::read_to_string(&template).is_ok() {
            //IGNORE
        } else {
            template = TEMPLATES_PATH.replace("{{$HOME}}", &keywords["{{$HOME}}"]) + &template
        }

        template
    }

    /// This method shows information about current Template, basically Reads them from Information
    /// section in the Template TOML file
    fn show_info(template: &Self) {
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