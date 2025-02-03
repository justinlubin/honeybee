#![allow(dead_code)]

use honeybee::*;

use clap::{builder::styling::*, Parser, Subcommand};
use std::path::PathBuf;
use toml;

fn styles() -> Styles {
    Styles::styled()
        .header(AnsiColor::Green.on_default().bold())
        .usage(AnsiColor::Green.on_default().bold())
        .literal(AnsiColor::Cyan.on_default().bold())
        .placeholder(AnsiColor::Cyan.on_default())
        // .error(AnsiColor::Red.on_default())
        .valid(AnsiColor::Green.on_default())
        .invalid(AnsiColor::Yellow.on_default())
}

#[derive(Parser)]
#[command(
    version,
    about = format!("{} with {}",
        ansi_term::Color::Purple.bold().paint("Programming by Navigation"),
        ansi_term::Color::Yellow.bold().paint("🐝 Honeybee"),
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
    },
}

impl Command {
    pub fn handle(self) -> Result<(), String> {
        match self {
            Self::Interact { library, program } => {
                Self::interact(library, program)
            }
        }
    }

    fn interact(library: PathBuf, program: PathBuf) -> Result<(), String> {
        let lib_string = std::fs::read_to_string(library).unwrap();
        let prog_string = std::fs::read_to_string(program).unwrap();

        let lib = toml::from_str::<core::Library>(&lib_string)
            .map_err(|e| format!("library error: {}", e))?;

        let prog = toml::from_str::<core::Program>(&prog_string)
            .map_err(|e| format!("program error: {}", e))?;

        let problem = core::Problem::new(lib, prog)
            .map_err(|e| format!("type error: {}", e))?;

        Ok(())
    }
}

fn main() {
    println!("Done!");

    let cli = Cli::parse();

    match cli.command.handle() {
        Ok(()) => (),
        Err(e) => {
            println!("{} {}", ansi_term::Color::Red.bold().paint("error:"), e);
            std::process::exit(1)
        }
    }
}
