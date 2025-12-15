use crate::top_down::*;
use crate::util;

use pbn::*;

use indexmap::IndexMap;

/// The type of hole fillings (mappings from hole names to sketches).
pub type HoleFilling<F> = IndexMap<HoleName, Sketch<F>>;

/// The type of synthesizers solving the traditional Any task (for sketches).
pub trait AnySynthesizer {
    type F: Function;
    fn provide_any(
        &mut self,
        timer: &util::Timer,
        start: &Sketch<Self::F>,
    ) -> Result<Option<HoleFilling<Self::F>>, util::EarlyCutoff>;
}

/// The type of synthesizers solving the traditional All task (for sketches).
pub trait AllSynthesizer {
    type F: Function;
    fn provide_all(
        &mut self,
        timer: &util::Timer,
        start: &Sketch<Self::F>,
    ) -> Result<Vec<HoleFilling<Self::F>>, util::EarlyCutoff>;
}

/// A wrapper for solving the Programming By Navigation Synthesis Problem using
/// a complete All synthesizer.
pub struct AllBasedStepProvider<Synth: AllSynthesizer>(pub Synth);

impl<Synth: AllSynthesizer> StepProvider<util::Timer>
    for AllBasedStepProvider<Synth>
{
    type Step = TopDownStep<Synth::F>;

    fn provide(
        &mut self,
        timer: &util::Timer,
        e: &Sketch<Synth::F>,
    ) -> Result<Vec<Self::Step>, util::EarlyCutoff> {
        let mut steps = vec![];
        for solution in self.0.provide_all(timer, e)? {
            for (h, binding) in solution {
                match binding {
                    Sketch::Hole(_) => panic!(),
                    Sketch::App(f, args) => {
                        let step = TopDownStep::Extend(h, f, args);
                        steps.push(step);
                    }
                }
            }
        }
        Ok(steps)
    }
}

/// A wrapper for solving the traditional Any task using a Programming By
/// Navigation synthesizer.
pub struct StepProviderBasedAnySynthesizer<
    F: Function,
    SP: StepProvider<util::Timer, Step = TopDownStep<F>>,
    V: ValidityChecker<Exp = Sketch<F>>,
> {
    provider: SP,
    checker: V,
}

impl<
        F: Function,
        SP: StepProvider<util::Timer, Step = TopDownStep<F>>,
        V: ValidityChecker<Exp = Sketch<F>>,
    > StepProviderBasedAnySynthesizer<F, SP, V>
{
    pub fn new(provider: SP, checker: V) -> Self {
        Self { provider, checker }
    }
}

impl<
        F: Function,
        SP: StepProvider<util::Timer, Step = TopDownStep<F>>,
        V: ValidityChecker<Exp = Sketch<F>>,
    > AnySynthesizer for StepProviderBasedAnySynthesizer<F, SP, V>
{
    type F = F;

    fn provide_any(
        &mut self,
        timer: &util::Timer,
        start: &Sketch<Self::F>,
    ) -> Result<Option<HoleFilling<Self::F>>, util::EarlyCutoff> {
        let mut ret = start.clone();
        loop {
            if ret.size() > util::MAX_EXP_SIZE {
                return Err(util::EarlyCutoff::OutOfMemory);
            }

            let options = self.provider.provide(timer, &ret)?;
            let step = match options.into_iter().next() {
                Some(step) => step,
                None => return Ok(None),
            };
            ret = step.apply(&ret).unwrap();
            if self.checker.check(&ret) {
                return Ok(start.pattern_match(&ret));
            }
        }
    }
}
