use crate::next::pbn::*;
use crate::next::top_down::*;
use crate::next::util::Timer;

use indexmap::IndexMap;

pub type HoleFilling<F> = IndexMap<HoleName, Sketch<F>>;

/// The type of synthesizers solving the traditional Any task (for sketches).
pub trait AnySynthesizer {
    type F: Function;
    fn provide_any<T: Timer>(
        &self,
        start: &Sketch<Self::F>,
        timer: &T,
    ) -> Result<Option<HoleFilling<Self::F>>, T::Expired>;
}

/// The type of synthesizers solving the traditional All task (for sketches).
pub trait AllSynthesizer {
    type F: Function;
    fn provide_all<T: Timer>(
        &self,
        start: &Sketch<Self::F>,
        timer: &T,
    ) -> Result<Vec<HoleFilling<Self::F>>, T::Expired>;
}

struct NaiveStepProvider<T: AllSynthesizer>(T);

impl<Synth: AllSynthesizer> StepProvider for NaiveStepProvider<Synth> {
    type Step = TopDownStep<Synth::F>;

    fn provide<T: Timer>(
        &mut self,
        e: &Sketch<Synth::F>,
        timer: &T,
    ) -> Result<Vec<Self::Step>, T::Expired> {
        let mut steps = vec![];
        for solution in self.0.provide_all(e, timer)? {
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

impl<F: Function, I: Interaction<Step = TopDownStep<F>>> AnySynthesizer for I {
    type F = F;

    fn provide_any<T: Timer>(
        &self,
        start: &Sketch<Self::F>,
        timer: &T,
    ) -> Result<Option<HoleFilling<Self::F>>, T::Expired> {
        todo!()
    }
}
