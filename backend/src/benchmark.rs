use crate::task::*;

use crate::backend;
use crate::derivation;
use crate::enumerate;
use crate::pbn;
use crate::syntax;

use chumsky::Parser;
use serde::Serialize;
use std::path::PathBuf;
use std::time::Instant;

#[derive(Debug, Clone, Serialize)]
#[allow(non_camel_case_types)]
enum Algorithm {
    // Baselines/alternatives
    ALT_Enum,
    ALT_EnumPrune,
    // True PBN
    PBN_Datalog,
    // Ablations
    // PBN_DatalogMemo,
    // PBN_Enum,
    // PBN_EnumPrune,
}

#[derive(Debug, Clone, Serialize)]
struct Record<'a> {
    suite: &'a str,
    entry: &'a str,
    task: &'a str,
    algorithm: Algorithm,
    completed: bool,
    duration: u128,
    solution_count: usize,
    solution_size: usize,
}

struct Timed<T> {
    val: T,
    duration: u128,
}

fn run_one(
    sp: SynthesisProblem,
    algorithm: Algorithm,
) -> Timed<SynthesisResult> {
    let now = Instant::now();
    let sr = match algorithm {
        Algorithm::ALT_Enum => {
            enumerate::synthesize(sp, enumerate::Config::Basic)
        }
        Algorithm::ALT_EnumPrune => {
            enumerate::synthesize(sp, enumerate::Config::Prune)
        }
        Algorithm::PBN_Datalog => pbn::synthesize(sp, pbn::Config::Basic),
    };
    for t in &sr.results {
        // To be fair to LLMs, include Python conversion time
        let _ = backend::Python::new(t).emit().plain_text("");
    }
    let duration = now.elapsed().as_millis();
    Timed { val: sr, duration }
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

        for algorithm in vec![
            Algorithm::PBN_Datalog,
            Algorithm::ALT_Enum,
            Algorithm::ALT_EnumPrune,
        ] {
            for task in vec![
                Task::AnyValid,
                Task::AllValid,
                Task::Particular(&particular),
            ] {
                let task_str = task.to_string();

                let sp = SynthesisProblem {
                    lib: &lib,
                    prog: &prog,
                    task,
                    soft_timeout,
                };

                for _ in 0..run_count {
                    let Timed {
                        val: SynthesisResult { results, completed },
                        duration,
                    } = run_one(sp.clone(), algorithm.clone());

                    wtr.serialize(Record {
                        suite,
                        entry,
                        task: &task_str,
                        algorithm: algorithm.clone(),
                        completed,
                        duration,
                        solution_count: results.len(),
                        solution_size: results.iter().map(|t| t.size()).sum(),
                    })?;
                }
                wtr.flush()?;
            }
        }
    }

    Ok(())
}
