//! # Top-level handlers
//!
//! This module handles the top-level commands that Honeybee provides

use crate::*;

use ansi_term::Color::*;
use codegen::Codegen;
use instant::Duration;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn write_file(path: PathBuf, s: &str) -> Result<(), String> {
    match File::create(path) {
        Ok(mut file) => write!(file, "{}", s).map_err(|e| e.to_string()),
        Err(e) => Err(e.to_string()),
    }
}

fn load_problem(
    library: PathBuf,
    program: PathBuf,
) -> Result<core::Problem, String> {
    let lib_string = std::fs::read_to_string(library).map_err(|e| {
        format!("error while reading library file: {}", e.to_string())
    })?;
    let prog_string = std::fs::read_to_string(program).map_err(|e| {
        format!("error while reading program file: {}", e.to_string())
    })?;

    let library = parse::library(&lib_string).map_err(|e| {
        format!("{}\n{}", Red.bold().paint("parse error (library):"), e)
    })?;

    let program = parse::program(&prog_string).map_err(|e| {
        format!("{}\n{}", Red.bold().paint("parse error (program):"), e)
    })?;

    let problem = core::Problem { library, program };

    typecheck::problem(&problem).map_err(|e| {
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

    Ok(problem)
}

/// Use Programming By Navigation interactively
pub fn interact(
    library: PathBuf,
    program: PathBuf,
    quiet: bool,
    json: Option<PathBuf>,
    algorithm: menu::Algorithm,
) -> Result<(), String> {
    if let Some(path) = &json {
        let ok = match path.parent() {
            Some(parent) => parent.exists(),
            None => false,
        };
        if !ok {
            return Err(format!(
                "{} invalid json path '{}'",
                Red.bold().paint("error:"),
                path.to_str().unwrap()
            ));
        }
    }

    let problem = load_problem(library, program)?;

    let lib = problem.library.clone();

    let gen: Box<dyn Codegen> = if true {
        Box::new(codegen::Full::new(&lib)?)
    } else {
        Box::new(codegen::Simple {
            indent: 1,
            color: true,
        })
    };

    let timer = util::Timer::infinite();
    let mut controller = algorithm.controller(timer, problem);

    let mut round = 0;
    while !controller.valid() {
        round += 1;

        let mut options = controller.provide().unwrap();

        if options.is_empty() {
            if !quiet {
                println!("{}", Red.bold().paint("Not possible!"));
            }
            return Ok(());
        }

        if !quiet {
            println!(
                "{}\n\n{}\n\n{}\n\n{}\n",
                Fixed(8).paint(format!(
                    "══ Round {} {}",
                    round,
                    "═".repeat(40)
                )),
                Cyan.bold().paint("Working expression:"),
                gen.exp(&controller.working_expression())?,
                Cyan.bold().paint("Possible next steps:"),
            );
        }

        if quiet {
            println!("option count: {}", options.len())
        } else {
            for (i, option) in options.iter().cloned().enumerate() {
                print!("  {}) ", i + 1);
                match option {
                    top_down::TopDownStep::Extend(h, f, args) => {
                        println!(
                            "{}",
                            Yellow.paint(format!(
                                "{} ↦ {}",
                                top_down::pretty_hole_string(h),
                                codegen::simple_single(&top_down::Sketch::App(
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
        }

        let idx = loop {
            if !quiet {
                print!(
                    "\n{} {}\n\n> ",
                    Purple.bold().paint("Which step would you like to take?"),
                    Fixed(8).paint("('q' to quit)"),
                );
                std::io::stdout().flush().unwrap();
            }

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

    if quiet {
        println!("output: {}", gen.exp(&controller.working_expression())?);
    } else {
        println!(
            "\n{}\n\n{}",
            Green.bold().paint("Final expression:"),
            gen.exp(&controller.working_expression())?
        );
    }

    if let Some(json) = json {
        let contents = unparse::exp(&controller.working_expression())?;
        match write_file(json, &contents) {
            Ok(()) => (),
            Err(e) => eprintln!("file write error: {}\njson:\n{}", e, contents),
        };
    }

    Ok(())
}

/// Check if a Honeybee poblem is solvable
pub fn check(library: PathBuf, program: PathBuf) -> Result<(), String> {
    let problem = load_problem(library, program)?;
    let chosen_metadata = problem.program.goal.args.clone();
    let engine = egglog::Egglog::new(true);
    let mut oracle = dl_oracle::Oracle::new(engine, problem).unwrap();
    let vgm = oracle.valid_goal_metadata();
    if vgm.contains(&chosen_metadata) {
        println!("{}", Green.bold().paint("Solvable!"));
    } else {
        println!("{}", Red.bold().paint("Not solvable..."));
    }
    Ok(())
}

/// Translate a serialized json file to a Python program
pub fn translate(path: PathBuf, print_size: bool) -> Result<(), String> {
    let exp_string =
        std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let exp = parse::exp(&exp_string)?;
    let gen = codegen::Simple {
        indent: 0,
        color: false,
    };
    println!("{}", gen.exp(&exp)?);
    if print_size {
        println!("# size: {}", exp.size());
    }
    Ok(())
}

/// Benchmark the synthesizers in this project
pub fn benchmark(
    suite_paths: Vec<PathBuf>,
    algorithms: Vec<menu::Algorithm>,
    replicates: usize,
    timeout_secs: u64,
    entry_filter: String,
    parallel: bool,
    particular_solution_limit: usize,
) -> Result<(), String> {
    let config = benchmark::Config {
        replicates,
        timeout: Duration::from_secs(timeout_secs),
        entry_filter,
        parallel,
        algorithms,
        particular_solution_limit,
    };
    let runner = benchmark::Runner::new(config, std::io::stdout());
    runner.suites(&suite_paths);
    Ok(())
}
