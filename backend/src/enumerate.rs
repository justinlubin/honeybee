use crate::derivation;
use crate::ir::*;
use crate::task::*;
use crate::util;

use std::time::Instant;

pub enum Config {
    Basic,
    Prune,
    PruneSMT,
}

enum ExpansionResult {
    Complete(derivation::Tree),
    Incomplete(Vec<derivation::Tree>),
}

fn support_one(annotations: &Vec<Fact>, vt: &ValueType) -> Vec<Value> {
    let mut result = vec![];
    for f in annotations {
        for (_, v) in &f.args {
            if v.infer() == *vt && !result.contains(v) {
                result.push(v.clone())
            }
        }
    }
    result
}

fn support(
    annotations: &Vec<Fact>,
    params: &Vec<(String, ValueType)>,
) -> Vec<Vec<(String, Value)>> {
    let choices = params
        .iter()
        .map(|(p, vt)| {
            support_one(annotations, vt)
                .into_iter()
                .map(|v| (p.clone(), v))
                .collect()
        })
        .collect();
    util::cartesian_product(choices)
}

fn expand(
    lib: &Library,
    prog: &Program,
    tree: derivation::Tree,
) -> ExpansionResult {
    match tree {
        derivation::Tree::Axiom(_) => ExpansionResult::Complete(tree),
        derivation::Tree::Goal(fact_name) => {
            let fact_sig = lib.fact_signature(&fact_name).unwrap();
            // TODO: Check this?
            match fact_sig.kind {
                FactKind::Input => ExpansionResult::Incomplete(
                    prog.annotations
                        .iter()
                        .map(|f| derivation::Tree::Axiom(f.clone()))
                        .collect(),
                ),
                FactKind::Output => {
                    let mut expansions = vec![];
                    for cs in lib.matching_computation_signatures(&fact_name) {
                        let fact_params =
                            &lib.fact_signature(&fact_name).unwrap().params;
                        for args in support(&prog.annotations, fact_params) {
                            let consequent = Fact {
                                name: fact_name.clone(),
                                args,
                            };
                            // TODO safely prune with SMT and brute force contradiction checker
                            // Possibly not here
                            expansions.push(derivation::Tree::Step {
                                label: cs.name.clone(),
                                antecedents: cs
                                    .params
                                    .iter()
                                    .map(|(tag, goal_name, _)| {
                                        (
                                            tag.clone(),
                                            derivation::Tree::Goal(
                                                goal_name.clone(),
                                            ),
                                        )
                                    })
                                    .collect(),
                                consequent,
                                side_condition: cs.precondition.clone(),
                            });
                        }
                    }
                    ExpansionResult::Incomplete(expansions)
                }
            }
        }
        derivation::Tree::Collect(_, _) => todo!(),
        derivation::Tree::Step {
            ref label,
            ref antecedents,
            ref consequent,
            ref side_condition,
        } => {
            let mut complete = true;
            let mut antecedent_choices = vec![];
            for (tag, subtree) in antecedents.clone() {
                match expand(lib, prog, subtree) {
                    ExpansionResult::Complete(t) => {
                        antecedent_choices.push(vec![(tag, t)])
                    }
                    ExpansionResult::Incomplete(ts) => {
                        complete = false;
                        antecedent_choices.push(
                            ts.into_iter().map(|t| (tag.clone(), t)).collect(),
                        )
                    }
                }
            }
            if complete {
                ExpansionResult::Complete(tree)
            } else {
                ExpansionResult::Incomplete(
                    util::cartesian_product(antecedent_choices)
                        .into_iter()
                        .map(|new_antecedents| derivation::Tree::Step {
                            label: label.clone(),
                            antecedents: new_antecedents,
                            consequent: consequent.clone(),
                            side_condition: side_condition.clone(),
                        })
                        .collect(),
                )
            }
        }
    }
}

// returns (results, completed)
pub fn synthesize(
    SynthesisProblem {
        lib,
        prog,
        task,
        soft_timeout,
    }: SynthesisProblem,
    _config: Config,
) -> SynthesisResult {
    let mut results = vec![];
    let mut worklist = vec![derivation::Tree::from_goal(&prog.goal)];

    let now = Instant::now();

    while !worklist.is_empty() {
        let mut new_worklist = vec![];

        for t in worklist.into_iter() {
            if now.elapsed().as_millis() > soft_timeout {
                return SynthesisResult {
                    results,
                    completed: false,
                };
            }
            match expand(lib, prog, t) {
                ExpansionResult::Complete(t) => match task {
                    Task::AnyValid => {
                        if t.valid(&prog.annotations) {
                            return SynthesisResult {
                                results: vec![t],
                                completed: true,
                            };
                        }
                    }
                    Task::AllValid => {
                        if t.valid(&prog.annotations) {
                            results.push(t)
                        }
                    }
                    Task::AnySimplyTyped => {
                        return SynthesisResult {
                            results: vec![t],
                            completed: true,
                        }
                    }
                    Task::AllSimplyTyped => results.push(t),
                    Task::Particular(choice) => {
                        if t == *choice {
                            return SynthesisResult {
                                results: vec![t],
                                completed: true,
                            };
                        }
                    }
                },
                ExpansionResult::Incomplete(ts) => new_worklist.extend(ts),
            }
        }

        worklist = new_worklist;
    }
    SynthesisResult {
        results,
        completed: true,
    }
}
