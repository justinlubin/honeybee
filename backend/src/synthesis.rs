use crate::ir::*;

use crate::derivation;
use crate::egglog_adapter;

#[derive(Debug, Clone)]
pub struct Synthesizer<'a> {
    pub tree: derivation::Tree,
    lib: &'a Library,
}

#[derive(Debug, Clone)]
pub struct ComputationOption {
    pub name: String,
    pub assignment_options: Vec<Assignment>,
}

#[derive(Debug, Clone)]
pub enum GoalOption {
    Output {
        path: Vec<derivation::PathEntry>,
        tag: String,
        computation_options: Vec<ComputationOption>,
    },
    Input {
        path: Vec<derivation::PathEntry>,
        tag: String,
        fact_name: String,
        assignment_options: Vec<Assignment>,
    },
}

// TODO: computation field ignored for annotation goals
#[derive(Debug, Clone)]
pub enum Choice {
    Output {
        path: Vec<String>,
        tag: String,
        computation_name: String,
        assignment: Assignment,
    },
    Input {
        path: Vec<String>,
        tag: String,
        fact_name: String,
        assignment: Assignment,
    },
}

fn restrict(assignments: &Vec<Assignment>, var: &str) -> Vec<Assignment> {
    let var_substr = format!("fv%{}*", var);
    let mut new_assignments = vec![];
    for a in assignments {
        let mut new_a = vec![];
        for (k, v) in a {
            if k.contains(&var_substr) {
                new_a.push((k.clone(), v.clone()));
            }
        }
        new_a.sort();
        new_assignments.push(new_a);
    }
    new_assignments.sort();
    new_assignments.dedup();
    new_assignments
        .into_iter()
        .map(|kvs| kvs.into_iter().collect())
        .collect()
}

impl<'a> Synthesizer<'a> {
    pub fn new(lib: &'a Library, prog: &'a Program) -> Synthesizer<'a> {
        Synthesizer {
            tree: derivation::Tree::from_goal(&prog.goal),
            lib,
        }
    }

    // Precondition: egg must have same lib and prog as synthesizer
    pub fn options(
        &self,
        egg: &mut egglog_adapter::Instance,
    ) -> Vec<GoalOption> {
        let mut ops = vec![];

        for (path, query) in self.tree.queries(self.lib) {
            let basic_assignments = egg.query(&query);

            for (cut_param, goal_fact_name, _mode) in
                &query.computation_signature.params
            {
                ops.push(
                    match self.lib.fact_signature(goal_fact_name).unwrap().kind
                    {
                        FactKind::Input => GoalOption::Input {
                            fact_name: goal_fact_name.clone(),
                            path: path.clone(),
                            tag: cut_param.clone(),
                            assignment_options: restrict(
                                &basic_assignments,
                                &cut_param,
                            ),
                        },
                        FactKind::Output => GoalOption::Output {
                            computation_options: self
                                .lib
                                .matching_computation_signatures(goal_fact_name)
                                .into_iter()
                                .filter_map(|lemma| {
                                    let assignment_options = egg.query(
                                        &query.cut(self.lib, cut_param, lemma),
                                    );
                                    if assignment_options.is_empty() {
                                        None
                                    } else {
                                        Some(ComputationOption {
                                            assignment_options,
                                            name: lemma.name.clone(),
                                        })
                                    }
                                })
                                .collect(),
                            path: path.clone(),
                            tag: cut_param.clone(),
                        },
                    },
                )
            }
        }

        ops
    }

    fn args_and_condition(
        assignment: &Assignment,
        tag: &String,
    ) -> (Vec<(String, Value)>, Predicate) {
        let mut ret_args = vec![];
        let mut additional_condition = vec![];

        for (lhs, rhs) in assignment {
            let components = lhs
                .strip_prefix("fv%")
                .unwrap()
                .split("*")
                .collect::<Vec<&str>>();

            assert!(components.len() == 2);

            let selector = components[0].to_owned();
            let arg = components[1].to_owned();

            if selector == *tag {
                ret_args.push((arg.clone(), rhs.clone()))
            } else {
                additional_condition.push(PredicateRelation::BinOp(
                    PredicateRelationBinOp::Eq,
                    PredicateAtom::Select { selector, arg },
                    PredicateAtom::Const(rhs.clone()),
                ))
            }
        }

        (ret_args, additional_condition)
    }

    pub fn step(&mut self, choice: &Choice) {
        match choice {
            Choice::Input {
                path,
                tag,
                fact_name,
                assignment,
            } => {
                let (args, additional_condition) =
                    Synthesizer::args_and_condition(&assignment, &tag);

                self.tree = self
                    .tree
                    .replace(
                        &path
                            .iter()
                            .cloned()
                            .chain(std::iter::once(tag.clone()))
                            .collect::<Vec<String>>(),
                        &derivation::Tree::Axiom(Fact {
                            name: fact_name.clone(),
                            args,
                        }),
                    )
                    .add_side_condition(&path[..], &additional_condition);
            }
            Choice::Output {
                path,
                tag,
                computation_name,
                assignment,
            } => {
                let cs =
                    self.lib.computation_signature(&computation_name).unwrap();

                let mut ret_args = vec![];
                let mut additional_condition = vec![];

                for (lhs, rhs) in assignment {
                    let components = lhs
                        .strip_prefix("fv%")
                        .unwrap()
                        .split("*")
                        .collect::<Vec<&str>>();

                    assert!(components.len() == 2);

                    let selector = components[0].to_owned();
                    let arg = components[1].to_owned();

                    if selector == *tag {
                        ret_args.push((arg.clone(), rhs.clone()))
                    } else {
                        additional_condition.push(PredicateRelation::BinOp(
                            PredicateRelationBinOp::Eq,
                            PredicateAtom::Select { selector, arg },
                            PredicateAtom::Const(rhs.clone()),
                        ))
                    }
                }

                self.tree = self
                    .tree
                    .replace(
                        &path
                            .iter()
                            .cloned()
                            .chain(std::iter::once(tag.clone()))
                            .collect::<Vec<String>>(),
                        &derivation::Tree::from_computation_signature(
                            cs, ret_args,
                        ),
                    )
                    .add_side_condition(&path[..], &additional_condition);
            }
        }
    }
}
