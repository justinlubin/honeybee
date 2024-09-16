use crate::pbn::*;

use im::hashmap::HashMap;
use im::hashset::HashSet;
use im::vector::Vector;

pub type Expansion<F> = (HoleName, F);
pub type Oracle<F> = fn(&Exp<F>) -> HashSet<Expansion<F>>;

pub struct Synthesizer<F: Function> {
    oracle: Oracle<F>,
}

impl<F: Function> Synthesizer<F> {
    fn provide(&self, steps: Vector<Step<F>>) -> StepSet<F> {
        if steps.is_empty() {
            (self.oracle)(&Exp::Hole(0))
                .iter()
                .map(|(_, f)| {
                    Step::Intro(
                        f.clone(),
                        f.arity()
                            .iter()
                            .enumerate()
                            .map(|(i, p)| (p.clone(), Exp::Hole(i)))
                            .collect(),
                    )
                })
                .collect()
        } else {
            let es = steps
                .iter()
                .fold(HashSet::new(), |acc, s| s.step(&acc).unwrap());
            assert!(es.len() == 1);
            let e = es.into_iter().next().unwrap();

            (self.oracle)(&e)
                .into_iter()
                .map(|(h, f)| {
                    let base_h = e.fresh();
                    let args: HashMap<String, Exp<F>> = f
                        .arity()
                        .iter()
                        .enumerate()
                        .map(|(i, p)| (p.clone(), Exp::Hole(base_h + i)))
                        .collect();
                    Step::Seq(
                        Box::new(Step::Intro(f.clone(), args.clone())),
                        Box::new(Step::Merge(h, Exp::App(f, args))),
                    )
                })
                .collect()
        }
    }
}
