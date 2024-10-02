use crate::ir::*;
use crate::task::*;

use crate::backend;
use crate::derivation;
use crate::enumerate;
use crate::pbn;
use crate::syntax;
use crate::task;

use rayon::prelude::*;

use chumsky::Parser;
use serde::Serialize;

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Instant;

#[derive(Debug, Clone, Serialize)]
#[allow(non_camel_case_types)]
pub enum Algorithm {
    E,
    EP,
    PBN_E,
    PBN_EP,
    PBN_DL,
    PBN_DLmem,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum Task {
    Any,
    All,
    Particular,
}

#[derive(Debug, Clone, Serialize)]
pub struct Record {
    pub suite: String,
    pub entry: String,
    pub task: Task,
    pub algorithm: Algorithm,
    pub completed: bool,
    pub duration: u128,
    pub solution_count: usize,
    pub solution_size: usize,
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
        Algorithm::E => enumerate::synthesize(sp, enumerate::Config::Basic),
        Algorithm::EP => enumerate::synthesize(sp, enumerate::Config::Prune),
        Algorithm::PBN_DL => pbn::synthesize(sp, pbn::Config::Basic),
        Algorithm::PBN_DLmem => pbn::synthesize(sp, pbn::Config::Memo),
        Algorithm::PBN_E => {
            pbn::synthesize(sp, pbn::Config::Enum(enumerate::Config::Basic))
        }
        Algorithm::PBN_EP => {
            pbn::synthesize(sp, pbn::Config::Enum(enumerate::Config::Prune))
        }
    };
    for t in &sr.results {
        // To be fair to LLMs, include Python conversion time
        let _ = backend::Python::new(t).emit().plain_text("");
    }
    let duration = now.elapsed().as_millis();
    Timed { val: sr, duration }
}

fn task_results(
    lib: &Library,
    prog: &Program,
    soft_timeout: u128,
    run_count: usize,
    suite: &str,
    entry: &str,
    algorithm: Algorithm,
    task: Task,
    synthesis_task: task::Task,
    wtr: &Arc<Mutex<Option<csv::Writer<std::io::Stdout>>>>,
) -> Vec<Record> {
    let sp = SynthesisProblem {
        lib: &lib,
        prog: &prog,
        task: synthesis_task,
        soft_timeout,
    };

    let mut records = vec![];

    for _ in 0..run_count {
        let Timed {
            val: SynthesisResult { results, completed },
            duration,
        } = run_one(sp.clone(), algorithm.clone());

        let r = Record {
            suite: suite.to_owned(),
            entry: entry.to_owned(),
            task: task.clone(),
            algorithm: algorithm.clone(),
            completed,
            duration,
            solution_count: results.len(),
            solution_size: results.iter().map(|t| t.size()).sum(),
        };

        records.push(r.clone());

        Arc::clone(wtr).lock().unwrap().as_mut().map(|wtr| {
            wtr.serialize(r).unwrap();
            wtr.flush().unwrap();
        });
    }

    records
}

fn results(
    lib: &Library,
    prog: &Program,
    particulars: &Option<Vec<derivation::Tree>>,
    soft_timeout: u128,
    run_count: usize,
    suite: &str,
    entry: &str,
    algorithm: Algorithm,
    task: Task,
    parallel: bool,
    wtr: &Arc<Mutex<Option<csv::Writer<std::io::Stdout>>>>,
) -> Vec<Record> {
    let synthesis_tasks = match task {
        Task::Any => vec![task::Task::AnyValid],
        Task::All => vec![task::Task::AllValid],
        Task::Particular => match particulars {
            Some(ps) => ps
                .iter()
                .map(|dt| task::Task::Particular(dt.clone()))
                .collect(),
            None => return vec![],
        },
    };

    if parallel {
        synthesis_tasks
            .par_iter()
            .flat_map(|synthesis_task| {
                task_results(
                    &lib,
                    &prog,
                    soft_timeout,
                    run_count,
                    &suite,
                    &entry,
                    algorithm.clone(),
                    task.clone(),
                    synthesis_task.clone(),
                    wtr,
                )
            })
            .collect()
    } else {
        synthesis_tasks
            .iter()
            .flat_map(|synthesis_task| {
                task_results(
                    &lib,
                    &prog,
                    soft_timeout,
                    run_count,
                    &suite,
                    &entry,
                    algorithm.clone(),
                    task.clone(),
                    synthesis_task.clone(),
                    wtr,
                )
            })
            .collect()
    }
}

