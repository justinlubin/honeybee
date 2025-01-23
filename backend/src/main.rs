#![allow(dead_code)]

use honeybee::*;

use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use toml;

#[derive(Debug, Clone, Serialize, Deserialize, clap::ValueEnum)]
enum RunMode {
    Interactive,
    Auto,
    ProcessInteractive,
}

impl RunMode {
    pub fn analyzer(&self) -> analysis::CLI {
        match self {
            RunMode::Interactive => analysis::CLI {
                mode: analysis::CLIMode::Manual,
                print_mode: analysis::CLIPrintMode::Full,
            },
            RunMode::Auto => analysis::CLI {
                mode: analysis::CLIMode::Auto,
                print_mode: analysis::CLIPrintMode::NoPrint,
            },
            RunMode::ProcessInteractive => analysis::CLI {
                mode: analysis::CLIMode::Manual,
                print_mode: analysis::CLIPrintMode::LenPrint,
            },
        }
    }
}

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

        /// The run mode
        #[arg(short, long, value_name = "MODE", value_enum, default_value_t = RunMode::Interactive)]
        mode: RunMode,

        /// Path to output JSON of synthesized derivation tree (blank for no output)
        #[arg(short, long, value_name = "FILE", default_value = "")]
        json: String,
    },
    /// Benchmark Honeybee and baselines
    Benchmark {
        /// The benchmark suite directories to use (comma-separated list)
        #[arg(short, long, value_name = "DIRS")]
        suite: String,

        /// The number of times to run each benchmark entry
        #[arg(short, long, value_name = "N", default_value_t = 1)]
        replicates: usize,

        /// The (soft) time cutoff to use for synthesis in milliseconds
        #[arg(
            short,
            long,
            value_name = "MILLISECONDS",
            default_value_t = 2000
        )]
        timeout: u128,

        /// Filter to benchmark entries that contain this substring
        #[arg(short, long, value_name = "SUBSTRING", default_value = "")]
        filter: String,

        /// Use a quick (parallel) approximation - not for publication use
        #[arg(short, long, value_name = "BOOL", default_value_t = false)]
        quick: bool,

        /// Set the algorithms and tasks
        #[arg(
            short,
            long,
            value_name = "A1:T1,A2:T2,...",
            default_value = "PBN_DLmem:Particular,PBN_DL:Particular,EP:All,E:All"
        )]
        algotasks: String,

        /// Show results for each benchmark row (results in invalid TSV)
        #[arg(long, value_name = "BOOL", default_value_t = false)]
        show_results: bool,
    },
}

fn parse_json_path(s: &str) -> Option<PathBuf> {
    if s.is_empty() {
        None
    } else {
        Some(PathBuf::from(s))
    }
}

fn parse_suites(s: &str) -> Vec<PathBuf> {
    if s.is_empty() {
        println!(
            "{} {}",
            ansi_term::Color::Red.bold().paint("error:"),
            "--suite must be nonempty"
        );
        std::process::exit(1)
    } else {
        s.split(",").map(|x| PathBuf::from(x)).collect()
    }
}

fn parse_algotasks(
    s: &str,
) -> Vec<(benchmark_data::Algorithm, benchmark_data::Task)> {
    s.split(",")
        .map(|at| {
            let (a, t) = at
                .split_once(":")
                .expect(&format!("malformed algotask: {}", at));
            (
                serde_json::from_str(&format!("\"{}\"", a)).unwrap(),
                serde_json::from_str(&format!("\"{}\"", t)).unwrap(),
            )
        })
        .collect()
}

fn main() {
    env_logger::init();

    let libfile =
        std::fs::read_to_string("../benchmark/next/bio.hblib.toml").unwrap();
    let progfile =
        std::fs::read_to_string("../benchmark/next/prog.hb.toml").unwrap();

    match toml::from_str::<next::core::Library>(&libfile) {
        Ok(_) => (),
        Err(e) => {
            println!("library error: {}", e);
            return;
        }
    }

    match toml::from_str::<next::core::Program>(&progfile) {
        Ok(_) => (),
        Err(e) => {
            println!("program error: {}", e);
            return;
        }
    }

    println!("Done!");
    return;

    let cli = Cli::parse();

    let result = match &cli.command {
        Commands::Run {
            library,
            implementation,
            program,
            mode,
            json,
        } => {
            let json = parse_json_path(json);
            run::run(mode.analyzer(), library, implementation, program, &json)
        }
        Commands::Benchmark {
            suite,
            replicates,
            timeout,
            filter,
            quick,
            algotasks,
            show_results,
        } => {
            let suites = parse_suites(suite);
            let algotasks = parse_algotasks(&algotasks);
            benchmark::run(
                &suites,
                *replicates,
                *timeout,
                filter,
                true,
                *quick,
                *show_results,
                &algotasks,
            )
            .map(|_| ())
        }
    };

    match result {
        Ok(()) => (),
        Err(e) => {
            println!("{} {}", ansi_term::Color::Red.bold().paint("error:"), e);
            std::process::exit(1)
        }
    }
}
