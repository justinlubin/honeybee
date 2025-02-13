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

pub fn ep(
    problem: core::Problem,
    timer: util::Timer,
) -> pbn::Controller<core::Step> {
    let pruner = enumerate::ExhaustivePruner;
    let all_synth = enumerate::EnumerativeSynthesis::new(problem, pruner);
    let provider = traditional_synthesis::NaiveStepProvider(all_synth);
    let start = top_down::Sketch::blank();
    let checker = top_down::GroundChecker::new();
    pbn::Controller::new(timer, provider, checker, start)
}
