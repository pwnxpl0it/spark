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

    for pairs in &args.keywords {
        parse_from_pairs(pairs, &mut keywords)?;
    }

    match (&args.command, args.template.as_deref()) {
        (Some(Commands::Init), _) => run_init(),
        (Some(Commands::List), _) => run_list(&config),
        (None, Some(template)) => run_template(&args, &config, template, keywords),
        (None, None) => Err(Error::Custom(format!(
            "no template specified, see {}",
            "--help".bold().green()
        ))),
    }
}

/// Parses one `--from "k=v, k2=v2"` argument into the keyword map.
///
/// Entries are split on commas. A segment that contains no `=` is treated as
/// a continuation of the previous value, so `tags=a,b` yields `tags = "a,b"`.
fn parse_from_pairs(pairs: &str, keywords: &mut HashMap<String, String>) -> spark::Result<()> {
    let mut current: Option<(String, String)> = None;

    for segment in pairs.split(',') {
        match segment.split_once('=') {
            Some((key, value)) => {
                if let Some((k, v)) = current.take() {
                    keywords.insert(Keywords::from(&k, None), v);
                }
                let key = key.trim();
                if key.is_empty() {
                    return Err(Error::Custom(format!(
                        "invalid --from entry '{}': empty key",
                        segment.trim()
                    )));
                }
                current = Some((key.to_string(), value.trim().to_string()));
            }
            None => match current.as_mut() {
                Some((_, v)) => {
                    v.push(',');
                    v.push_str(segment.trim());
                }
                None => {
                    if segment.trim().is_empty() {
                        continue;
                    }
                    return Err(Error::Custom(format!(
                        "invalid --from entry '{}': expected KEY=VALUE",
                        segment.trim()
                    )));
                }
            },
        }
    }

    if let Some((k, v)) = current {
        keywords.insert(Keywords::from(&k, None), v);
    }
    Ok(())
}

fn run_init() -> spark::Result<()> {
    let cwd = std::env::current_dir()?;
    let name = cwd.file_name().and_then(|n| n.to_str()).ok_or_else(|| {
        Error::Custom("cannot derive a template name from the current directory".into())
    })?;
    let dest = format!("{name}.toml");
    eprintln!("{}: {}", "Creating Template".bold().green(), dest.yellow());
    Template::generate(&dest).map_err(Error::Custom)
}

/// Returns the sorted template names (file stem) found in the templates dir,
/// each paired with its `[info]` summary when the file parses.
fn list_templates(config: &Config) -> spark::Result<Vec<(String, Option<String>)>> {
    let dir = Path::new(&config.templates_path);
    if !dir.is_dir() {
        return Ok(Vec::new());
    }

    let mut entries: Vec<(String, Option<String>)> = fs::read_dir(dir)?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.is_file() && p.extension().is_some_and(|ext| ext == "toml"))
        .map(|p| {
            let name = p
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();
            let summary = Template::from_file(&p)
                .ok()
                .and_then(|t| t.info.and_then(|i| i.description.or(i.name)));
            (name, summary)
        })
        .collect();
    entries.sort();
    Ok(entries)
}

fn run_list(config: &Config) -> spark::Result<()> {
    let templates = list_templates(config)?;
    if templates.is_empty() {
        eprintln!(
            "{} {}",
            "No templates found in".yellow(),
            config.templates_path.bold()
        );
        return Ok(());
    }
    eprintln!(
        "{}: {}\n",
        "Templates in".blue(),
        config.templates_path.bold()
    );
    for (name, summary) in templates {
        match summary {
            Some(desc) => println!("{}  {}", name.green().bold(), desc.dimmed()),
            None => println!("{}", name.green().bold()),
        }
    }
    Ok(())
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
        eprintln!(
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

    let mut context = Context::from(keywords)
        .force(args.force)
        .allow_outside(args.allow_outside);
    if args.strict {
        context = context.on_missing(MissingPolicy::Error);
    }
    if args.dry_run {
        let rendered = parsed_template.render(&context)?;
        print_dry_run(&rendered);
        return Ok(());
    }

    // Template and JSON paths are already resolved, so it is safe to move into
    // the output directory now; relative [[files]] paths land there.
    if let Some(out) = &args.output_dir {
        fs::create_dir_all(out)
            .map_err(|e| Error::Custom(format!("cannot create '{}': {}", out.display(), e)))?;
        std::env::set_current_dir(out)
            .map_err(|e| Error::Custom(format!("cannot enter '{}': {}", out.display(), e)))?;
    }

    parsed_template.extract_with_context(&context)?;
    Ok(())
}

/// Prints each rendered file as a `==> path` header followed by its content.
fn print_dry_run(rendered: &[spark::RenderedFile]) {
    eprintln!("{}", "dry run: nothing will be written".yellow().bold());
    for file in rendered {
        println!("{} {}", "==>".blue().bold(), file.path.bold());
        println!("{}", file.content);
    }
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
    fn from_pairs_keep_commas_inside_values() {
        let mut k = HashMap::new();
        parse_from_pairs("tags=a, b,c, name=x", &mut k).unwrap();
        assert_eq!(k.get("{{$tags}}").map(String::as_str), Some("a,b,c"));
        assert_eq!(k.get("{{$name}}").map(String::as_str), Some("x"));

        let mut k = HashMap::new();
        parse_from_pairs("list=1,2,3", &mut k).unwrap();
        assert_eq!(k.get("{{$list}}").map(String::as_str), Some("1,2,3"));
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

    #[test]
    fn list_templates_reads_toml_files_sorted_with_descriptions() {
        let dir = std::env::temp_dir().join("spark_test_list_templates");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("templates")).unwrap();
        fs::write(
            dir.join("templates/zeta.toml"),
            "[info]\ndescription='last one'\n[[files]]\npath='a'\ncontent='b'",
        )
        .unwrap();
        fs::write(
            dir.join("templates/alpha.toml"),
            "[[files]]\npath='a'\ncontent='b'",
        )
        .unwrap();
        fs::write(dir.join("templates/notes.txt"), "ignored").unwrap();
        let config = Config::new(&dir.join("config.toml").to_string_lossy());

        let listed = list_templates(&config).unwrap();
        assert_eq!(
            listed,
            vec![
                ("alpha".to_string(), None),
                ("zeta".to_string(), Some("last one".to_string()))
            ]
        );

        let missing = Config::new("/nonexistent/spark/config.toml");
        assert!(list_templates(&missing).unwrap().is_empty());

        let _ = fs::remove_dir_all(&dir);
    }
}
