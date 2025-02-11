#![allow(dead_code)]

use honeybee::*;

use ansi_term::Color::*;
use clap::{builder::styling::*, Parser, Subcommand};
use std::io::Write;
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

        let mut round = 0;
        while !controller.valid() {
            round += 1;

            let mut options = util::ok(controller.provide());

            if options.is_empty() {
                println!("{}", Red.bold().paint("Not possible!"));
                return Ok(());
            }

            println!(
                "{}\n\n{}\n\n  {}\n\n{}\n",
                Fixed(8).paint(format!(
                    "══ Round {} {}",
                    round,
                    "═".repeat(40)
                )),
                Cyan.bold().paint("Working expression:"),
                codegen::python_multi(&controller.working_expression(), 1),
                Cyan.bold().paint("Possible next steps:"),
            );

            for (i, option) in options.iter().cloned().enumerate() {
                print!("  {}) ", i + 1);
                match option {
                    top_down::TopDownStep::Extend(h, f, args) => {
                        println!(
                            "{}",
                            Green.paint(format!(
                                "{} ↦ {}",
                                top_down::pretty_hole_string(h),
                                codegen::python_single(&top_down::Sketch::App(
                                    f, args
                                ),),
                            ))
                        )
                    }
                    top_down::TopDownStep::Seq(_, _) => {
                        println!("<unexpected>")
                    }
                }
            }

            let idx = loop {
                print!(
                    "\n{} {}\n\n> ",
                    Purple.bold().paint("Which step would you like to take?"),
                    Fixed(8).paint("('q' to quit)"),
                );

                std::io::stdout().flush().unwrap();

                let mut input = String::new();
                std::io::stdin().read_line(&mut input).unwrap();
                let input = input.trim();

                if input == "q" {
                    return Ok(());
                }

                match input.parse::<usize>() {
                    Ok(choice) => {
                        if 1 <= choice && choice <= options.len() {
                            break choice - 1;
                        } else {
                            continue;
                        }
                    }
                    Err(_) => continue,
                };
            };

            controller.decide(options.swap_remove(idx))
        }

        println!(
            "\n{}\n\n  {}",
            Green.bold().paint("Final expression:"),
            codegen::python_multi(&controller.working_expression(), 1)
        );

        Ok(())
    }
}

fn main() {
    env_logger::init();

    let cli = Cli::parse();

    match cli.command.handle() {
        Ok(()) => (),
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1)
        }
    }
}
