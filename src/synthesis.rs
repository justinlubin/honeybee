use crate::derivation;

use crate::ir::*;

pub struct Synthesizer {
    tree: derivation::Tree,
}

pub enum Msg {
    Success,
    InvalidBreadcrumbs,
}

impl Synthesizer {
    fn new(top_level_goal: Fact) -> Synthesizer {
        Synthesizer {
            tree: derivation::Tree::Goal(top_level_goal),
        }
    }

    fn step(&self) {}
}
