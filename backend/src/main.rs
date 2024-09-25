#![allow(dead_code)]

mod analysis;
mod backend;
mod benchmark;
mod derivation;
mod egglog_adapter;
mod enumerate;
mod ir;
mod pbn;
mod run;
mod syntax;
mod synthesis;
mod task;
mod util;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Programming by Navigation with 🐝 Honeybee
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run Honeybee interactively in the CLI
    Run {
        /// The library file to use (.hblib)
        #[arg(short, long, value_name = "FILE")]
        library: PathBuf,

        /// The library implementation file to use (.py)
        #[arg(short, long, value_name = "FILE")]
        implementation: PathBuf,

        /// The Honeybee program to use (.hb)
        #[arg(short, long, value_name = "FILE")]
        program: PathBuf,

        /// Whether to output JSON of synthesized derivation tree (same directory as program)
        #[arg(short, long, value_name = "BOOL", default_value_t = false)]
        json: bool,
    },
    /// Benchmark Honeybee and baselines
    Benchmark {
        /// The benchmark suite directory to use
        #[arg(short, long, value_name = "DIR")]
        suite: PathBuf,

        /// The number of times to run each benchmark entry
        #[arg(short, long, value_name = "N", default_value_t = 1)]
        run_count: usize,

        /// The (soft) time cutoff to use for synthesis in milliseconds
        #[arg(
            short,
            long,
            value_name = "MILLISECONDS",
            default_value_t = 2000
        )]
        timeout: u128,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = match &cli.command {
        Commands::Run {
            library,
            implementation,
            program,
            json,
        } => run::run(library, implementation, program, *json),
        Commands::Benchmark {
            suite,
            run_count,
            timeout,
        } => benchmark::run(suite, *run_count, *timeout),
    };

    match result {
        Ok(()) => (),
        Err(e) => {
            println!("{} {}", ansi_term::Color::Red.bold().paint("error:"), e);
            std::process::exit(1)
        }
    }
}
