use crate::config::*;
use colored::*;
use dotenvy::dotenv;
use spark::{Context, Error, Hooks, Keywords, MissingPolicy, Options, Template, UndoLog};
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
        (
            Some(Commands::Init {
                git,
                liquid,
                project_root,
                from_github,
            }),
            _,
        ) => run_init(*git, *liquid, project_root.clone(), from_github.clone()),
        (Some(Commands::List), _) => run_list(&config),
        (Some(Commands::Undo), _) => run_undo(&config),
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

fn run_init(
    git: bool,
    liquid: bool,
    project_root: Option<String>,
    from_github: Option<String>,
) -> spark::Result<()> {
    // Only emit an `[options]` table when at least one option was supplied, so a
    // plain `spark init` keeps producing a minimal template.
    let options = (git || liquid || project_root.is_some()).then(|| Options {
        git,
        use_liquid: liquid.then_some(true),
        json_data: None,
        project_root: project_root.unwrap_or_default(),
    });

    // Source the files from a cloned GitHub repo, or the current directory.
    // The `TempDir` guard must outlive the capture, so keep it bound.
    let (source, name, _clone_guard) = match from_github {
        Some(spec) => {
            let (dir, repo, guard) = clone_github(&spec)?;
            (dir, repo, Some(guard))
        }
        None => {
            let cwd = std::env::current_dir()?;
            let name = cwd
                .file_name()
                .and_then(|n| n.to_str())
                .ok_or_else(|| {
                    Error::Custom("cannot derive a template name from the current directory".into())
                })?
                .to_string();
            (cwd, name, None)
        }
    };

    let dest = format!("{name}.toml");
    eprintln!("{}: {}", "Creating Template".bold().green(), dest.yellow());
    Template::generate_from_with(&source, &dest, options).map_err(Error::Custom)
}

/// Splits a `--from-github` spec into its `owner/repo` slug, optional ref, and
/// the repo name used for the generated template file. Accepts `owner/repo` or
/// `owner/repo@ref`; a trailing `.git` on the repo is tolerated.
fn parse_github_spec(spec: &str) -> spark::Result<(String, Option<String>, String)> {
    let (slug, reference) = match spec.split_once('@') {
        Some((s, r)) => (s.trim(), Some(r.trim().to_string())),
        None => (spec.trim(), None),
    };
    if reference.as_deref().is_some_and(str::is_empty) {
        return Err(Error::Custom(format!(
            "invalid --from-github '{spec}': empty ref after '@'"
        )));
    }

    let slug = slug.trim_end_matches('/');
    let (owner, repo) = slug.split_once('/').ok_or_else(|| {
        Error::Custom(format!(
            "invalid --from-github '{spec}': expected owner/repo"
        ))
    })?;
    let repo = repo.trim_end_matches(".git");
    if owner.is_empty() || repo.is_empty() || repo.contains('/') {
        return Err(Error::Custom(format!(
            "invalid --from-github '{spec}': expected owner/repo[@ref]"
        )));
    }

    Ok((format!("{owner}/{repo}"), reference, repo.to_string()))
}

