use clap::{App, Arg, Command};

pub struct Cli;

impl Cli {
    pub fn parse() -> clap::ArgMatches {
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
            .get_matches()
    }
}
