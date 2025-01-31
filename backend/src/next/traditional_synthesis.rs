use crate::next::pbn::*;
use crate::next::timer::*;
use crate::next::top_down::*;

use indexmap::IndexMap;

pub type HoleFilling<F> = IndexMap<HoleName, Sketch<F>>;

/// The type of synthesizers solving the traditional Any task (for sketches).
pub trait AnySynthesizer {
    type F: Function;
    fn provide_any<E>(
        &self,
        start: &Sketch<Self::F>,
        timer: &impl Timer<E>,
    ) -> Result<Option<HoleFilling<Self::F>>, E>;
}

/// The type of synthesizers solving the traditional All task (for sketches).
pub trait AllSynthesizer {
    type F: Function;
    fn provide_all<E>(
        &self,
        start: &Sketch<Self::F>,
        timer: &impl Timer<E>,
    ) -> Result<Vec<HoleFilling<Self::F>>, E>;
}

struct NaiveStepProvider<T: AllSynthesizer>(T);

impl<T: AllSynthesizer> StepProvider for NaiveStepProvider<T> {
    type Step = TopDownStep<T::F>;

    fn provide<E>(
        &mut self,
        e: &Sketch<T::F>,
        timer: &impl Timer<E>,
    ) -> Result<Vec<Self::Step>, E> {
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

impl<F: Function, T: Interaction<Step = TopDownStep<F>>> AnySynthesizer for T {
    type F = F;

    fn provide_any<E>(
        &self,
        start: &Sketch<Self::F>,
        timer: &impl Timer<E>,
    ) -> Result<Option<HoleFilling<Self::F>>, E> {
        todo!()
    }
}
