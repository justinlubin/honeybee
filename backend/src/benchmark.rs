use crate::ir::*;

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

use crate::benchmark_data::*;

#[derive(Debug, Clone, Serialize)]
pub struct Record {
    pub suite: String,
    pub entry: String,
    pub task: Task,
    pub subentry: String,
    pub replicate: usize,
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
    sp: task::SynthesisProblem,
    algorithm: Algorithm,
) -> Timed<task::SynthesisResult> {
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
    // for t in &sr.results {
    //     // To be fair to LLMs, include Python conversion time
    //     let _ = backend::Python::new(t).emit().plain_text("");
    // }
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
    subentry: &str,
    show_results: bool,
    wtr: &Arc<Mutex<Option<csv::Writer<std::io::Stdout>>>>,
) -> Vec<Record> {
    let sp = task::SynthesisProblem {
        lib: &lib,
        prog: &prog,
        task: synthesis_task,
        soft_timeout,
    };

    let mut records = vec![];

    for replicate in 0..run_count {
        let Timed {
            val: task::SynthesisResult { results, completed },
            duration,
        } = run_one(sp.clone(), algorithm.clone());

        let r = Record {
            suite: suite.to_owned(),
            entry: entry.to_owned(),
            task: task.clone(),
            subentry: subentry.to_owned(),
            replicate,
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
            if show_results {
                println!(">>> Start Results >>>\n");
                for r in results {
                    println!("{}", r);
                }
                println!("<<< End Results <<<");
            }
        });
    }

    records
}

fn results(
    lib: &Library,
    prog: &Program,
    particulars: &Vec<(String, derivation::Tree)>,
    soft_timeout: u128,
    run_count: usize,
    suite: &str,
    entry: &str,
    algorithm: Algorithm,
    task: Task,
    parallel: bool,
    show_results: bool,
    wtr: &Arc<Mutex<Option<csv::Writer<std::io::Stdout>>>>,
) -> Vec<Record> {
    let synthesis_tasks = match task {
        Task::Any => vec![("NA".to_owned(), task::Task::AnyValid)],
        Task::All => vec![("NA".to_owned(), task::Task::AllValid)],
        Task::Particular => particulars
            .iter()
            .map(|(subentry, dt)| {
                (subentry.clone(), task::Task::Particular(dt.clone()))
            })
            .collect(),
    };

    if parallel {
        synthesis_tasks
            .par_iter()
            .flat_map(|(subentry, synthesis_task)| {
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
                    subentry,
                    show_results,
                    wtr,
                )
            })
            .collect()
    } else {
        synthesis_tasks
            .iter()
            .flat_map(|(subentry, synthesis_task)| {
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
                    subentry,
                    show_results,
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
    particulars: &Vec<(String, derivation::Tree)>,
    soft_timeout: u128,
    run_count: usize,
    suite: &str,
    parallel: bool,
    show_results: bool,
    algotasks: &Vec<(Algorithm, Task)>,
    wtr: &Arc<Mutex<Option<csv::Writer<std::io::Stdout>>>>,
) -> Vec<Record> {
    // let particulars = {
    //     let sr = pbn::synthesize(
    //         task::SynthesisProblem {
    //             lib: &lib,
    //             prog: &prog,
    //             task: task::Task::AllValid,
    //             soft_timeout,
    //         },
    //         pbn::Config::Memo,
    //     );

    //     if sr.completed {
    //         Some(sr.results)
    //     } else {
    //         None
    //     }
    // };

    if parallel {
        algotasks
            .par_iter()
            .flat_map(|(algorithm, task)| {
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
                    show_results,
                    wtr,
                )
            })
            .collect()
    } else {
        algotasks
            .iter()
            .flat_map(|(algorithm, task)| {
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
                    show_results,
                    wtr,
                )
            })
            .collect()
    }
}

pub fn suite_results(
    suite_directory: &PathBuf,
    run_count: usize,
    soft_timeout: u128, // in milliseconds
    filter: &str,
    parallel: bool,
    show_results: bool,
    algotasks: &Vec<(Algorithm, Task)>,
    wtr: &Arc<Mutex<Option<csv::Writer<std::io::Stdout>>>>,
) -> Result<Vec<Record>, Box<dyn std::error::Error>> {
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

        let mut particulars = vec![];
        for particular_filename in glob::glob(
            prog_filename_without_extension
                .join("*.json")
                .to_str()
                .unwrap(),
        )
        .unwrap()
        .filter_map(Result::ok)
        {
            let particular_json =
                std::fs::read_to_string(&particular_filename).unwrap();
            let particular = serde_json::from_str(&particular_json).expect(
                &format!("unparseable particular: {:?}", particular_filename),
            );
            particulars.push((
                particular_filename
                    .with_extension("")
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_owned(),
                particular,
            ));
        }

        progs.push((prog, entry, particulars));
    }

    Ok(if parallel {
        progs
            .par_iter()
            .flat_map(|(prog, entry, particulars)| {
                entry_results(
                    &lib,
                    prog,
                    entry,
                    particulars,
                    soft_timeout,
                    run_count,
                    suite,
                    parallel,
                    show_results,
                    algotasks,
                    &wtr,
                )
            })
            .collect()
    } else {
        progs
            .iter()
            .flat_map(|(prog, entry, particulars)| {
                entry_results(
                    &lib,
                    prog,
                    entry,
                    particulars,
                    soft_timeout,
                    run_count,
                    suite,
                    parallel,
                    show_results,
                    algotasks,
                    &wtr,
                )
            })
            .collect()
    })
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
    suite_directories: &Vec<PathBuf>,
    run_count: usize,
    soft_timeout: u128, // in milliseconds
    filter: &str,
    write_stdout: bool,
    parallel: bool,
    show_results: bool,
    algotasks: &Vec<(Algorithm, Task)>,
) -> Result<Vec<Record>, Box<dyn std::error::Error>> {
    for suite_directory in suite_directories {
        assert!(suite_directory.is_dir());
    }

    let wtr = Arc::new(Mutex::new(if write_stdout {
        Some(
            csv::WriterBuilder::new()
                .delimiter(b'\t')
                .from_writer(std::io::stdout()),
        )
    } else {
        None
    }));

    let mut results = vec![];

    for suite_directory in suite_directories {
        results.extend(suite_results(
            &suite_directory,
            run_count,
            soft_timeout,
            filter,
            parallel,
            show_results,
            algotasks,
            &wtr,
        )?);
    }

    Ok(results)
}
