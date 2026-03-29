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
    style: menu::CodegenStyle,
    quiet: bool,
    out: PathBuf,
    json: Option<PathBuf>,
    algorithm: menu::Algorithm,
) -> Result<(), String> {
    // Quick check to prevent definitely failing to write later
    if let Some(path) = &json {
        if path.is_dir() {
            return Err(format!(
                "{} invalid json path '{}' (path is directory)",
                Red.bold().paint("error:"),
                path.to_str().unwrap()
            ));
        }
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() && !parent.exists() {
                return Err(format!(
                    "{} invalid json path '{}' (parent '{}' does not exist)",
                    Red.bold().paint("error:"),
                    path.to_str().unwrap(),
                    parent.to_str().unwrap(),
                ));
            }
        }
    }

    let problem = load_problem(library, program)?;
    let gen = style.codegen(problem.library.clone())?;

    let timer = util::Timer::infinite();
    let mut controller = algorithm.controller(timer, problem, false);

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
                                codegen::Simple::single(
                                    &top_down::Sketch::App(f, args),
                                ),
                            ))
                        )
                    }
                    top_down::TopDownStep::Seq(_, _) => {
                        fn summarize_seq(
                            step: &top_down::TopDownStep<
                                core::ParameterizedFunction,
                            >,
                        ) -> Option<String> {
                            // Collect all Extend steps in the Seq
                            fn collect_extends(
                                step: &top_down::TopDownStep<
                                    core::ParameterizedFunction,
                                >,
                                out: &mut Vec<core::ParameterizedFunction>,
                            ) {
                                match step {
                                    top_down::TopDownStep::Extend(_, f, _) => {
                                        out.push(f.clone());
                                    }
                                    top_down::TopDownStep::Seq(a, b) => {
                                        collect_extends(a, out);
                                        collect_extends(b, out);
                                    }
                                }
                            }
                            let mut fns = Vec::new();
                            collect_extends(step, &mut fns);

                            // Count defaults and find the final choice
                            let num_defaults = fns.iter()
                                .filter(|f| f.name.0 == "choose_default_par_factor")
                                .count();
                            let choice = fns.iter().rev()
                                .find(|f| f.name.0.starts_with("choose_par_factor_"))?;
                            let par_factor = choice.metadata
                                .get(&core::MetParam("par_factor".to_owned()))?;
                            let stream_level = choice.metadata
                                .get(&core::MetParam("stream_level".to_owned()))?;

                            let par_val = match par_factor {
                                core::Value::Int(n) => n.to_string(),
                                other => format!("{:?}", other),
                            };
                            let level_val = match stream_level {
                                core::Value::Int(n) => n.to_string(),
                                other => format!("{:?}", other),
                            };

                            let mut desc = format!(
                                "loop i{}: par_factor = {}",
                                level_val, par_val,
                            );
                            if num_defaults > 0 {
                                // Collect which loops are defaulted
                                let defaulted: Vec<String> = fns.iter()
                                    .filter(|f| f.name.0.starts_with("advance_par_factor_level_"))
                                    .filter_map(|f| f.metadata.get(
                                        &core::MetParam("stream_level".to_owned())
                                    ))
                                    .map(|v| match v {
                                        core::Value::Int(n) => format!("i{}", n),
                                        other => format!("{:?}", other),
                                    })
                                    .collect();
                                desc.push_str(&format!(
                                    "  ({} → 1)",
                                    defaulted.join(", "),
                                ));
                            }
                            Some(desc)
                        }

                        match summarize_seq(&option) {
                            Some(summary) => println!(
                                "{}",
                                Yellow.paint(summary),
                            ),
                            None => {
                                // Fallback: show raw names
                                fn collect_names(
                                    step: &top_down::TopDownStep<
                                        core::ParameterizedFunction,
                                    >,
                                    names: &mut Vec<String>,
                                ) {
                                    match step {
                                        top_down::TopDownStep::Extend(_, f, _) => {
                                            names.push(f.name.0.clone());
                                        }
                                        top_down::TopDownStep::Seq(a, b) => {
                                            collect_names(a, names);
                                            collect_names(b, names);
                                        }
                                    }
                                }
                                let mut names = Vec::new();
                                collect_names(&option, &mut names);
                                println!(
                                    "{}",
                                    Yellow.paint(format!(
                                        "[{}]",
                                        names.join(" → "),
                                    ))
                                )
                            }
                        }
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

    let output = gen.exp(&controller.working_expression())?;

    if quiet {
        println!("output: {}", output);
    } else {
        println!(
            "\n{}\n\n{}",
            Green.bold().paint("Final expression:"),
            output
        );
    }

    match write_file(out, &output) {
        Ok(()) => (),
        Err(e) => eprintln!("file write error: {}", e),
    };

    if let Some(json) = json {
        let contents = unparse::exp(&controller.working_expression())?;
        match write_file(json, &contents) {
            Ok(()) => (),
            Err(e) => eprintln!("file write error: {}\njson:\n{}", e, contents),
        };
    }

    Ok(())
}

/// Check if a Honeybee problem is solvable
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

/// Check if a Honeybee library is parseable and well-typed
pub fn validate(library: PathBuf) -> Result<(), String> {
    let lib_string = std::fs::read_to_string(library).map_err(|e| {
        format!("error while reading library file: {}", e.to_string())
    })?;

    let library = parse::library(&lib_string).map_err(|e| {
        format!("{}\n{}", Red.bold().paint("parse error (library):"), e)
    })?;

    typecheck::library(&library).map_err(|e| {
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

    println!("{}", Green.bold().paint("library validated!"));

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
