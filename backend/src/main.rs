use honeybee::main_handler;

use ansi_term::Color::*;
use clap::{builder::styling::*, Parser, Subcommand};
use std::path::PathBuf;

mod custom_parse {
    use honeybee::menu;
    use std::path::PathBuf;

    pub fn at_most_one_path(s: &str) -> Option<PathBuf> {
        if s.is_empty() {
            None
        } else {
            Some(PathBuf::from(s))
        }
    }

    pub fn one_or_more_paths(
        s: &str,
        option: &str,
    ) -> Result<Vec<PathBuf>, String> {
        if s.is_empty() {
            Err(format!("{} must be nonempty", option))
        } else {
            Ok(s.split(",").map(PathBuf::from).collect())
        }
    }

    pub fn algs(s: &str) -> Vec<menu::Algorithm> {
        if s.is_empty() {
            menu::Algorithm::all()
        } else {
            s.split(",").map(|s| s.parse().unwrap()).collect()
        }
    }

    pub fn limit(s: &str) -> usize {
        s.parse::<usize>().unwrap_or(usize::MAX)
    }
}

fn styles() -> Styles {
    Styles::styled()
        .header(AnsiColor::Green.on_default().bold())
        .usage(AnsiColor::Green.on_default().bold())
        .literal(AnsiColor::Cyan.on_default().bold())
        .placeholder(AnsiColor::Cyan.on_default())
        .valid(AnsiColor::Green.on_default())
        .invalid(AnsiColor::Yellow.on_default())
}

#[derive(Parser)]
#[command(
    version,
    about = format!("{} with {}",
        Purple.bold().paint("Programming by Navigation"),
        Yellow.bold().paint("🐝 Honeybee"),
    ),
    long_about = None,
    styles = styles(),
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Run Honeybee interactively in the CLI
    Interact {
        /// The library file to use (.hblib.toml)
        #[arg(short, long, value_name = "FILE")]
        library: PathBuf,

        /// The Honeybee program to use (.hb.toml)
        #[arg(short, long, value_name = "FILE")]
        program: PathBuf,

        /// The codegen style to use
        #[arg(
            short,
            long,
            value_name = "STYLE",
            default_value = "PlainTextNotebook"
        )]
        style: honeybee::menu::CodegenStyle,

        /// Whether or not to use "quiet" mode
        #[arg(short, long, action)]
        quiet: bool,

        /// Path to output JSON of synthesized expression (blank for no output)
        #[arg(short, long, value_name = "FILE", default_value = "")]
        json: String,

        /// The algorithm to use
        #[arg(
            short,
            long,
            value_name = "ALGORITHM",
            default_value = "PBNHoneybee"
        )]
        algorithm: honeybee::menu::Algorithm,
    },

    /// Check if a Honeybee problem is solvable
    Check {
        /// The library file to use (.hblib.toml)
        #[arg(short, long, value_name = "FILE")]
        library: PathBuf,

        /// The Honeybee program to use (.hb.toml)
        #[arg(short, long, value_name = "FILE")]
        program: PathBuf,
    },

    /// Run a benchmark suite
    Benchmark {
        /// The benchmark suite directories to use (comma-separated list)
        #[arg(short, long, value_name = "DIRS")]
        suite: String,

        /// Algorithms to use (comma-separated list, blank for all)
        #[arg(short, long, value_name = "ALGORITHMS", default_value = "")]
        algorithms: String,

        /// The number of times to run each benchmark entry
        #[arg(short, long, value_name = "N", default_value_t = 1)]
        replicates: usize,

        /// The (soft) time cutoff to use for synthesis (in seconds)
        #[arg(short, long, value_name = "SECONDS", default_value_t = 2)]
        timeout: u64,

        /// Filter to benchmark entries that contain this substring
        #[arg(short, long, value_name = "SUBSTRING", default_value = "")]
        filter: String,

        /// Set the maximum number of particular solutions to use (blank for no limit)
        #[arg(short, long, value_name = "N", default_value = "")]
        limit: String,

        /// Run benchmarks in parallel (for approximation only)
        #[arg(short, long, value_name = "BOOL", default_value_t = false)]
        parallel: bool,
    },

    /// Translate serialized JSON to Python expression
    Translate {
        /// Path to serialized JSON
        #[arg(short, long, value_name = "FILE")]
        path: PathBuf,

        /// Whether or not to print the size as a comment at the end of the file
        #[arg(short, long, value_name = "BOOL", default_value_t = false)]
        size: bool,
    },
}

impl Command {
    pub fn handle(self) -> Result<(), String> {
        match self {
            Self::Interact {
                library,
                program,
                style,
                quiet,
                json,
                algorithm,
            } => main_handler::interact(
                library,
                program,
                style,
                quiet,
                custom_parse::at_most_one_path(&json),
                algorithm,
            ),
            Self::Check { library, program } => {
                main_handler::check(library, program)
            }
            Self::Benchmark {
                suite,
                algorithms,
                replicates,
                timeout,
                filter,
                parallel,
                limit,
            } => main_handler::benchmark(
                custom_parse::one_or_more_paths(&suite, "--suite")?,
                custom_parse::algs(&algorithms),
                replicates,
                timeout,
                filter,
                parallel,
                custom_parse::limit(&limit),
            ),
            Self::Translate { path, size } => {
                main_handler::translate(path, size)
            }
        }
    }
}

fn main() {
    env_logger::init();

    let cli = Cli::parse();

    let result = cli.command.handle();

    match result {
        Ok(()) => (),
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1)
        }
    }
}
