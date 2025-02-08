#![allow(dead_code)]

use honeybee::*;

use ansi_term::Color::*;
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

        let lib =
            toml::from_str::<core::Library>(&lib_string).map_err(|e| {
                format!("{}\n{}", Red.bold().paint("parse error (library):"), e)
            })?;

        let prog =
            toml::from_str::<core::Program>(&prog_string).map_err(|e| {
                format!("{}\n{}", Red.bold().paint("parse error (program):"), e)
            })?;

        let problem = core::Problem::new(lib, prog).map_err(|e| {
            format!(
                "{} {}\n  occurred:{}",
                Red.bold().paint("type error:"),
                ansi_term::Style::new().bold().paint(e.message),
                e.context
                    .into_iter()
                    .map(|ctx| format!("\n    - in {}", ctx))
                    .collect::<Vec<_>>()
                    .join("")
            )
        })?;

        let engine = egglog::Egglog::new(true);
        let oracle = dl_oracle::Oracle::new(engine, problem).unwrap();
        let ccs = top_down::ClassicalConstructiveSynthesis { oracle };
        let start = top_down::Sketch::blank();
        let checker = top_down::GroundChecker::new();
        let mut controller = pbn::Controller::new(
            util::InfiniteTimer::new(),
            ccs,
            checker,
            start,
        );
        while !controller.valid() {
            let options = util::ok(controller.provide());
            println!("{:?}", options);
            break;
        }

        Ok(())
    }
}

fn main() {
    println!("Done!");

    let cli = Cli::parse();

    match cli.command.handle() {
        Ok(()) => (),
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1)
        }
    }
}
