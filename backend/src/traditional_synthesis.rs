use crate::pbn::*;
use crate::top_down::*;
use crate::util::{EarlyCutoff, Timer};

use indexmap::IndexMap;

pub type HoleFilling<F> = IndexMap<HoleName, Sketch<F>>;

/// The type of synthesizers solving the traditional Any task (for sketches).
pub trait AnySynthesizer {
    type F: Function;
    fn provide_any(
        &mut self,
        timer: &Timer,
        start: &Sketch<Self::F>,
    ) -> Result<Option<HoleFilling<Self::F>>, EarlyCutoff>;
}

/// The type of synthesizers solving the traditional All task (for sketches).
pub trait AllSynthesizer {
    type F: Function;
    fn provide_all(
        &mut self,
        timer: &Timer,
        start: &Sketch<Self::F>,
    ) -> Result<Vec<HoleFilling<Self::F>>, EarlyCutoff>;
}

pub struct AllBasedStepProvider<Synth: AllSynthesizer>(pub Synth);

impl<Synth: AllSynthesizer> StepProvider for AllBasedStepProvider<Synth> {
    type Step = TopDownStep<Synth::F>;

    fn provide(
        &mut self,
        timer: &Timer,
        e: &Sketch<Synth::F>,
    ) -> Result<Vec<Self::Step>, EarlyCutoff> {
        let mut steps = vec![];
        for solution in self.0.provide_all(timer, e)? {
            for (h, binding) in solution {
                match binding {
                    Sketch::Hole(_) => panic!(),
                    Sketch::App(f, args) => {
                        let step = TopDownStep::Extend(h, f, args);
                        if steps.contains(&step) {
                            continue;
                        }
                        steps.push(step);
                    }
                }
            }
        }
        Ok(steps)
    }
}

pub struct StepProviderBasedAnySynthesizer<
    F: Function,
    SP: StepProvider<Step = TopDownStep<F>>,
    V: ValidityChecker<Exp = Sketch<F>>,
> {
    provider: SP,
    checker: V,
}

impl<
        F: Function,
        SP: StepProvider<Step = TopDownStep<F>>,
        V: ValidityChecker<Exp = Sketch<F>>,
    > StepProviderBasedAnySynthesizer<F, SP, V>
{
    pub fn new(provider: SP, checker: V) -> Self {
        Self { provider, checker }
    }
}

impl<
        F: Function,
        SP: StepProvider<Step = TopDownStep<F>>,
        V: ValidityChecker<Exp = Sketch<F>>,
    > AnySynthesizer for StepProviderBasedAnySynthesizer<F, SP, V>
{
    type F = F;

    fn provide_any(
        &mut self,
        timer: &Timer,
        start: &Sketch<Self::F>,
    ) -> Result<Option<HoleFilling<Self::F>>, EarlyCutoff> {
        let mut ret = start.clone();
        loop {
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
