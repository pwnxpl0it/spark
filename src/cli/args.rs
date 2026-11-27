use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// A fast and flexible project initializer using TOML-based templates.
#[derive(Debug, Parser)]
#[command(name = "spark", version, author = "Mohamed Tarek @pwnxpl0it")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Template used to generate files (a path, or a name in the templates dir)
    pub template: Option<String>,

    /// Hide information of the template
    #[arg(short = 'q', long, requires = "template")]
    pub quiet: bool,

    /// Config path
    #[arg(short = 'c', long, global = true, default_value = crate::config::DEFAULT_CONFIG_PATH)]
    pub config: String,

    /// Read key,value pairs from a JSON file
    #[arg(long, requires = "template", value_name = "FILE")]
    pub json: Option<PathBuf>,

    /// Initialize a git repo regardless of template options
    #[arg(long, requires = "template")]
    pub git: bool,

    /// Enable Liquid rendering regardless of template options
    #[arg(long, requires = "template", conflicts_with = "no_liquid")]
    pub liquid: bool,

    /// Disable Liquid rendering regardless of template options
    #[arg(long, requires = "template")]
    pub no_liquid: bool,

    /// Key=value pairs to pre-set placeholders and skip prompts.
    /// Comma-separated, repeatable; a comma inside a value is kept when the
    /// text after it has no '='. Example: --from 'name=spark, tags=a,b' --from author=me
    #[arg(long = "from", requires = "template", value_name = "PAIRS", action = clap::ArgAction::Append)]
    pub keywords: Vec<String>,

    /// Fail when a placeholder has no value instead of writing an empty string
    #[arg(long, requires = "template")]
    pub strict: bool,

    /// Overwrite existing files without asking
    #[arg(short = 'f', long, requires = "template")]
    pub force: bool,

    /// Allow destination paths that escape the working directory: absolute paths
    /// (including '~'/'$VAR' that expand to one) or paths containing '..'
    #[arg(long, requires = "template")]
    pub allow_outside: bool,

    /// Render the template and print the result without writing any files
    #[arg(short = 'n', long, requires = "template")]
    pub dry_run: bool,

    /// Run the template's pre/post hooks (shell commands). Off by default:
    /// without this flag any hooks a template defines are skipped.
    #[arg(long, requires = "template")]
    pub run_hooks: bool,

    /// Directory to generate into (created if missing); relative file paths are resolved from here
    #[arg(short = 'o', long, requires = "template", value_name = "DIR")]
    pub output_dir: Option<PathBuf>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Creates a template for the current directory
    Init {
        /// Set `git = true` in the generated `[options]`
        #[arg(long)]
        git: bool,
        /// Set `use_liquid = true` in the generated `[options]`
        #[arg(long)]
        liquid: bool,
        /// Set `project_root` in the generated `[options]`
        #[arg(long, value_name = "VALUE")]
        project_root: Option<String>,
        /// Generate the template from a GitHub repo instead of the current
        /// directory. Accepts `owner/repo` or `owner/repo@ref` (branch or tag).
        #[arg(long, value_name = "OWNER/REPO")]
        from_github: Option<String>,
    },
    /// Lists the templates available in the templates directory
    List,
    /// Reverses the most recent extraction: deletes files Spark created and
    /// restores files it overwrote or patched
    Undo,
}

impl Cli {
    pub fn parse_args() -> Self {
        Self::parse()
    }
}

#[cfg(test)]
mod tests {
    use super::{Cli, Commands};
    use clap::Parser;

    #[test]
    fn parses_template_argument() {
        let cli = Cli::try_parse_from(["spark", "browser_extension"]).unwrap();
        assert_eq!(cli.template.as_deref(), Some("browser_extension"));
        assert!(cli.command.is_none());
    }

    #[test]
    fn parses_flags_with_template() {
        let cli = Cli::try_parse_from([
            "spark",
            "demo",
            "--git",
            "--no-liquid",
            "--strict",
            "--force",
            "--allow-outside",
            "--dry-run",
            "-o",
            "/tmp/out",
            "-q",
            "--from",
            "name=spark,author=me",
            "--json",
            "data.json",
            "-c",
            "/tmp/config.toml",
        ])
        .unwrap();

        assert_eq!(cli.template.as_deref(), Some("demo"));
        assert!(cli.git);
        assert!(cli.no_liquid);
        assert!(cli.strict);
        assert!(cli.force);
        assert!(cli.allow_outside);
        assert!(cli.dry_run);
        assert_eq!(
            cli.output_dir.as_deref().unwrap().to_str(),
            Some("/tmp/out")
        );
        assert!(cli.quiet);
        assert_eq!(cli.keywords, vec!["name=spark,author=me".to_string()]);
        assert_eq!(cli.json.as_deref().unwrap().to_str(), Some("data.json"));
        assert_eq!(cli.config, "/tmp/config.toml");
    }

