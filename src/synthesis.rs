use crate::derivation;
use crate::egglog_adapter;

use crate::ir::*;

pub struct Synthesizer {
    tree: derivation::Tree,
}

pub enum Msg {
    Success,
    InvalidBreadcrumbs,
}

pub struct ComputationOption {
    signature: ComputationSignature,
    assignment_options: Vec<Assignment>,
}

pub struct GoalOption {
    path: Vec<String>,
    goal: BasicQuery,
    computation_options: Vec<ComputationOption>,
}

impl Synthesizer {
    fn new(top_level_goal: Fact) -> Synthesizer {
        Synthesizer {
            tree: derivation::Tree::Goal(top_level_goal.to_basic_query()),
        }
    }

    fn options(&self, lib: &Library, prog: &Program) -> Vec<GoalOption> {
        self.tree
            .immediately_partial_steps()
            .into_iter()
            .flat_map(|(path, query)| {
                query.entries.into_iter().map(move |(n, goal)| GoalOption {
                    computation_options: lib
                        .matching_computation_signatures(&goal.name)
                        .into_iter()
                        .cloned()
                        .map(|signature| ComputationOption {
                            assignment_options: egglog_adapter::query(
                                lib,
                                &prog.annotations,
                                todo!(),
                            ),
                            signature,
                        })
                        .collect(),
                    path: path.clone(),
                    goal,
                })
            })
            .collect()
    }

    fn step(&self) {}
}
