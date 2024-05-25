use crate::derivation;
use crate::egglog_adapter;

use crate::ir::*;

#[derive(Debug, Clone)]
pub struct Synthesizer<'a> {
    pub tree: derivation::Tree,
    lib: &'a Library,
    prog: &'a Program,
}

#[derive(Debug, Clone)]
pub struct ComputationOption {
    pub name: String,
    pub assignment_options: Vec<Assignment>,
}

#[derive(Debug, Clone)]
pub enum GoalOption {
    Analysis {
        path: Vec<String>,
        tag: String,
        computation_options: Vec<ComputationOption>,
    },
    Annotation {
        path: Vec<String>,
        tag: String,
        fact_name: String,
        assignment_options: Vec<Assignment>,
    },
}

// TODO: computation field ignored for annotation goals
#[derive(Debug, Clone)]
pub enum Choice {
    Analysis {
        path: Vec<String>,
        tag: String,
        computation_name: String,
        assignment: Assignment,
    },
    Annotation {
        path: Vec<String>,
        tag: String,
        fact_name: String,
        assignment: Assignment,
    },
}

impl<'a> Synthesizer<'a> {
    pub fn new(lib: &'a Library, prog: &'a Program) -> Synthesizer<'a> {
        Synthesizer {
            tree: derivation::Tree::from_goal(&prog.goal),
            lib,
            prog,
        }
    }

    pub fn options(&self) -> Vec<GoalOption> {
        let mut ops = vec![];

        for (path, query) in self.tree.queries(self.lib) {
            let basic_assignments =
                egglog_adapter::query(self.lib, &self.prog.annotations, &query);

            for (cut_param, goal_fact_name, _mode) in
                &query.computation_signature.params
            {
                ops.push(
                    match self.lib.fact_signature(goal_fact_name).unwrap().kind
                    {
                        FactKind::Annotation => GoalOption::Annotation {
                            fact_name: goal_fact_name.clone(),
                            path: path.clone(),
                            tag: cut_param.clone(),
                            assignment_options: basic_assignments.clone(),
                        },
                        FactKind::Analysis => GoalOption::Analysis {
                            computation_options: self
                                .lib
                                .matching_computation_signatures(goal_fact_name)
                                .into_iter()
                                .map(|lemma| ComputationOption {
                                    assignment_options: egglog_adapter::query(
                                        self.lib,
                                        &self.prog.annotations,
                                        &query.cut(self.lib, cut_param, lemma),
                                    ),
                                    name: lemma.name.clone(),
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
            Choice::Annotation {
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
            Choice::Analysis {
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
