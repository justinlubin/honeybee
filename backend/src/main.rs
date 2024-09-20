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
    },
    /// Benchmark Honeybee and baselines
    Benchmark {
        /// The benchmark suite directory to use
        #[arg(short, long, value_name = "DIR")]
        suite: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = match &cli.command {
        Commands::Run {
            library,
            implementation,
            program,
        } => run::run(library, implementation, program),
        Commands::Benchmark { suite } => benchmark::run(suite),
    };

    match result {
        Ok(_) => (),
        Err(_) => std::process::exit(1),
    }
}
