#![allow(dead_code)]

use honeybee::*;

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
        /// The benchmark suite directories to use (comma-separated list)
        #[arg(short, long, value_name = "DIRS")]
        suite: String,

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

        /// Filter to benchmark entries that contain this substring
        #[arg(short, long, value_name = "SUBSTRING", default_value = "")]
        filter: String,

        /// Use a quick (parallel) approximation - not for publication use
        #[arg(short, long, value_name = "BOOL", default_value_t = false)]
        quick: bool,

        /// Use only certain algorithms (e.g. EP,PBN_DLmem)
        #[arg(short, long, value_name = "A1,A2,...", default_value = "")]
        algorithms: String,

        /// Solve only certain tasks (e.g. Any,Particular)
        #[arg(long, value_name = "T1,T2,...", default_value = "")]
        tasks: String,

        /// Show results for each benchmark row (results in invalid TSV)
        #[arg(long, value_name = "BOOL", default_value_t = false)]
        show_results: bool,
    },
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

fn parse_algorithms(s: &str) -> Vec<benchmark_data::Algorithm> {
    if s.is_empty() {
        benchmark_data::ALGORITHMS.to_vec()
    } else {
        s.split(",")
            .map(|x| serde_json::from_str(&format!("\"{}\"", x)).unwrap())
            .collect()
    }
}

fn parse_tasks(s: &str) -> Vec<benchmark_data::Task> {
    if s.is_empty() {
        benchmark_data::TASKS.to_vec()
    } else {
        s.split(",")
            .map(|x| serde_json::from_str(&format!("\"{}\"", x)).unwrap())
            .collect()
    }
}

fn main() {
    env_logger::init();

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
            filter,
            quick,
            algorithms,
            tasks,
            show_results,
        } => {
            let suites = parse_suites(suite);
            let algorithms = parse_algorithms(&algorithms);
            let tasks = parse_tasks(&tasks);
            benchmark::run(
                &suites,
                *run_count,
                *timeout,
                filter,
                true,
                *quick,
                *show_results,
                &algorithms,
                &tasks,
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
