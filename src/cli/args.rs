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
    #[arg(short = 'c', long, default_value = "~/.config/spark/config.toml")]
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

    /// Key, value pairs to be replaced.
    /// You can use this to skip user inputs and other function calls.
    /// Example: 'name=spark, author=pwnxpl0it'
    #[arg(long = "from", requires = "template", value_name = "PAIRS")]
    pub keywords: Option<String>,

    /// Fail when a placeholder has no value instead of writing an empty string
    #[arg(long, requires = "template")]
    pub strict: bool,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Creates a template for the current directory
    Init,
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
        assert!(cli.quiet);
        assert_eq!(cli.keywords.as_deref(), Some("name=spark,author=me"));
        assert_eq!(cli.json.as_deref().unwrap().to_str(), Some("data.json"));
        assert_eq!(cli.config, "/tmp/config.toml");
    }

    #[test]
    fn default_config_path() {
        let cli = Cli::try_parse_from(["spark", "demo"]).unwrap();
        assert_eq!(cli.config, "~/.config/spark/config.toml");
    }

    #[test]
    fn parses_init_subcommand() {
        let cli = Cli::try_parse_from(["spark", "init"]).unwrap();
        assert!(matches!(cli.command, Some(Commands::Init)));
        assert!(cli.template.is_none());
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
}
