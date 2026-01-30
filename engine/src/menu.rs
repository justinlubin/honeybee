//! # A menu of possible synthesizers and code generators
//!
//! This module hooks together all the components (ingredients) of this project
//! into a set of items on a menu of possible choices. To the extent possible,
//! these menu items all share the same interface and can be used in the same
//! way (e.g. as controllers for Programming By Navigation).

use crate::*;

use serde::{Deserialize, Serialize};

////////////////////////////////////////////////////////////////////////////////
// Synthesizers

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Algorithm {
    PBNHoneybee,
    PBNHoneybeeNoMemo,
    PBNConstructiveOracle,
    NaiveEnumeration,
    PrunedEnumeration,
}

impl Algorithm {
    /// The list of all the possible synthesizers
    pub fn all() -> Vec<Self> {
        vec![
            Self::PBNHoneybee,
            Self::PBNHoneybeeNoMemo,
            Self::PBNConstructiveOracle,
            Self::NaiveEnumeration,
            Self::PrunedEnumeration,
        ]
    }

    /// Returns a controller to solve the Programming By Navigation Synthesis
    /// Problem using the underlying synthesis algorithm
    pub fn controller(
        &self,
        timer: util::Timer,
        problem: core::Problem,
        save_history: bool,
    ) -> pbn::Controller<util::Timer, core::Step> {
        match self {
            Algorithm::PBNHoneybee => {
                let engine = egglog::Egglog::new(true);
                let oracle = dl_oracle::Oracle::new(engine, problem).unwrap();
                let provider =
                    top_down::ClassicalConstructiveSynthesis::new(oracle);
                let start = top_down::Sketch::blank();
                let checker = top_down::GroundChecker::new();
                pbn::Controller::new(
                    timer,
                    provider,
                    checker,
                    start,
                    save_history,
                )
            }
            Algorithm::PBNHoneybeeNoMemo => {
                let engine = egglog::Egglog::new(false);
                let oracle = dl_oracle::Oracle::new(engine, problem).unwrap();
                let provider =
                    top_down::ClassicalConstructiveSynthesis::new(oracle);
                let start = top_down::Sketch::blank();
                let checker = top_down::GroundChecker::new();
                pbn::Controller::new(
                    timer,
                    provider,
                    checker,
                    start,
                    save_history,
                )
            }
            Algorithm::PBNConstructiveOracle => {
                let pruner = enumerate::ExhaustivePruner;
                let oracle =
                    enumerate::EnumerativeSynthesis::new(problem, pruner);
                let provider =
                    top_down::ClassicalConstructiveSynthesis::new(oracle);
                let start = top_down::Sketch::blank();
                let checker = top_down::GroundChecker::new();
                pbn::Controller::new(
                    timer,
                    provider,
                    checker,
                    start,
                    save_history,
                )
            }
            Algorithm::NaiveEnumeration => {
                let pruner = enumerate::NaivePruner;
                let all_synth =
                    enumerate::EnumerativeSynthesis::new(problem, pruner);
                let provider =
                    traditional_synthesis::AllBasedStepProvider(all_synth);
                let start = top_down::Sketch::blank();
                let checker = top_down::GroundChecker::new();
                pbn::Controller::new(
                    timer,
                    provider,
                    checker,
                    start,
                    save_history,
                )
            }
            Algorithm::PrunedEnumeration => {
                let pruner = enumerate::ExhaustivePruner;
                let all_synth =
                    enumerate::EnumerativeSynthesis::new(problem, pruner);
                let provider =
                    traditional_synthesis::AllBasedStepProvider(all_synth);
                let start = top_down::Sketch::blank();
                let checker = top_down::GroundChecker::new();
                pbn::Controller::new(
                    timer,
                    provider,
                    checker,
                    start,
                    save_history,
                )
            }
        }
    }

    /// Returns an AnySynthesizer to solve the traditional Any task using the
    /// underlying synthesis algorithm
    pub fn any_synthesizer(
        &self,
        problem: core::Problem,
    ) -> Box<
        dyn traditional_synthesis::AnySynthesizer<
            F = core::ParameterizedFunction,
        >,
    > {
        match self {
            Algorithm::PBNHoneybee => {
                let engine = egglog::Egglog::new(true);
                let oracle = dl_oracle::Oracle::new(engine, problem).unwrap();
                let provider =
                    top_down::ClassicalConstructiveSynthesis::new(oracle);
                let checker = top_down::GroundChecker::new();
                let synth =
                    traditional_synthesis::StepProviderBasedAnySynthesizer::new(
                        provider, checker,
                    );
                Box::new(synth)
            }
            Algorithm::PBNHoneybeeNoMemo => {
                let engine = egglog::Egglog::new(false);
                let oracle = dl_oracle::Oracle::new(engine, problem).unwrap();
                let provider =
                    top_down::ClassicalConstructiveSynthesis::new(oracle);
                let checker = top_down::GroundChecker::new();
                let synth =
                    traditional_synthesis::StepProviderBasedAnySynthesizer::new(
                        provider, checker,
                    );
                Box::new(synth)
            }
            Algorithm::PBNConstructiveOracle => {
                let pruner = enumerate::ExhaustivePruner;
                let oracle =
                    enumerate::EnumerativeSynthesis::new(problem, pruner);
                let provider =
                    top_down::ClassicalConstructiveSynthesis::new(oracle);
                let checker = top_down::GroundChecker::new();
                let synth =
                    traditional_synthesis::StepProviderBasedAnySynthesizer::new(
                        provider, checker,
                    );
                Box::new(synth)
            }
            Algorithm::NaiveEnumeration => {
                let pruner = enumerate::NaivePruner;
                let synth =
                    enumerate::EnumerativeSynthesis::new(problem, pruner);
                Box::new(synth)
            }
            Algorithm::PrunedEnumeration => {
                let pruner = enumerate::ExhaustivePruner;
                let synth =
                    enumerate::EnumerativeSynthesis::new(problem, pruner);
                Box::new(synth)
            }
        }
    }
}

impl std::str::FromStr for Algorithm {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(&format!("\"{}\"", s))
    }
}

////////////////////////////////////////////////////////////////////////////////
// Code generators

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum CodegenStyle {
    PlainTextNotebook,
    Simple,
}

impl CodegenStyle {
    /// The list of all the possible code generators
    pub fn all() -> Vec<Self> {
        vec![Self::PlainTextNotebook, Self::Simple]
    }

    /// Returns a code generator for the given style
    pub fn codegen(
        &self,
        library: core::Library,
    ) -> Result<Box<dyn Codegen>, String> {
        match self {
            Self::PlainTextNotebook => {
                Ok(Box::new(codegen::PlainTextNotebook::new(library)))
            }
            Self::Simple => Ok(Box::new(codegen::Simple {
                indent: 1,
                color: true,
            })),
        }
    }
}

impl std::str::FromStr for CodegenStyle {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(&format!("\"{}\"", s))
    }
}
