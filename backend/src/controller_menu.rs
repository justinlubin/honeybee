use crate::{
    core, dl_oracle, egglog, enumerate, pbn, top_down, traditional_synthesis,
    util,
};

pub fn honeybee(
    problem: core::Problem,
    timer: util::Timer,
) -> pbn::Controller<core::Step> {
    let engine = egglog::Egglog::new(true);
    let oracle = dl_oracle::Oracle::new(engine, problem).unwrap();
    let provider = top_down::ClassicalConstructiveSynthesis::new(oracle);
    let start = top_down::Sketch::blank();
    let checker = top_down::GroundChecker::new();
    pbn::Controller::new(timer, provider, checker, start)
}

pub fn honeybee_ablation(
    problem: core::Problem,
    timer: util::Timer,
) -> pbn::Controller<core::Step> {
    let engine = egglog::Egglog::new(false);
    let oracle = dl_oracle::Oracle::new(engine, problem).unwrap();
    let provider = top_down::ClassicalConstructiveSynthesis::new(oracle);
    let start = top_down::Sketch::blank();
    let checker = top_down::GroundChecker::new();
    pbn::Controller::new(timer, provider, checker, start)
}

pub fn pruned_enumerate(
    problem: core::Problem,
    timer: util::Timer,
) -> pbn::Controller<core::Step> {
    let pruner = enumerate::ExhaustivePruner;
    let all_synth = enumerate::EnumerativeSynthesis::new(problem, pruner);
    let provider = traditional_synthesis::AllBasedStepProvider(all_synth);
    let start = top_down::Sketch::blank();
    let checker = top_down::GroundChecker::new();
    pbn::Controller::new(timer, provider, checker, start)
}

pub fn naive_enumeration(
    problem: core::Problem,
    timer: util::Timer,
) -> pbn::Controller<core::Step> {
    let pruner = enumerate::NaivePruner;
    let all_synth = enumerate::EnumerativeSynthesis::new(problem, pruner);
    let provider = traditional_synthesis::AllBasedStepProvider(all_synth);
    let start = top_down::Sketch::blank();
    let checker = top_down::GroundChecker::new();
    pbn::Controller::new(timer, provider, checker, start)
}
