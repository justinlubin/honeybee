use crate::pbn::*;
use crate::top_down::*;
use crate::util::{Timer, TimerExpired};

use indexmap::IndexMap;

pub type HoleFilling<F> = IndexMap<HoleName, Sketch<F>>;

/// The type of synthesizers solving the traditional Any task (for sketches).
pub trait AnySynthesizer {
    type F: Function;
    fn provide_any(
        &self,
        timer: &Timer,
        start: &Sketch<Self::F>,
    ) -> Result<Option<HoleFilling<Self::F>>, TimerExpired>;
}

/// The type of synthesizers solving the traditional All task (for sketches).
pub trait AllSynthesizer {
    type F: Function;
    fn provide_all(
        &self,
        timer: &Timer,
        start: &Sketch<Self::F>,
    ) -> Result<Vec<HoleFilling<Self::F>>, TimerExpired>;
}

struct NaiveStepProvider<T: AllSynthesizer>(T);

impl<Synth: AllSynthesizer> StepProvider for NaiveStepProvider<Synth> {
    type Step = TopDownStep<Synth::F>;

    fn provide(
        &mut self,
        timer: &Timer,
        e: &Sketch<Synth::F>,
    ) -> Result<Vec<Self::Step>, TimerExpired> {
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

// impl<F: Function, I: Interaction<Step = TopDownStep<F>>> AnySynthesizer for I {
//     type F = F;
//
//     fn provide_any(
//         &self,
//         start: &Sketch<Self::F>,
//         timer: &T,
//     ) -> Result<Option<HoleFilling<Self::F>>, TimerExpired> {
//         todo!()
//     }
// }
