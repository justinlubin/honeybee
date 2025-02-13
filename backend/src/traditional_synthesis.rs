use crate::pbn::*;
use crate::top_down::*;
use crate::util::Timer;

use indexmap::IndexMap;

pub type HoleFilling<F> = IndexMap<HoleName, Sketch<F>>;

/// The type of synthesizers solving the traditional Any task (for sketches).
pub trait AnySynthesizer {
    type F: Function;
    fn provide_any<T: Timer>(
        &self,
        timer: &T,
        start: &Sketch<Self::F>,
    ) -> Result<Option<HoleFilling<Self::F>>, T::Expired>;
}

/// The type of synthesizers solving the traditional All task (for sketches).
pub trait AllSynthesizer {
    type F: Function;
    fn provide_all<T: Timer>(
        &self,
        timer: &T,
        start: &Sketch<Self::F>,
    ) -> Result<Vec<HoleFilling<Self::F>>, T::Expired>;
}

struct NaiveStepProvider<T: AllSynthesizer>(T);

impl<T: Timer, Synth: AllSynthesizer> StepProvider<T>
    for NaiveStepProvider<Synth>
{
    type Step = TopDownStep<Synth::F>;

    fn provide(
        &mut self,
        timer: &T,
        e: &Sketch<Synth::F>,
    ) -> Result<Vec<Self::Step>, T::Expired> {
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
//     fn provide_any<T: Timer>(
//         &self,
//         start: &Sketch<Self::F>,
//         timer: &T,
//     ) -> Result<Option<HoleFilling<Self::F>>, T::Expired> {
//         todo!()
//     }
// }
