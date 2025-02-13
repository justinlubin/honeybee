use crate::{core, dl_oracle, egglog, pbn, top_down, util};

pub fn honeybee(
    problem: core::Problem,
    timer: util::Timer,
) -> pbn::Controller<core::Step> {
    let engine = egglog::Egglog::new(true);
    let oracle = dl_oracle::Oracle::new(engine, problem).unwrap();
    let ccs = top_down::ClassicalConstructiveSynthesis::new(oracle);
    let start = top_down::Sketch::blank();
    let checker = top_down::GroundChecker::new();
    pbn::Controller::new(timer, ccs, checker, start)
}
