use crate::*;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Algorithm {
    PBNHoneybee,
    PBNHoneybeeNoMemo,
    PBNConstructiveOracle,
    NaiveEnumeration,
    PrunedEnumeration,
}

impl Algorithm {
    pub fn all() -> Vec<Self> {
        vec![
            Self::PBNHoneybee,
            Self::PBNHoneybeeNoMemo,
            Self::PBNConstructiveOracle,
            Self::NaiveEnumeration,
            Self::PrunedEnumeration,
        ]
    }

    pub fn controller(
        &self,
        timer: util::Timer,
        problem: core::Problem,
    ) -> pbn::Controller<core::Step> {
        match self {
            Algorithm::PBNHoneybee => {
                let engine = egglog::Egglog::new(true);
                let oracle = dl_oracle::Oracle::new(engine, problem).unwrap();
                let provider =
                    top_down::ClassicalConstructiveSynthesis::new(oracle);
                let start = top_down::Sketch::blank();
                let checker = top_down::GroundChecker::new();
                pbn::Controller::new(timer, provider, checker, start)
            }
            Algorithm::PBNHoneybeeNoMemo => {
                let engine = egglog::Egglog::new(false);
                let oracle = dl_oracle::Oracle::new(engine, problem).unwrap();
                let provider =
                    top_down::ClassicalConstructiveSynthesis::new(oracle);
                let start = top_down::Sketch::blank();
                let checker = top_down::GroundChecker::new();
                pbn::Controller::new(timer, provider, checker, start)
            }
            Algorithm::PBNConstructiveOracle => {
                let pruner = enumerate::ExhaustivePruner;
                let oracle =
                    enumerate::EnumerativeSynthesis::new(problem, pruner);
                let provider =
                    top_down::ClassicalConstructiveSynthesis::new(oracle);
                let start = top_down::Sketch::blank();
                let checker = top_down::GroundChecker::new();
                pbn::Controller::new(timer, provider, checker, start)
            }
            Algorithm::NaiveEnumeration => {
                let pruner = enumerate::NaivePruner;
                let all_synth =
                    enumerate::EnumerativeSynthesis::new(problem, pruner);
                let provider =
                    traditional_synthesis::AllBasedStepProvider(all_synth);
                let start = top_down::Sketch::blank();
                let checker = top_down::GroundChecker::new();
                pbn::Controller::new(timer, provider, checker, start)
            }
            Algorithm::PrunedEnumeration => {
                let pruner = enumerate::ExhaustivePruner;
                let all_synth =
                    enumerate::EnumerativeSynthesis::new(problem, pruner);
                let provider =
                    traditional_synthesis::AllBasedStepProvider(all_synth);
                let start = top_down::Sketch::blank();
                let checker = top_down::GroundChecker::new();
                pbn::Controller::new(timer, provider, checker, start)
            }
        }
    }

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
