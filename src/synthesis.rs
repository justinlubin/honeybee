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
pub struct GoalOption {
    path: Vec<String>,
    tag: String,
    computation_options: Vec<ComputationOption>,
}

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
        self.tree
            .queries(self.lib)
            .into_iter()
            .flat_map(|(path, query)| {
                query
                    .computation_signature
                    .params
                    .iter()
                    .map(|(cut_param, goal_fact_name, _)| GoalOption {
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
                    })
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    pub fn step(&mut self, choice: &Choice) {
        let ops = self.options();

        let goal_path = &ops[choice.goal].path;
        let goal_tag = &ops[choice.goal].tag;

        let computation_name =
            &ops[choice.goal].computation_options[choice.computation].name;

        let assignment = &ops[choice.goal].computation_options
            [choice.computation]
            .assignment_options[choice.assignment];

        let cs = self.lib.computation_signature(&computation_name).unwrap();

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

            if selector == *goal_tag {
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
                &goal_path
                    .iter()
                    .cloned()
                    .chain(std::iter::once(goal_tag.clone()))
                    .collect::<Vec<String>>(),
                &derivation::Tree::make_step(self.lib, cs, ret_args),
            )
            .add_side_condition(&goal_path[..], &additional_condition);
    }

    // fn step(&self) {}
}
