use crate::ir::*;
use crate::task::*;

use crate::analysis;
use crate::derivation;
use crate::egglog_adapter;
use crate::synthesis;

use std::collections::HashMap;
use std::time::Instant;

pub enum Config {
    Basic,
}

pub fn synthesize(
    SynthesisProblem {
        lib,
        prog,
        task,
        soft_timeout,
    }: SynthesisProblem,
    config: Config,
) -> SynthesisResult {
    let now = Instant::now();

    let mut egg = egglog_adapter::Instance::new(
        lib,
        prog,
        match config {
            Config::Basic => false,
        },
    );

    if !egg.check_possible() {
        return SynthesisResult {
            results: vec![],
            completed: true,
        };
    }

    match task {
        Task::AnyValid => {
            let mut synthesizer = synthesis::Synthesizer::new(lib, prog);

            loop {
                if now.elapsed().as_millis() > soft_timeout {
                    return SynthesisResult {
                        results: vec![],
                        completed: false,
                    };
                }
                let options = synthesizer.options(&mut egg);
                if options.is_empty() {
                    return SynthesisResult {
                        results: vec![synthesizer.tree],
                        completed: true,
                    };
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
        Task::AllValid => {
            let mut worklist = vec![synthesis::Synthesizer::new(lib, prog)];
            let mut results = vec![];
            while let Some(synthesizer) = worklist.pop() {
                if now.elapsed().as_millis() > soft_timeout {
                    return SynthesisResult {
                        results,
                        completed: false,
                    };
                }
                let options = synthesizer.options(&mut egg);
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
                                worklist.push(new);
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
                            worklist.push(new);
                        }
                    }
                }
            }
            SynthesisResult {
                results,
                completed: true,
            }
        }
        Task::Particular(tree) => {
            let mut synthesizer = synthesis::Synthesizer::new(lib, prog);

            loop {
                if now.elapsed().as_millis() > soft_timeout {
                    return SynthesisResult {
                        results: vec![],
                        completed: false,
                    };
                }
                let options = synthesizer.options(&mut egg);
                if options.is_empty() {
                    return SynthesisResult {
                        results: vec![synthesizer.tree],
                        completed: true,
                    };
                }
                synthesizer.step(&match options.into_iter().next().unwrap() {
                    synthesis::GoalOption::Output {
                        path,
                        tag,
                        computation_options,
                    } => {
                        let path = derivation::into_tags(path);
                        let (name_choice, assignment_choice) = match tree
                            .get(&path)
                        {
                            derivation::Tree::Step {
                                consequent,
                                antecedents,
                                ..
                            } => {
                                let mut assignment_choice = HashMap::new();
                                for (k, v) in &consequent.args {
                                    assignment_choice.insert(
                                        format!("fv%{}*{}", tag, k),
                                        v.clone(),
                                    );
                                }
                                for (t, a) in antecedents {
                                    let args = match a {
                                        derivation::Tree::Axiom(f) => &f.args,
                                        derivation::Tree::Goal(_) => panic!(
                                            "Incomplete particular reference"
                                        ),
                                        derivation::Tree::Collect(_, _) => {
                                            todo!()
                                        }
                                        derivation::Tree::Step {
                                            consequent,
                                            ..
                                        } => &consequent.args,
                                    };
                                    for (k, v) in args {
                                        assignment_choice.insert(
                                            format!("fv%{}*{}", t, k),
                                            v.clone(),
                                        );
                                    }
                                }
                                let name_choice = antecedents
                                    .iter()
                                    .find_map(|(t, a)| {
                                        if *t == tag {
                                            match a {
                                                derivation::Tree::Step {
                                                    label,
                                                    ..
                                                } => Some(label),
                                                _ => panic!(),
                                            }
                                        } else {
                                            None
                                        }
                                    })
                                    .unwrap();
                                (name_choice, assignment_choice)
                            }
                            _ => panic!("Improper particular reference"),
                        };

                        let synthesis::ComputationOption {
                            name,
                            assignment_options,
                        } = computation_options
                            .into_iter()
                            .find(|c| c.name == *name_choice)
                            .unwrap();

                        let assignment = assignment_options
                            .into_iter()
                            .find(|a| {
                                a.iter().all(|(k, v)| {
                                    assignment_choice.get(k) == Some(v)
                                })
                            })
                            .unwrap();

                        synthesis::Choice::Output {
                            path,
                            tag,
                            computation_name: name,
                            assignment,
                        }
                    }
                    synthesis::GoalOption::Input {
                        path,
                        tag,
                        fact_name,
                        assignment_options,
                    } => {
                        let mut path = derivation::into_tags(path);
                        path.push(tag.clone());
                        let assignment_choice = match tree.get(&path) {
                            derivation::Tree::Axiom(f) => f
                                .args
                                .iter()
                                .map(|(k, v)| {
                                    (format!("fv%{}*{}", tag, k), v.clone())
                                })
                                .collect::<HashMap<_, _>>(),

                            _ => panic!("Improper particular reference"),
                        };
                        path.pop();
                        let assignment = assignment_options
                            .into_iter()
                            .find(|a| {
                                a.iter().all(|(k, v)| {
                                    assignment_choice.get(k) == Some(v)
                                })
                            })
                            .unwrap();
                        synthesis::Choice::Input {
                            tag,
                            fact_name,
                            assignment,
                            path,
                        }
                    }
                });
            }
        }
        Task::AnySimplyTyped => panic!("PBN_Datalog cannot do AnySimplyTyped"),
        Task::AllSimplyTyped => panic!("PBN_Datalog cannot do AllSimplyTyped"),
    }
}

pub fn run(
    lib: &Library,
    prog: &Program,
    interactive: bool,
) -> Option<derivation::Tree> {
    let mut egg = egglog_adapter::Instance::new(lib, prog, false);

    if !egg.check_possible() {
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
        let options = synthesizer.options(&mut egg);
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