fn entry_results(
    lib: &Library,
    prog: &Program,
    entry: &str,
    soft_timeout: u128,
    run_count: usize,
    suite: &str,
    parallel: bool,
    wtr: &Arc<Mutex<Option<csv::Writer<std::io::Stdout>>>>,
) -> Vec<Record> {
    let particulars = {
        let sr = pbn::synthesize(
            SynthesisProblem {
                lib: &lib,
                prog: &prog,
                task: task::Task::AllValid,
                soft_timeout,
            },
            pbn::Config::Memo,
        );

        if sr.completed {
            Some(sr.results)
        } else {
            None
        }
    };

    let algorithms = vec![
        Algorithm::E,
        Algorithm::EP,
        Algorithm::PBN_E,
        Algorithm::PBN_EP,
        Algorithm::PBN_DL,
        Algorithm::PBN_DLmem,
    ];

    let tasks = vec![Task::Particular, Task::Any, Task::All];

    if parallel {
        algorithms
            .par_iter()
            .flat_map(|algorithm| {
                tasks.par_iter().flat_map(|task| {
                    results(
                        &lib,
                        &prog,
                        &particulars,
                        soft_timeout,
                        run_count,
                        &suite,
                        &entry,
                        algorithm.clone(),
                        task.clone(),
                        parallel,
                        wtr,
                    )
                })
            })
            .collect()
    } else {
        algorithms
            .iter()
            .flat_map(|algorithm| {
                tasks.iter().flat_map(|task| {
                    results(
                        &lib,
                        &prog,
                        &particulars,
                        soft_timeout,
                        run_count,
                        &suite,
                        &entry,
                        algorithm.clone(),
                        task.clone(),
                        parallel,
                        wtr,
                    )
                })
            })
            .collect()
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
    filter: &str,
    write_stdout: bool,
    parallel: bool,
) -> Result<Vec<Record>, Box<dyn std::error::Error>> {
    assert!(suite_directory.is_dir());

    let suite = suite_directory.file_name().unwrap().to_str().unwrap();
    let lib_filename = suite_directory.join("_suite.hblib");
    let imp_filename = suite_directory.join("_suite.py");

    let lib_src = std::fs::read_to_string(lib_filename).unwrap();
    let _imp_src = std::fs::read_to_string(imp_filename).unwrap();

    let lib = syntax::parse::library()
        .parse(lib_src)
        .map_err(|_| "Library parse error")?;
    lib.check()
        .map_err(|e| format!("[Library type error] {}", e))?;

    let wtr = Arc::new(Mutex::new(if write_stdout {
        Some(csv::Writer::from_writer(std::io::stdout()))
    } else {
        None
    }));

    let mut progs = vec![];

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
            .unwrap()
            .to_owned();

        if !entry.contains(filter) {
            continue;
        }

        let prog_src = std::fs::read_to_string(&prog_filename).unwrap();
        let prog = syntax::parse::program()
            .parse(prog_src)
            .map_err(|_| "Program parse error")?;
        prog.check(&lib)
            .map_err(|e| format!("[Program type error] {}", e))?;

        progs.push((prog, entry));
    }

    Ok(if parallel {
        progs
            .par_iter()
            .flat_map(|(prog, entry)| {
                entry_results(
                    &lib,
                    prog,
                    entry,
                    soft_timeout,
                    run_count,
                    suite,
                    parallel,
                    &wtr,
                )
            })
            .collect()
    } else {
        progs
            .iter()
            .flat_map(|(prog, entry)| {
                entry_results(
                    &lib,
                    prog,
                    entry,
                    soft_timeout,
                    run_count,
                    suite,
                    parallel,
                    &wtr,
                )
            })
            .collect()
    })
}
