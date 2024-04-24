use crate::derivation;
use crate::egglog_adapter;

use crate::ir::*;

#[derive(Debug, Clone)]
pub struct Synthesizer {
    pub tree: derivation::Tree,
}

// pub enum Msg {
//     Success,
//     InvalidBreadcrumbs,
// }

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

impl Synthesizer {
    pub fn new(top_level_goal: &Fact) -> Synthesizer {
        Synthesizer {
            tree: derivation::Tree::new(top_level_goal),
        }
    }

    pub fn options(&self, lib: &Library, prog: &Program) -> Vec<GoalOption> {
        self.tree
            .queries(lib)
            .into_iter()
            .flat_map(|(path, query)| {
                query
                    .computation_signature
                    .params
                    .iter()
                    .map(|(cut_param, goal_fact_name)| GoalOption {
                        computation_options: lib
                            .matching_computation_signatures(goal_fact_name)
                            .into_iter()
                            .map(|lemma| ComputationOption {
                                assignment_options: egglog_adapter::query(
                                    lib,
                                    &prog.annotations,
                                    &query.cut(lib, cut_param, lemma),
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

    // fn step(&self) {}
}
