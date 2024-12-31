use clap::{App, Arg, Command};

pub struct Cli {}

impl Cli {
    pub fn parse() -> clap::ArgMatches {
        App::new("spark")
            .about("A fast and flexible project initializer using TOML-based templates. Automate project setup, file generation, and reporting workflows with JSON input, dynamic placeholders, and optional Liquid support.")
            .version("2.0")
            .author("Mohamed Tarek @pwnxpl0it")
            .arg(
                Arg::with_name("template")
                    .help("Template used to generate files")
                    .takes_value(true)
                    .index(1),
            )
            .arg(
                Arg::with_name("quiet")
                    .help("Hide information of the template")
                    .short('q')
                    .requires("template"),
            )
            .arg(
                Arg::with_name("config")
                    .long("config")
                    .short('c')
                    .help("Config path")
                    .default_value("~/.config/spark/config.toml")
                    .requires("template"),
            )
            .arg(
                Arg::with_name("json")
                    .help("read key,value pairs from a json file")
                    .long("json")
                    .takes_value(true)
                    .requires("template"),
            )
            .arg(
                Arg::with_name("git")
                    .help("Initialize a git repo, this works regardless of template options")
                    .long("git")
                    .takes_value(false)
                    .requires("template"),
            )
            .arg(
                Arg::with_name("no-liquid")
                    .help("Disable Liquid support")
                    .long("no-liquid")
                    .takes_value(false)
                    .requires("template"),
            )
            .subcommand(Command::new("init").about("Creates a template for the current directory"))
            .get_matches()
    }
}
