use crate::derivation;
use crate::ir::*;
use crate::task::*;
use crate::util;

use std::time::Instant;

#[derive(Debug, Clone)]
pub enum Config {
    Basic,
    Prune,
}

pub enum ExpansionResult {
    Complete(derivation::Tree),
    Incomplete(Vec<derivation::Tree>),
    TimedOut,
}

fn support_one(prog: &Program, vt: &ValueType) -> Vec<Value> {
    let mut result = match vt {
        // 0 and 1 for bool false/true
        ValueType::Int => vec![Value::Int(0), Value::Int(1)],
        _ => vec![],
    };
    for f in &prog.annotations {
        for (_, v) in &f.args {
            if v.infer() == *vt && !result.contains(v) {
                result.push(v.clone())
            }
        }
    }
    for (_, v) in &prog.goal.args {
        if v.infer() == *vt && !result.contains(v) {
            result.push(v.clone())
        }
    }
    result
}

pub fn support(
    prog: &Program,
    params: &Vec<(String, ValueType)>,
) -> Vec<Vec<(String, Value)>> {
    let choices = params
        .iter()
        .map(|(p, vt)| {
            support_one(prog, vt)
                .into_iter()
                .map(|v| (p.clone(), v))
                .collect()
        })
        .collect();
    util::timed_cartesian_product(choices, u128::MAX).unwrap()
}

fn should_keep(
    antecedents: &Vec<(String, derivation::Tree)>,
    consequent: &Fact,
    predicate: &Predicate,
    config: Config,
) -> bool {
    match config {
        Config::Basic => true,
        Config::Prune => {
            let args = antecedents
                .iter()
                .map(|(s, t)| {
                    (
                        s.clone(),
                        match t {
                            derivation::Tree::Axiom(f) => f,
                            derivation::Tree::Step { consequent, .. } => {
                                consequent
                            }
                            derivation::Tree::Goal(_) => {
                                panic!("Cannot prune goal antecedent")
                            }
                            derivation::Tree::Collect(_, _) => todo!(),
                        }
                        .clone(),
                    )
                })
                .collect();
            predicate.iter().all(|pr| pr.sat(consequent, &args))
        }
    }
}

pub fn expand(
    lib: &Library,
    prog: &Program,
    tree: derivation::Tree,
    config: Config,
    start: Instant,
    soft_timeout: u128,
) -> ExpansionResult {
    match tree {
        derivation::Tree::Axiom(_) => ExpansionResult::Complete(tree),
        derivation::Tree::Goal(fact_name) => {
            let fact_sig = lib.fact_signature(&fact_name).unwrap();
            match fact_sig.kind {
                FactKind::Input => ExpansionResult::Incomplete(
                    prog.annotations
                        .iter()
                        .filter_map(|f| {
                            if f.name == fact_name {
                                Some(derivation::Tree::Axiom(f.clone()))
                            } else {
                                None
                            }
                        })
                        .collect(),
                ),
                FactKind::Output => {
                    let mut expansions = vec![];
                    for cs in lib.matching_computation_signatures(&fact_name) {
                        let fact_params =
                            &lib.fact_signature(&fact_name).unwrap().params;
                        for args in support(&prog, fact_params) {
                            let consequent = Fact {
                                name: fact_name.clone(),
                                args,
                            };
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
                match expand(
                    lib,
                    prog,
                    subtree,
                    config.clone(),
                    start,
                    soft_timeout,
                ) {
                    ExpansionResult::Complete(t) => {
                        antecedent_choices.push(vec![(tag, t)])
                    }
                    ExpansionResult::Incomplete(ts) => {
                        complete = false;
                        antecedent_choices.push(
                            ts.into_iter().map(|t| (tag.clone(), t)).collect(),
                        )
                    }
                    ExpansionResult::TimedOut => {
                        return ExpansionResult::TimedOut
                    }
                }
            }
            if complete {
                ExpansionResult::Complete(tree)
            } else {
                let elapsed = start.elapsed().as_millis();
                if elapsed > soft_timeout {
                    return ExpansionResult::TimedOut;
                }
                match util::timed_cartesian_product(
                    antecedent_choices,
                    soft_timeout - elapsed,
                ) {
                    Some(prod) => ExpansionResult::Incomplete(
                        prod.into_iter()
                            .filter_map(|new_antecedents| {
                                if should_keep(
                                    &new_antecedents,
                                    consequent,
                                    &side_condition,
                                    config.clone(),
                                ) {
                                    Some(derivation::Tree::Step {
                                        label: label.clone(),
                                        antecedents: new_antecedents,
                                        consequent: consequent.clone(),
                                        side_condition: side_condition.clone(),
                                    })
                                } else {
                                    None
                                }
                            })
                            .collect(),
                    ),
                    None => ExpansionResult::TimedOut,
                }
            }
        }
    }
}

pub fn synthesize(sp: SynthesisProblem, config: Config) -> SynthesisResult {
    let worklist = vec![derivation::Tree::from_goal(&sp.prog.goal)];
    synthesize_worklist(sp, config, worklist)
}

pub fn synthesize_worklist(
    SynthesisProblem {
        lib,
        prog,
        task,
        soft_timeout,
    }: SynthesisProblem,
    config: Config,
    mut worklist: Vec<derivation::Tree>,
) -> SynthesisResult {
    let mut results = vec![];

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

            match expand(lib, prog, t, config.clone(), now, soft_timeout) {
                ExpansionResult::Complete(new_t) => match task {
                    Task::AnyValid => {
                        if new_t.valid(&prog.annotations) {
                            return SynthesisResult {
                                results: vec![new_t],
                                completed: true,
                            };
                        }
                    }
                    Task::AllValid => {
                        if new_t.valid(&prog.annotations) {
                            results.push(new_t)
                        }
                    }
                    Task::AnySimplyTyped => {
                        return SynthesisResult {
                            results: vec![new_t],
                            completed: true,
                        }
                    }
                    Task::AllSimplyTyped => results.push(new_t),
                    Task::Particular(ref choice) => {
                        if new_t.eq_ignoring_conditions(choice) {
                            return SynthesisResult {
                                results: vec![new_t],
                                completed: true,
                            };
                        }
                    }
                },
                ExpansionResult::Incomplete(ts) => new_worklist.extend(ts),
                ExpansionResult::TimedOut => {
                    return SynthesisResult {
                        results,
                        completed: false,
                    };
                }
            }
        }

        worklist = new_worklist;
    }
    SynthesisResult {
        results,
        completed: true,
    }
}
