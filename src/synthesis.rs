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
    name: String,
    assignment_options: Vec<Assignment>,
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
        name: String,
        assignment_options: Vec<Assignment>,
    },
}

// TODO: computation field ignored for annotation goals
#[derive(Debug, Clone)]
pub struct Choice {
    pub goal: usize,
    pub computation: usize,
    pub assignment: usize,
}

impl<'a> Synthesizer<'a> {
    pub fn new(lib: &'a Library, prog: &'a Program) -> Synthesizer<'a> {
        Synthesizer {
            tree: derivation::Tree::new(&prog.goal),
            lib,
            prog,
        }
    }

    // TODO: cache this if slow?
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
                            name: goal_fact_name.clone(),
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
        let ops = self.options();
        let op = &ops[choice.goal];

        match op {
            GoalOption::Annotation {
                path,
                tag,
                name,
                assignment_options,
            } => {
                let assignment = &assignment_options[choice.assignment];
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
                            name: name.clone(),
                            args,
                        }),
                    )
                    .add_side_condition(&path[..], &additional_condition);
            }
            GoalOption::Analysis {
                path,
                tag,
                computation_options,
            } => {
                let computation_name =
                    &computation_options[choice.computation].name;

                let assignment = &computation_options[choice.computation]
                    .assignment_options[choice.assignment];

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
                        &derivation::Tree::make_step(self.lib, cs, ret_args),
                    )
                    .add_side_condition(&path[..], &additional_condition);
            }
        }
    }

    // fn step(&self) {}
}
