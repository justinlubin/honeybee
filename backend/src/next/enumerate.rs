use crate::next::core::*;
use crate::next::timer::*;
use crate::next::top_down::*;
use crate::next::traditional_synthesis::*;

use indexmap::IndexMap;

trait Prune {
    fn should_keep(formula: &Formula) -> bool;
}

struct EnumerativeSynthesis<P: Prune> {
    problem: Problem,
    pruner: P,
}

impl<P: Prune> EnumerativeSynthesis<P> {
    fn enumerate<E>(
        &self,
        timer: &impl Timer<E>,
        start: &Exp,
        max_solutions: usize,
    ) -> Result<Vec<IndexMap<HoleName, Exp>>, E> {
        todo!()
    }
}

impl<P: Prune> TraditionalSynthesizer for EnumerativeSynthesis<P> {
    type Exp = Exp;

    fn provide_any<E>(
        &self,
        timer: &impl Timer<E>,
    ) -> Result<Option<Self::Exp>, E> {
        Ok(self
            .enumerate(timer, &Sketch::Hole(0), 1)?
            .into_iter()
            .next()
            .and_then(|sol| sol.into_iter().next().map(|(_, e)| e)))
    }
}

impl<P: Prune> pbn::StepProvider for EnumerativeSynthesis<P> {}

impl<P: Prune> InhabitationOracle for EnumerativeSynthesis<P> {
    type F = ParameterizedFunction;

    fn expansions<E>(
        &mut self,
        e: &Sketch<Self::F>,
        timer: &impl Timer<E>,
    ) -> Result<Vec<(HoleName, Self::F)>, E> {
        let mut steps = vec![];
        for sol in self.enumerate(timer, e, usize::MAX)? {
            for (h, binding) in sol {
                match binding {
                    Sketch::Hole(_) => panic!(),
                    Sketch::App(f, args) => {
                        let step = TopDownStep::Extend(h, f, args);
                        if steps.contains(step) {
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
