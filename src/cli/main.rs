use crate::config::*;
use colored::*;
use dotenvy::dotenv;
use spark::{Context, Error, Keywords, MissingPolicy, Template};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
mod args;
mod config;
use args::{Cli, Commands};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{}: {}", "error".red().bold(), e);
            ExitCode::FAILURE
        }
    }
}

fn run() -> spark::Result<()> {
    let args = Cli::parse_args();
    let config = Config::new(&args.config);
    config.ensure_default_exists(args.config == DEFAULT_CONFIG_PATH);
    dotenv().ok();
    let mut keywords = Keywords::init();
    keywords.extend(config.get_keywords());

    if let Some(pairs) = args.keywords.as_deref() {
        parse_from_pairs(pairs, &mut keywords)?;
    }

    match (&args.command, args.template.as_deref()) {
        (Some(Commands::Init), _) => run_init(),
        (None, Some(template)) => run_template(&args, &config, template, keywords),
        (None, None) => Err(Error::Custom(format!(
            "no template specified, see {}",
            "--help".bold().green()
        ))),
    }
}

/// Parses `--from "k=v, k2=v2"` into the keyword map.
fn parse_from_pairs(pairs: &str, keywords: &mut HashMap<String, String>) -> spark::Result<()> {
    for pair in pairs.split(',') {
        let pair = pair.trim();
        if pair.is_empty() {
            continue;
        }
        let (keyword, value) = pair.split_once('=').ok_or_else(|| {
            Error::Custom(format!(
                "invalid --from entry '{}': expected KEY=VALUE",
                pair
            ))
        })?;
        let keyword = keyword.trim();
        if keyword.is_empty() {
            return Err(Error::Custom(format!(
                "invalid --from entry '{}': empty key",
                pair
            )));
        }
        keywords.insert(Keywords::from(keyword, None), value.trim().to_string());
    }
    Ok(())
}

fn run_init() -> spark::Result<()> {
    let cwd = std::env::current_dir()?;
    let name = cwd.file_name().and_then(|n| n.to_str()).ok_or_else(|| {
        Error::Custom("cannot derive a template name from the current directory".into())
    })?;
    let dest = format!("{name}.toml");
    println!("{}: {}", "Creating Template".bold().green(), dest.yellow());
    Template::generate(&dest).map_err(Error::Custom)
}

/// Resolves `<name>` to a template file: a local path first, then the
/// templates directory next to the config file.
fn resolve_template_path(config: &Config, template: &str) -> spark::Result<PathBuf> {
    let mut name = template.to_string();
    if !name.ends_with(".toml") {
        name.push_str(".toml");
    }

    let local = PathBuf::from(&name);
    if local.is_file() {
        return Ok(local);
    }

    let in_config = Path::new(&config.templates_path).join(&name);
    if in_config.is_file() {
        return Ok(in_config);
    }

    Err(Error::Custom(format!(
        "template '{}' not found (looked for '{}' and '{}')",
        template,
        local.display(),
        in_config.display()
    )))
}

fn run_template(
    args: &Cli,
    config: &Config,
    template: &str,
    keywords: HashMap<String, String>,
) -> spark::Result<()> {
    let template_path = resolve_template_path(config, template)?;
    let mut parsed_template = Template::from_file(&template_path)
        .map_err(|e| Error::Custom(format!("{}: {}", template_path.display(), e)))?;

    if !args.quiet {
        println!(
            "\n{}: {}",
            "Using Template".blue(),
            template_path.display().to_string().magenta()
        );
        Template::show_info(&parsed_template);
    }

    let mut options = parsed_template.dump_options().unwrap_or_default();

    if let Some(json_path) = &args.json {
        let json_file = fs::read_to_string(json_path)
            .map_err(|e| Error::Custom(format!("{}: {}", json_path.display(), e)))?;
        let json_data = serde_json::from_str(&json_file)
            .map_err(|e| Error::Custom(format!("{}: {}", json_path.display(), e)))?;
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
    parsed_template.extract_with_context(&context)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_pairs_populate_keywords() {
        let mut k = HashMap::new();
        parse_from_pairs("PROJECTNAME=myapp, author=alice, version=1.0", &mut k).unwrap();
        assert_eq!(k.get("{{$PROJECTNAME}}").map(String::as_str), Some("myapp"));
        assert_eq!(k.get("{{$author}}").map(String::as_str), Some("alice"));
        assert_eq!(k.get("{{$version}}").map(String::as_str), Some("1.0"));
    }

    #[test]
    fn from_pairs_reject_malformed_entries() {
        let mut k = HashMap::new();
        assert!(parse_from_pairs("novalue", &mut k).is_err());
        assert!(parse_from_pairs("=x", &mut k).is_err());
        assert!(k.is_empty());
    }

    #[test]
    fn resolve_template_prefers_local_then_config_dir() {
        let dir = std::env::temp_dir().join("spark_test_resolve_template");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("templates")).unwrap();
        fs::write(dir.join("templates/demo.toml"), "").unwrap();
        let config = Config::new(&dir.join("config.toml").to_string_lossy());

        let found = resolve_template_path(&config, "demo").unwrap();
        assert_eq!(found, dir.join("templates/demo.toml"));

        let err = resolve_template_path(&config, "missing").unwrap_err();
        assert!(err.to_string().contains("not found"));

        let _ = fs::remove_dir_all(&dir);
    }
}
