use crate::derivation;

use crate::ir::*;

pub enum Mode<'a> {
    AnyValid,
    AllValid,
    AnySimplyTyped,
    AllSimplyTyped,
    Particular(&'a derivation::Tree),
}

enum ExpansionResult {
    Complete(derivation::Tree),
    Incomplete(Vec<derivation::Tree>),
}

pub fn cartesian_product<T: Clone>(
    choices_sequence: Vec<Vec<T>>,
) -> Vec<Vec<T>> {
    let mut result = vec![vec![]];
    for choices in choices_sequence {
        let mut new_result = vec![];
        for choice in choices {
            for mut r in result.clone() {
                r.push(choice.clone());
                new_result.push(r);
            }
        }
        result = new_result;
    }
    result
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
    cartesian_product(choices)
}

fn expand(
    lib: &Library,
    prog: &Program,
    tree: derivation::Tree,
) -> ExpansionResult {
    match tree {
        derivation::Tree::Axiom(_) => ExpansionResult::Complete(tree),
        derivation::Tree::Goal(fact_name) => {
            let mut expansions = vec![];
            for cs in lib.matching_computation_signatures(&fact_name) {
                let fact_params =
                    &lib.fact_signature(&fact_name).unwrap().params;
                for args in support(&prog.annotations, fact_params) {
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
                                    derivation::Tree::Goal(goal_name.clone()),
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
                    cartesian_product(antecedent_choices)
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

fn enumerate(
    lib: &Library,
    prog: &Program,
    mode: Mode,
    max_iterations: usize,
) -> Vec<derivation::Tree> {
    let mut results = vec![];
    let mut worklist = vec![derivation::Tree::from_goal(&prog.goal)];
    let mut iterations = 0;

    while !worklist.is_empty() && iterations < max_iterations {
        let mut new_worklist = vec![];

        for t in worklist.into_iter() {
            match expand(lib, prog, t) {
                ExpansionResult::Complete(t) => match mode {
                    Mode::AnyValid => {
                        if t.valid(&prog.annotations) {
                            return vec![t];
                        }
                    }
                    Mode::AllValid => {
                        if t.valid(&prog.annotations) {
                            results.push(t)
                        }
                    }
                    Mode::AnySimplyTyped => return vec![t],
                    Mode::AllSimplyTyped => results.push(t),
                    Mode::Particular(choice) => {
                        if t == *choice {
                            return vec![t];
                        }
                    }
                },
                ExpansionResult::Incomplete(ts) => new_worklist.extend(ts),
            }
        }

        worklist = new_worklist;
        iterations += 1;
    }
    results
}