/// A temporary directory that is removed when dropped. Keeps the clone off the
/// dependency list (std only) while still cleaning up on every exit path.
struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new() -> std::io::Result<Self> {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let path = std::env::temp_dir().join(format!("spark-{}-{}", std::process::id(), nanos));
        fs::create_dir_all(&path)?;
        Ok(Self { path })
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

/// Shallow-clones a GitHub repo into a temporary directory and returns the
/// clone path, the repo name, and the `TempDir` guard (which deletes the clone
/// when dropped). The ref, when given, must be a branch or tag.
fn clone_github(spec: &str) -> spark::Result<(PathBuf, String, TempDir)> {
    let (slug, reference, name) = parse_github_spec(spec)?;
    let url = format!("https://github.com/{slug}.git");

    let tmp = TempDir::new()
        .map_err(|e| Error::Custom(format!("cannot create a temporary directory: {e}")))?;
    let dest = tmp.path().join("repo");

    eprintln!("{}: {}", "Cloning".bold().green(), url.yellow());
    let mut cmd = std::process::Command::new("git");
    cmd.args(["clone", "--depth", "1", "--quiet"]);
    if let Some(reference) = &reference {
        cmd.args(["--branch", reference]);
    }
    cmd.arg(&url).arg(&dest);

    let status = cmd
        .status()
        .map_err(|e| Error::Custom(format!("failed to run git (is it installed?): {e}")))?;
    if !status.success() {
        let what = reference
            .map(|r| format!("{slug}@{r}"))
            .unwrap_or_else(|| slug.clone());
        return Err(Error::Custom(format!(
            "git clone failed for '{what}' (check the repo/ref exists and is accessible)"
        )));
    }

    Ok((dest, name, tmp))
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

    // Build with `/` rather than `Path::join` so the displayed path does not
    // mix separators on Windows (config paths are already normalised).
    let in_config = PathBuf::from(format!("{}/{}", config.templates_path, name));
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
    // Hooks run arbitrary shell commands, so they are opt-in: without
    // `--run-hooks` a template's hooks are never executed.
    let hooks = parsed_template.hooks.clone();

    if args.dry_run {
        let rendered = parsed_template.render(&context)?;
        print_dry_run(
            &rendered,
            parsed_template.files.as_deref().unwrap_or_default(),
        );
        if let Some(hooks) = hooks.as_ref().filter(|h| !h.is_empty()) {
            print_hook_plan(hooks, args.run_hooks);
        }
        return Ok(());
    }

    let run_hooks = match hooks.as_ref().filter(|h| !h.is_empty()) {
        Some(hooks) if args.run_hooks => Some(hooks),
        Some(hooks) => {
            eprintln!(
                "{}: template defines {} hook(s); re-run with {} to execute them",
                "note".bold(),
                hooks.len(),
                "--run-hooks".bold().green()
            );
            None
        }
        None => None,
    };

    // Resolve the undo manifest path before `--output-dir` changes the cwd.
    // A relative `--config` would otherwise resolve against the output dir,
    // writing the manifest inside the generated project and leaving it where
    // `spark undo` (run from the original directory) cannot find it.
    let manifest = config.undo_manifest_path();

    // Template and JSON paths are already resolved, so it is safe to move into
    // the output directory now; relative [[files]] paths land there.
    if let Some(out) = &args.output_dir {
        fs::create_dir_all(out)
            .map_err(|e| Error::Custom(format!("cannot create '{}': {}", out.display(), e)))?;
        std::env::set_current_dir(out)
            .map_err(|e| Error::Custom(format!("cannot enter '{}': {}", out.display(), e)))?;
    }

    // Pre hooks run before anything is written so a failed check aborts cleanly.
    if let Some(hooks) = run_hooks {
        hooks.run_pre()?;
    }

    let (_rendered, undo, project_root) = parsed_template.extract_with_undo(&context)?;

    // Record how to reverse this extraction so `spark undo` can roll it back.
    // A failure to write the manifest must not fail the generation itself.
    if !undo.is_empty() {
        match undo.save(&manifest) {
            Ok(()) => eprintln!(
                "{}: run {} to revert this",
                "undo".blue(),
                "spark undo".bold().green()
            ),
            Err(e) => eprintln!(
                "{}: could not record undo information: {}",
                "warning".yellow(),
                e
            ),
        }
    }

    // Post hooks run after the files (and the undo manifest) are in place,
    // inside the generated project root when the template defines one (so
    // `cargo build`, `npm install`, … run in the project), falling back to the
    // current directory otherwise.
    if let Some(hooks) = run_hooks {
        let post_dir = project_root
            .as_deref()
            .map(Path::new)
            .filter(|p| p.is_dir());
        hooks.run_post(post_dir)?;
    }
    Ok(())
}

/// Reverses the most recent extraction recorded in the undo manifest.
fn run_undo(config: &Config) -> spark::Result<()> {
    let manifest = config.undo_manifest_path();
    if !manifest.is_file() {
        return Err(Error::Custom(
            "nothing to undo (no template has been generated yet)".into(),
        ));
    }

    let log = UndoLog::load(&manifest)
        .map_err(|e| Error::Custom(format!("cannot read undo manifest: {e}")))?;
    let report = log.apply();

    for err in &report.errors {
        eprintln!("{}: {}", "warning".yellow(), err);
    }
    eprintln!(
        "{}: restored {}, removed {} file(s)",
        "undo".blue(),
        report.restored,
        report.deleted
    );

    // Consume the manifest so a second `undo` is a no-op rather than a replay.
    if let Err(e) = fs::remove_file(&manifest) {
        eprintln!(
            "{}: could not clear undo manifest: {}",
            "warning".yellow(),
            e
        );
    }
    Ok(())
}

/// Lists a template's hook commands for `--dry-run`, so they can be reviewed
/// before being trusted. Nothing is executed.
fn print_hook_plan(hooks: &Hooks, run_hooks: bool) {
    let note = if run_hooks {
        "hooks (not run in dry run)"
    } else {
        "hooks (skipped without --run-hooks)"
    };
    eprintln!("{}", note.yellow().bold());
    for cmd in &hooks.pre {
        eprintln!("  {} {}", "pre ".blue(), cmd);
    }
    for cmd in &hooks.post {
        eprintln!("  {} {}", "post".blue(), cmd);
    }
}

/// Prints each rendered file as a `==> path` header followed by its content.
///
/// For non-overwrite entries the rendered content is the patch *fragment*
/// (what gets merged), not the final file, so the mode is shown alongside the
/// path to make that clear.
fn print_dry_run(rendered: &[spark::RenderedFile], sources: &[spark::File]) {
    eprintln!("{}", "dry run: nothing will be written".yellow().bold());
    for (idx, file) in rendered.iter().enumerate() {
        let mode = sources.get(idx).map(|s| s.mode()).unwrap_or_default();
        if mode == spark::FileMode::Overwrite {
            println!("{} {}", "==>".blue().bold(), file.path.bold());
        } else {
            println!(
                "{} {} {}",
                "==>".blue().bold(),
                file.path.bold(),
                format!("({mode:?} fragment)").yellow()
            );
        }
        println!("{}", file.content);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn github_spec_without_ref() {
        let (slug, reference, name) = parse_github_spec("pwnxpl0it/spark").unwrap();
        assert_eq!(slug, "pwnxpl0it/spark");
        assert_eq!(reference, None);
        assert_eq!(name, "spark");
    }

    #[test]
    fn github_spec_with_ref_and_dot_git() {
        let (slug, reference, name) = parse_github_spec("pwnxpl0it/spark.git@v2.0.1").unwrap();
        assert_eq!(slug, "pwnxpl0it/spark");
        assert_eq!(reference.as_deref(), Some("v2.0.1"));
        assert_eq!(name, "spark");
    }

    #[test]
    fn github_spec_rejects_missing_owner_or_repo() {
        assert!(parse_github_spec("justname").is_err());
        assert!(parse_github_spec("owner/").is_err());
        assert!(parse_github_spec("/repo").is_err());
        assert!(parse_github_spec("a/b/c").is_err());
        assert!(parse_github_spec("owner/repo@").is_err());
    }

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
        let dir_tmp = tempfile::tempdir().unwrap();
        let dir = dir_tmp.path().to_path_buf();
        fs::create_dir_all(dir.join("templates")).unwrap();
        fs::write(dir.join("templates/demo.toml"), "").unwrap();
        let config = Config::new(&dir.join("config.toml").to_string_lossy());

        let found = resolve_template_path(&config, "demo").unwrap();
        assert_eq!(found, dir.join("templates/demo.toml"));

        let err = resolve_template_path(&config, "missing").unwrap_err();
        assert!(err.to_string().contains("not found"));
    }

    #[test]
    fn list_templates_reads_toml_files_sorted_with_descriptions() {
        let dir_tmp = tempfile::tempdir().unwrap();
        let dir = dir_tmp.path().to_path_buf();
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
    }
}
