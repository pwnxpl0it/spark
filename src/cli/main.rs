use crate::config::*;
use spark::Keywords;
use spark::Template;
use std::fs;
mod args;
use args::Cli;
mod config;
use colored::*;
use dotenv::dotenv;

fn main() {
    let args = Cli::parse();
    let config = Config::new(args.value_of("config").unwrap());

    dotenv().ok();
    let mut keywords = Keywords::init();

    keywords.extend(config.clone().get_keywords());

    if args.subcommand_matches("init").is_some() {
        let dest = format!(
            "{}.toml",
            std::env::current_dir()
                .unwrap()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
        );
        println!("{}: {}", "Creating Template".bold().green(), &dest.yellow());
        Template::generate(&dest);
    } else if let Some(temp) = args.value_of("template") {
        let mut template = temp.to_string();

        if !template.ends_with(".toml") {
            template += ".toml";
        }

        let full_template_path = if fs::read_to_string(&template).is_err() {
            format!("{}/{}", config.templates_path, template)
        } else {
            template
        };

        let template_content = fs::read_to_string(&full_template_path).unwrap_or_else(|_| {
            panic!(
                "{}: {}",
                "Failed to read template".red().bold(),
                full_template_path
            )
        });

        let mut parsed_template: Template = toml::from_str(&template_content).unwrap();

        if !args.is_present("quiet") {
            println!(
                "\n{}: {}",
                "Using Template".blue(),
                full_template_path.magenta()
            );
            Template::show_info(&parsed_template);
        }

        let mut options = parsed_template.dump_options().unwrap_or_default();

        if args.is_present("json") {
            let json_file = fs::read_to_string(args.value_of("json").unwrap());
            let json_data = serde_json::from_str(&json_file.unwrap()).unwrap();
            options.set_json(json_data);
        }

        if args.is_present("git") {
            options.set_git(true);
            options.set_project_root("{{$PROJECTNAME}}");
        }

        if args.is_present("no-liquid") {
            options.use_liquid = None;
        }

        parsed_template.set_options(options);
        parsed_template.extract(&mut keywords);
    } else {
        eprintln!(
            "{} {}",
            "No args specified please use".yellow(),
            "--help".bold().green()
        );
    }
}
