use crate::ir::*;

use crate::derivation;
use crate::enumerate;
use crate::pbn;
use crate::syntax;

use chumsky::Parser;
use serde::Serialize;
use std::path::PathBuf;
use std::time::Instant;

#[derive(Debug, Clone, Serialize)]
enum Algorithm {
    BaselineEnumerative,
    ClassicalConstructiveDatalog,
}

#[derive(Debug, Clone, Serialize)]
enum Task {
    Any,
    All,
    Particular,
}

#[derive(Debug, Serialize)]
struct Record<'a> {
    suite: &'a str,
    entry: &'a str,
    task: Task,
    algorithm: Algorithm,
    completed: bool,
    duration: u128,
    solution_count: usize,
    output: &'a str,
}

struct SynthesisResult {
    completed: bool,
    duration: u128,
    solutions: Vec<derivation::Tree>,
}

fn run_one(
    lib: &Library,
    prog: &Program,
    particular: &derivation::Tree,
    task: Task,
    algorithm: Algorithm,
    soft_timeout: u128,
) -> SynthesisResult {
    match (task, algorithm) {
        (Task::Any, Algorithm::BaselineEnumerative) => {
            let now = Instant::now();

            let (solutions, completed) = enumerate::enumerate(
                lib,
                prog,
                enumerate::Mode::AnyValid,
                soft_timeout,
            );

            let duration = now.elapsed().as_millis();

            SynthesisResult {
                completed,
                duration,
                solutions,
            }
        }
        (Task::All, Algorithm::BaselineEnumerative) => {
            let now = Instant::now();

            let (solutions, completed) = enumerate::enumerate(
                lib,
                prog,
                enumerate::Mode::AllValid,
                soft_timeout,
            );

            let duration = now.elapsed().as_millis();

            SynthesisResult {
                completed,
                duration,
                solutions,
            }
        }
        (Task::Particular, Algorithm::BaselineEnumerative) => {
            let now = Instant::now();

            let (solutions, completed) = enumerate::enumerate(
                lib,
                prog,
                enumerate::Mode::Particular(particular),
                soft_timeout,
            );

            let duration = now.elapsed().as_millis();

            SynthesisResult {
                completed,
                duration,
                solutions,
            }
        }
        (Task::Any, Algorithm::ClassicalConstructiveDatalog) => {
            let now = Instant::now();

            let (solutions, completed) =
                pbn::enumerate(lib, prog, pbn::Mode::Any, soft_timeout);

            let duration = now.elapsed().as_millis();

            SynthesisResult {
                completed,
                duration,
                solutions,
            }
        }
        (Task::All, Algorithm::ClassicalConstructiveDatalog) => {
            let now = Instant::now();

            let (solutions, completed) =
                pbn::enumerate(lib, prog, pbn::Mode::All, soft_timeout);

            let duration = now.elapsed().as_millis();

            SynthesisResult {
                completed,
                duration,
                solutions,
            }
        }
        (Task::Particular, Algorithm::ClassicalConstructiveDatalog) => {
            let now = Instant::now();

            let (solutions, completed) = pbn::enumerate(
                lib,
                prog,
                pbn::Mode::Particular(particular),
                soft_timeout,
            );

            let duration = now.elapsed().as_millis();

            SynthesisResult {
                completed,
                duration,
                solutions,
            }
        }
    }
}

// Directory format:
// - suite_directory/
//   - _suite.hblib
//   - _suite.py
//   - some_benchmark_name.hb
//   - some_benchmark_name.txt (the particular solution)
//   - another_benchmark.hb
//   - another_benchmark.txt
//   - yet_another.hb
//   - yet_another.txt
//   - ...
pub fn run(
    suite_directory: &PathBuf,
    run_count: usize,
    soft_timeout: u128, // in milliseconds
) -> Result<(), Box<dyn std::error::Error>> {
    assert!(suite_directory.is_dir());

    let suite = suite_directory.file_name().unwrap().to_str().unwrap();
    let lib_filename = suite_directory.join("_suite.hblib");
    let imp_filename = suite_directory.join("_suite.py");

    let lib_src = std::fs::read_to_string(lib_filename).unwrap();
    let _imp_src = std::fs::read_to_string(imp_filename).unwrap();

    let lib = syntax::parse::library()
        .parse(lib_src)
        .map_err(|_| "Library parse error")?;

    let mut wtr = csv::Writer::from_writer(std::io::stdout());

    for prog_filename in
        glob::glob(suite_directory.join("*.hb").to_str().unwrap())
            .unwrap()
            .filter_map(Result::ok)
    {
        let prog_filename_without_extension = prog_filename.with_extension("");
        let entry = prog_filename_without_extension
            .file_name()
            .unwrap()
            .to_str()
            .unwrap();

        let prog_src = std::fs::read_to_string(&prog_filename).unwrap();
        let prog = syntax::parse::program()
            .parse(prog_src)
            .map_err(|_| "Program parse error")?;

        let particular_filename = prog_filename.with_extension("json");
        let particular_src =
            std::fs::read_to_string(&particular_filename).unwrap();

        let particular: derivation::Tree =
            serde_json::from_str(&particular_src).unwrap();

        for task in vec![Task::Any, Task::All, Task::Particular] {
            for algorithm in vec![
                Algorithm::BaselineEnumerative,
                Algorithm::ClassicalConstructiveDatalog,
            ] {
                for _ in 0..run_count {
                    let sr = run_one(
                        &lib,
                        &prog,
                        &particular,
                        task.clone(),
                        algorithm.clone(),
                        soft_timeout,
                    );

                    wtr.serialize(Record {
                        suite,
                        entry,
                        task: task.clone(),
                        algorithm: algorithm.clone(),
                        completed: sr.completed,
                        duration: sr.duration,
                        solution_count: sr.solutions.len(),
                        output: "...",
                    })?;
                }
            }
        }
    }

    wtr.flush()?;
    Ok(())
}
