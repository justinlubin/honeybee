//! # Unsound Inhabitation Oracle
//!
//! This module defines an **unsound** inhabitation [`UnsoundOracle`].
//! When using with top-down classical-constructive synthesis, the resulting
//! algorithm satisfies Strong Completeness but not Strong Soundness.

use crate::core;
use crate::enumerate;
use crate::top_down;
use crate::util;

pub struct UnsoundOracle {
    synthesizer: enumerate::EnumerativeSynthesis<enumerate::NaivePruner>,
}

impl UnsoundOracle {
    pub fn new(problem: core::Problem) -> Self {
        Self {
            synthesizer: enumerate::EnumerativeSynthesis::new_unsound(problem),
        }
    }
}

impl top_down::InhabitationOracle for UnsoundOracle {
    type F = core::ParameterizedFunction;

    fn expansions(
        &mut self,
        timer: &util::Timer,
        e: &top_down::Sketch<Self::F>,
    ) -> Result<Vec<top_down::Expansion<Self::F>>, util::EarlyCutoff> {
        Ok(self
            .synthesizer
            .all_expansions(timer, e)?
            .into_iter()
            .flat_map(|(lhs, all_rhs)| {
                all_rhs.into_iter().map(move |rhs| (lhs, rhs))
            })
            .collect())
    }
}
