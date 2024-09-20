use crate::derivation;
use crate::enumerate;
use crate::ir::*;
use crate::syntax;

use chumsky::Parser;
use serde::Serialize;
use std::path::PathBuf;
use std::time::Instant;

#[derive(Debug, Serialize)]
enum Algorithm {
    BaselineEnumerative,
    ClassicalConstructiveDatalog,
}

#[derive(Debug, Serialize)]
enum Task {
    Any,
    All,
    // Particular,
}

#[derive(Debug, Serialize)]
struct Record<'a> {
    suite: &'a str,
    entry: &'a str,
    task: Task,
    algorithm: Algorithm,
    duration: u128,
    solution_count: usize,
    output: &'a str,
}

struct SynthesisResult {
    duration: u128,
    solutions: Vec<derivation::Tree>,
}

fn run_one(
    lib: &Library,
    prog: &Program,
    task: Task,
    algorithm: Algorithm,
) -> SynthesisResult {
    match (task, algorithm) {
        (Task::Any, Algorithm::BaselineEnumerative) => {
            let now = Instant::now();

            let solutions = enumerate::enumerate(
                lib,
                prog,
                enumerate::Mode::AnyValid,
                usize::MAX,
            );

            let duration = now.elapsed().as_millis();

            SynthesisResult {
                duration,
                solutions,
            }
        }
        _ => todo!(),
    }
}

const RUNS: usize = 1;

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
        // let particular_filename = prog_filename.with_extension(".txt");

        let entry = prog_filename.file_name().unwrap().to_str().unwrap();

        let prog_src = std::fs::read_to_string(&prog_filename).unwrap();
        let prog = syntax::parse::program()
            .parse(prog_src)
            .map_err(|_| "Program parse error")?;

        for _ in 0..RUNS {
            let sr =
                run_one(&lib, &prog, Task::Any, Algorithm::BaselineEnumerative);

            wtr.serialize(Record {
                suite,
                entry,
                algorithm: Algorithm::BaselineEnumerative,
                task: Task::Any,
                duration: sr.duration,
                solution_count: sr.solutions.len(),
                output: "TODO",
            })?;
        }
    }

    wtr.flush()?;
    Ok(())
}
