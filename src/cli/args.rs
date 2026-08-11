use clap::{App, Arg, Command};

pub struct Cli;

impl Cli {
    pub fn app() -> App<'static> {
        App::new("spark")
            .about("A fast and flexible project initializer using TOML-based templates.")
            .version("2.0")
            .author("Mohamed Tarek @pwnxpl0it")
            .arg(
                Arg::new("template")
                    .help("Template used to generate files")
                    .takes_value(true)
                    .index(1),
            )
            .arg(
                Arg::new("quiet")
                    .help("Hide information of the template")
                    .short('q')
                    .requires("template"),
            )
            .arg(
                Arg::new("config")
                    .long("config")
                    .short('c')
                    .help("Config path")
                    .default_value("~/.config/spark/config.toml")
                    .requires("template"),
            )
            .arg(
                Arg::new("json")
                    .help("Read key,value pairs from a JSON file")
                    .long("json")
                    .takes_value(true)
                    .requires("template"),
            )
            .arg(
                Arg::new("git")
                    .help("Initialize a git repo regardless of template options")
                    .long("git")
                    .takes_value(false)
                    .requires("template"),
            )
            .arg(
                Arg::new("no-liquid")
                    .help("Disable Liquid support")
                    .long("no-liquid")
                    .takes_value(false)
                    .requires("template"),
            )
            .arg(
                Arg::new("keywords")
                .help("Key, value pairs to be replaced,\nYou can use this to skip user inputs and other function calls,\nExample: 'name=spark, author=pwnxpl0it'")
                .long("from")
                .takes_value(true)
                .requires("template")
            )
            .subcommand(Command::new("init").about("Creates a template for the current directory"))
    }

    pub fn parse() -> clap::ArgMatches {
        Self::app().get_matches()
    }
}

#[cfg(test)]
mod tests {
    use super::Cli;

    #[test]
    fn parses_template_argument() {
        let matches = Cli::app()
            .try_get_matches_from(["spark", "browser_extension"])
            .unwrap();
        assert_eq!(matches.value_of("template"), Some("browser_extension"));
    }

    #[test]
    fn parses_flags_with_template() {
        let matches = Cli::app()
            .try_get_matches_from([
                "spark",
                "demo",
                "--git",
                "--no-liquid",
                "-q",
                "--from",
                "name=spark,author=me",
                "--json",
                "data.json",
                "-c",
                "/tmp/config.toml",
            ])
            .unwrap();

        assert_eq!(matches.value_of("template"), Some("demo"));
        assert!(matches.is_present("git"));
        assert!(matches.is_present("no-liquid"));
        assert!(matches.is_present("quiet"));
        assert_eq!(matches.value_of("keywords"), Some("name=spark,author=me"));
        assert_eq!(matches.value_of("json"), Some("data.json"));
        assert_eq!(matches.value_of("config"), Some("/tmp/config.toml"));
    }

    #[test]
    fn default_config_path() {
        let matches = Cli::app().try_get_matches_from(["spark", "demo"]).unwrap();
        assert_eq!(
            matches.value_of("config"),
            Some("~/.config/spark/config.toml")
        );
    }

    #[test]
    fn parses_init_subcommand() {
        let matches = Cli::app().try_get_matches_from(["spark", "init"]).unwrap();
        assert!(matches.subcommand_matches("init").is_some());
    }

    #[test]
    fn rejects_git_without_template() {
        let result = Cli::app().try_get_matches_from(["spark", "--git"]);
        assert!(result.is_err());
    }

    /// Scenario 5 — `--from="PROJECTNAME=myapp"` is parsed and the caller can
    /// build the keywords map from it (skipping the interactive prompt).
    ///
    /// This mirrors the logic in `main()`:
    ///   let pairs = args.value_of("keywords").unwrap();
    ///   for key in pairs.split(',') {
    ///       let (keyword, value) = key.split_once('=').unwrap();
    ///       keywords.insert(Keywords::from(keyword.trim(), None), value.trim());
    ///   }
    #[test]
    fn from_flag_projectname_populates_keywords_map() {
        use std::collections::HashMap;

        let matches = Cli::app()
            .try_get_matches_from(["spark", "demo", "--from", "PROJECTNAME=myapp"])
            .unwrap();

        // The raw string is correctly captured.
        let raw = matches.value_of("keywords").unwrap();
        assert_eq!(raw, "PROJECTNAME=myapp");

        // Simulate the main() keyword-expansion loop.
        let mut keywords: HashMap<String, String> = HashMap::new();
        for pair in raw.split(',') {
            let (k, v) = pair.split_once('=').unwrap();
            keywords.insert(format!("{{{{${}}}}}", k.trim()), v.trim().to_string());
        }

        // The keyword is now pre-set — extract() will use it without prompting.
        assert_eq!(
            keywords.get("{{$PROJECTNAME}}").map(String::as_str),
            Some("myapp"),
            "--from flag should pre-populate {{$PROJECTNAME}} so no prompt is needed"
        );
    }

    /// `--from` also works with multiple comma-separated key=value pairs.
    #[test]
    fn from_flag_multiple_pairs_all_populated() {
        use std::collections::HashMap;

        let matches = Cli::app()
            .try_get_matches_from([
                "spark",
                "demo",
                "--from",
                "PROJECTNAME=myapp, author=alice, version=1.0",
            ])
            .unwrap();

        let raw = matches.value_of("keywords").unwrap();
        let mut keywords: HashMap<String, String> = HashMap::new();
        for pair in raw.split(',') {
            let (k, v) = pair.split_once('=').unwrap();
            keywords.insert(format!("{{{{${}}}}}", k.trim()), v.trim().to_string());
        }

        assert_eq!(
            keywords.get("{{$PROJECTNAME}}").map(String::as_str),
            Some("myapp")
        );
        assert_eq!(
            keywords.get("{{$author}}").map(String::as_str),
            Some("alice")
        );
        assert_eq!(
            keywords.get("{{$version}}").map(String::as_str),
            Some("1.0")
        );
    }
}
