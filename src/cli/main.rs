use crate::config::*;
use colored::*;
use dotenvy::dotenv;
use spark::{Context, Keywords, MissingPolicy, Template};
use std::fs;
mod args;
mod config;
use args::{Cli, Commands};

fn main() {
    let args = Cli::parse_args();
    let config = Config::new(&args.config);
    dotenv().ok();
    let mut keywords = Keywords::init();
    keywords.extend(config.clone().get_keywords());

    if let Some(pairs) = args.keywords.as_deref() {
        let keys: Vec<&str> = pairs.split(',').collect();
        for key in keys {
            let (keyword, value) = key.split_once('=').unwrap();
            keywords.insert(
                Keywords::from(keyword.trim(), None),
                value.trim().to_string(),
            );
        }
    }

    if matches!(args.command, Some(Commands::Init)) {
        let dest = format!(
            "{}.toml",
            std::env::current_dir()
                .unwrap()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
        );
        println!("{}: {}", "Creating Template".bold().green(), dest.yellow());
        Template::generate(&dest).expect("Failed to generate template");
    } else if let Some(temp) = args.template.as_deref() {
        let mut template = temp.to_string();

        if !template.ends_with(".toml") {
            template.push_str(".toml");
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

        if !args.quiet {
            println!(
                "\n{}: {}",
                "Using Template".blue(),
                full_template_path.magenta()
            );
            Template::show_info(&parsed_template);
        }

        let mut options = parsed_template.dump_options().unwrap_or_default();

        if let Some(json_path) = &args.json {
            let json_file = fs::read_to_string(json_path);
            let json_data = serde_json::from_str(&json_file.unwrap()).unwrap();
            options.set_json(json_data);
        }

        if args.git {
            options.set_git(true);
            // Only fall back to prompting for a project name when the template
            // did not already define where the repository root lives.
            if options.project_root.trim().is_empty() {
                options.project_root = "{{$PROJECTNAME}}".to_string();
            }
        }

        if args.no_liquid {
            options.use_liquid = Some(false);
        } else if args.liquid {
            options.use_liquid = Some(true);
        }

        parsed_template.set_options(options);

        let mut context = Context::from(keywords);
        if args.strict {
            context = context.on_missing(MissingPolicy::Error);
        }
        parsed_template.extract_with_context(&context).unwrap();
    } else {
        eprintln!(
            "{} {}",
            "No args specified, please use".yellow(),
            "--help".bold().green()
        );
    }
}
