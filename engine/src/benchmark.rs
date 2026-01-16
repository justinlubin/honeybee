//! # Benchmarking
//!
//! This module defines everything necessary to benchmark the synthesizers in
//! this project.
//!
//! To use this code, create a [`Runner`] object using [`Runner::new`],
//! then call [`Runner::suites`] to run a set of benchmark suits.

use crate::util::{EarlyCutoff, Timer};
use crate::{core, menu, parse, top_down, typecheck};

use instant::{Duration, Instant};
use pbn::Step;
use rayon::prelude::*;
use serde::Serialize;
use std::io;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Benchmark configuration.
pub struct Config {
    /// How many times to run each entry
    pub replicates: usize,
    /// When to cut off the benchmark early
    pub timeout: Duration,
    /// Whether or not to run the benchmarks in parallel
    pub parallel: bool,
    /// Run only the benchmarks that contain this string
    pub entry_filter: String,
    /// The algorithms to benchmark
    pub algorithms: Vec<menu::Algorithm>,
    /// Consider at most this many particular solutions (set to usize::MAX for all)
    pub particular_solution_limit: usize,
}

/// The core data structure for running benchmarks.
pub struct Runner {
    config: Config,
    wtr: Arc<Mutex<csv::Writer<Box<dyn io::Write + Send + 'static>>>>,
}

struct Entry {
    // Key
    suite_name: String,
    entry_name: String,
    solution_name: String,
    algorithm: menu::Algorithm,
    replicate: usize,
    // Value
    problem: core::Problem,
    solution: Option<core::Exp>,
}

#[derive(Debug, Clone, Serialize)]
struct EntryResult {
    // Key
    suite_name: String,
    entry_name: String,
    solution_name: String,
    algorithm: menu::Algorithm,
    replicate: usize,
    // Value
    completed: bool,
    success: bool,
    duration: u128,
}

impl Runner {
    /// Create a new benchmark runner (start here!). The `writer` argument is
    /// the location that the benchmark results will get written to
    /// (e.g., stdout).
    pub fn new(
        config: Config,
        writer: impl io::Write + Send + 'static,
    ) -> Self {
        Self {
            wtr: Arc::new(Mutex::new(
                csv::WriterBuilder::new()
                    .delimiter(b'\t')
                    .from_writer(Box::new(writer)),
            )),
            config,
        }
    }

    fn load_entries(&self, suite_paths: &Vec<PathBuf>) -> Vec<Entry> {
        let mut entries = vec![];

        for suite_path in suite_paths {
            let suite_name = suite_path.file_name().unwrap().to_str().unwrap();

            let lib_path = suite_path.join("_suite.hblib.toml");
            let lib_string = std::fs::read_to_string(lib_path).unwrap();
            let library = parse::library(&lib_string).unwrap();

            for prog_path in
                glob::glob(suite_path.join("*.hb.toml").to_str().unwrap())
                    .unwrap()
                    .filter_map(Result::ok)
            {
                let prog_path_noext =
                    prog_path.with_extension("").with_extension("");

                let entry_name = prog_path_noext
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_owned();

                if !entry_name.contains(&self.config.entry_filter) {
                    continue;
                }

                let prog_string = std::fs::read_to_string(prog_path).unwrap();
                let program = parse::program(&prog_string).unwrap();

                let problem = core::Problem {
                    library: library.clone(),
                    program,
                };

                typecheck::problem(&problem).unwrap();

                let mut solutions = vec![("<ANY>".to_owned(), None)];

                for (i, solution_path) in
                    glob::glob(prog_path_noext.join("*.json").to_str().unwrap())
                        .unwrap()
                        .filter_map(Result::ok)
                        .enumerate()
                {
                    if i >= self.config.particular_solution_limit {
                        break;
                    }

                    let solution_name = solution_path
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_owned();
                    let solution_string =
                        std::fs::read_to_string(&solution_path).unwrap();
                    let solution = parse::exp(&solution_string).unwrap();

                    solutions.push((solution_name, Some(solution)));
                }

                for (solution_name, solution) in solutions {
                    for algorithm in &self.config.algorithms {
                        for replicate in 0..self.config.replicates {
                            entries.push(Entry {
                                suite_name: suite_name.to_owned(),
                                entry_name: entry_name.clone(),
                                solution_name: solution_name.to_owned(),
                                algorithm: algorithm.clone(),
                                replicate,
                                problem: problem.clone(),
                                solution: solution.clone(),
                            });
                        }
                    }
                }
            }
        }

        entries
    }

    fn entry_particular(
        &self,
        algorithm: menu::Algorithm,
        problem: core::Problem,
        solution: core::Exp,
    ) -> Result<bool, EarlyCutoff> {
        let timer = Timer::finite(self.config.timeout);
        let mut controller = algorithm.controller(timer, problem, false);

        loop {
            if *controller.working_expression() == solution {
                return Ok(true);
            }

            let options = controller.provide()?;
            if options.is_empty() {
                return Ok(false);
            }

            match options.into_iter().find(|opt| {
                let tentative =
                    opt.apply(controller.working_expression()).unwrap();
                tentative.pattern_match(&solution).is_some()
            }) {
                Some(step) => controller.decide(step),
                None => return Ok(false),
            }
        }
    }

    fn entry_any(
        &self,
        algorithm: menu::Algorithm,
        problem: core::Problem,
    ) -> Result<bool, EarlyCutoff> {
        let timer = Timer::finite(self.config.timeout);
        let start = top_down::Sketch::blank();
        let mut synth = algorithm.any_synthesizer(problem);
        Ok(synth.provide_any(&timer, &start)?.is_some())
    }

    fn entry(&self, e: Entry) {
        let now = Instant::now();

        let synthesis_result = match e.solution {
            Some(sol) => {
                self.entry_particular(e.algorithm.clone(), e.problem, sol)
            }
            None => self.entry_any(e.algorithm.clone(), e.problem),
        };

        let duration = now.elapsed().as_millis();

        let r = EntryResult {
            suite_name: e.suite_name.clone(),
            entry_name: e.entry_name.clone(),
            solution_name: e.solution_name.clone(),
            algorithm: e.algorithm,
            replicate: e.replicate,
            completed: synthesis_result.is_ok(),
            success: synthesis_result == Ok(true),
            duration,
        };

        let wtr = Arc::clone(&self.wtr);
        let mut wtr = wtr.lock().unwrap();
        wtr.serialize(r).unwrap();
        wtr.flush().unwrap();
    }

    /// Run a set of benchmark suites
    pub fn suites(&self, suite_paths: &Vec<PathBuf>) {
        let entries = self.load_entries(suite_paths);

        if self.config.parallel {
            entries.into_par_iter().for_each(|e| self.entry(e));
        } else {
            entries.into_iter().for_each(|e| self.entry(e));
        }
    }
}
