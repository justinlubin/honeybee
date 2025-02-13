#![allow(dead_code)]

use honeybee::*;

use ansi_term::Color::*;
use clap::{builder::styling::*, Parser, Subcommand};
use std::path::PathBuf;

mod custom_parse {
    use std::path::PathBuf;

    pub fn nullable_path(s: &str) -> Option<PathBuf> {
        if s.is_empty() {
            None
        } else {
            Some(PathBuf::from(s))
        }
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

        /// Whether or not to use "quiet" mode
        #[arg(short, long, action)]
        quiet: bool,

        /// Path to output JSON of synthesized expression (blank for no output)
        #[arg(short, long, value_name = "FILE", default_value = "")]
        json: String,
    },

    /// Translate serialized JSON to Python expression
    Translate {
        /// Path to serialized JSON
        #[arg(short, long, value_name = "FILE")]
        path: PathBuf,
    },
}

use Command::*;

fn main() {
    env_logger::init();

    let cli = Cli::parse();

    let result = match cli.command {
        Interact {
            library,
            program,
            quiet,
            json,
        } => main_handler::interact(
            library,
            program,
            quiet,
            custom_parse::nullable_path(&json),
        ),
        Translate { path } => main_handler::translate(path),
    };

    match result {
        Ok(()) => (),
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1)
        }
    }
}