    #[test]
    fn default_config_path() {
        let cli = Cli::try_parse_from(["spark", "demo"]).unwrap();
        assert_eq!(cli.config, crate::config::DEFAULT_CONFIG_PATH);
    }

    #[test]
    fn parses_init_subcommand() {
        let cli = Cli::try_parse_from(["spark", "init"]).unwrap();
        assert!(matches!(
            cli.command,
            Some(Commands::Init {
                git: false,
                liquid: false,
                project_root: None,
                from_github: None,
            })
        ));
        assert!(cli.template.is_none());
    }

    #[test]
    fn parses_init_with_options() {
        let cli = Cli::try_parse_from([
            "spark",
            "init",
            "--git",
            "--liquid",
            "--project-root",
            "{{$PROJECTNAME}}",
        ])
        .unwrap();
        match cli.command {
            Some(Commands::Init {
                git,
                liquid,
                project_root,
                from_github,
            }) => {
                assert!(git);
                assert!(liquid);
                assert_eq!(project_root.as_deref(), Some("{{$PROJECTNAME}}"));
                assert!(from_github.is_none());
            }
            other => panic!("expected Init, got {other:?}"),
        }
    }

    #[test]
    fn parses_init_from_github() {
        let cli = Cli::try_parse_from(["spark", "init", "--from-github", "owner/repo@v2"]).unwrap();
        match cli.command {
            Some(Commands::Init { from_github, .. }) => {
                assert_eq!(from_github.as_deref(), Some("owner/repo@v2"));
            }
            other => panic!("expected Init, got {other:?}"),
        }
    }

    #[test]
    fn rejects_git_without_template() {
        assert!(Cli::try_parse_from(["spark", "--git"]).is_err());
    }

    #[test]
    fn liquid_and_no_liquid_conflict() {
        assert!(Cli::try_parse_from(["spark", "demo", "--liquid", "--no-liquid"]).is_err());
        let cli = Cli::try_parse_from(["spark", "demo", "--liquid"]).unwrap();
        assert!(cli.liquid);
    }

    #[test]
    fn version_comes_from_cargo_manifest() {
        use clap::CommandFactory;
        let cmd = Cli::command();
        assert_eq!(cmd.get_version(), Some(env!("CARGO_PKG_VERSION")));
    }

    #[test]
    fn parses_list_subcommand() {
        let cli = Cli::try_parse_from(["spark", "list"]).unwrap();
        assert!(matches!(cli.command, Some(Commands::List)));
    }

    #[test]
    fn config_flag_works_after_subcommand() {
        let cli = Cli::try_parse_from(["spark", "list", "-c", "/tmp/x.toml"]).unwrap();
        assert!(matches!(cli.command, Some(Commands::List)));
        assert_eq!(cli.config, "/tmp/x.toml");
    }

    #[test]
    fn run_hooks_defaults_off_and_requires_template() {
        let cli = Cli::try_parse_from(["spark", "demo"]).unwrap();
        assert!(!cli.run_hooks);
        let cli = Cli::try_parse_from(["spark", "demo", "--run-hooks"]).unwrap();
        assert!(cli.run_hooks);
        assert!(Cli::try_parse_from(["spark", "--run-hooks"]).is_err());
    }

    #[test]
    fn from_flag_is_repeatable() {
        let cli = Cli::try_parse_from(["spark", "demo", "--from", "a=1", "--from", "b=2"]).unwrap();
        assert_eq!(cli.keywords, vec!["a=1".to_string(), "b=2".to_string()]);
    }

    #[test]
    fn parses_undo_subcommand() {
        let cli = Cli::try_parse_from(["spark", "undo"]).unwrap();
        assert!(matches!(cli.command, Some(Commands::Undo)));
        assert!(cli.template.is_none());
    }
}
