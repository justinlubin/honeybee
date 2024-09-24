use crate::analysis;
use crate::derivation;
use crate::egglog_adapter;
use crate::ir::*;
use crate::synthesis;

use std::time::Instant;

pub enum Mode<'a> {
    Any,
    All,
    Particular(&'a derivation::Tree),
}

// returns (results, completed)
pub fn enumerate(
    lib: &Library,
    prog: &Program,
    mode: Mode,
    soft_timeout: u128, // milliseconds
) -> (Vec<derivation::Tree>, bool) {
    let now = Instant::now();

    if !egglog_adapter::check_possible(lib, prog) {
        return (vec![], true);
    }

    match mode {
        Mode::Any => {
            let mut synthesizer = synthesis::Synthesizer::new(lib, prog);

            loop {
                if now.elapsed().as_millis() > soft_timeout {
                    return (vec![], false);
                }
                let options = synthesizer.options();
                if options.is_empty() {
                    return (vec![synthesizer.tree], true);
                }
                synthesizer.step(&match options.into_iter().next().unwrap() {
                    synthesis::GoalOption::Output {
                        path,
                        tag,
                        computation_options,
                    } => {
                        let synthesis::ComputationOption {
                            name,
                            assignment_options,
                        } = computation_options.into_iter().next().unwrap();
                        synthesis::Choice::Output {
                            path: derivation::into_tags(path),
                            tag,
                            computation_name: name,
                            assignment: assignment_options
                                .into_iter()
                                .next()
                                .unwrap(),
                        }
                    }
                    synthesis::GoalOption::Input {
                        path,
                        tag,
                        fact_name,
                        assignment_options,
                    } => synthesis::Choice::Input {
                        tag,
                        fact_name,
                        assignment: assignment_options
                            .into_iter()
                            .next()
                            .unwrap(),
                        path: derivation::into_tags(path),
                    },
                });
            }
        }
        Mode::All => {
            let mut worklist = vec![synthesis::Synthesizer::new(lib, prog)];
            let mut results = vec![];
            while !worklist.is_empty() {
                let mut new_worklist = vec![];
                for synthesizer in worklist.into_iter() {
                    if now.elapsed().as_millis() > soft_timeout {
                        return (results, false);
                    }
                    let options = synthesizer.options();
                    if options.is_empty() {
                        results.push(synthesizer.tree);
                        continue;
                    }
                    match options.into_iter().next().unwrap() {
                        synthesis::GoalOption::Output {
                            path,
                            tag,
                            computation_options,
                        } => {
                            let path = derivation::into_tags(path);
                            for computation_option in computation_options {
                                let synthesis::ComputationOption {
                                    name,
                                    assignment_options,
                                } = computation_option;
                                for assignment in assignment_options {
                                    let mut new = synthesizer.clone();
                                    new.step(&synthesis::Choice::Output {
                                        path: path.clone(),
                                        tag: tag.clone(),
                                        computation_name: name.clone(),
                                        assignment,
                                    });
                                    new_worklist.push(new);
                                }
                            }
                        }
                        synthesis::GoalOption::Input {
                            path,
                            tag,
                            fact_name,
                            assignment_options,
                        } => {
                            let path = derivation::into_tags(path);
                            for assignment in assignment_options {
                                let mut new = synthesizer.clone();
                                new.step(&synthesis::Choice::Input {
                                    tag: tag.clone(),
                                    fact_name: fact_name.clone(),
                                    assignment,
                                    path: path.clone(),
                                });
                                new_worklist.push(new);
                            }
                        }
                    }
                }
                worklist = new_worklist;
            }
            (results, true)
        }
        Mode::Particular(tree) => {
            let mut synthesizer = synthesis::Synthesizer::new(lib, prog);

            loop {
                if now.elapsed().as_millis() > soft_timeout {
                    return (vec![], false);
                }
                return (vec![], false);
            }
        }
    }
}

pub fn run(
    lib: &Library,
    prog: &Program,
    interactive: bool,
) -> Option<derivation::Tree> {
    if !egglog_adapter::check_possible(lib, prog) {
        return None;
    }

    let mut synthesizer = synthesis::Synthesizer::new(lib, prog);
    let analyzer = if interactive {
        analysis::CLI {
            // mode: analysis::CLIMode::FastForward,
            mode: analysis::CLIMode::Manual,
            print: true,
        }
    } else {
        analysis::CLI {
            mode: analysis::CLIMode::Auto,
            print: false,
        }
    };

    let mut step = 1;
    loop {
        if interactive {
            println!(
                "{} {} {}\n\n{}",
                ansi_term::Color::Fixed(8).paint("═".repeat(2)),
                ansi_term::Color::Fixed(8).paint(format!("Step {}", step)),
                ansi_term::Color::Fixed(8).paint("═".repeat(40)),
                ansi_term::Style::new().bold().paint("Derivation tree:")
            );
            print!("{}", synthesizer.tree.pretty());
        }
        let options = synthesizer.options();
        if options.is_empty() {
            break;
        }
        if interactive {
            println!();
        }
        let choice = analyzer.analyze(options);
        synthesizer.step(&choice);
        step += 1;
    }
    Some(synthesizer.tree)
}
