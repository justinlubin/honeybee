use std::hash::Hash;

use im::hashmap::HashMap;
use im::hashset::HashSet;
use im::vector::Vector;

pub type HoleName = usize;

pub trait Function: Clone + Eq + Hash + PartialEq {
    fn arity(&self) -> HashSet<String>;
}

#[derive(Clone, Eq, Hash, PartialEq)]
pub enum Exp<F: Function> {
    Hole(HoleName),
    App(F, HashMap<String, Exp<F>>),
}

pub type ExpSet<F> = HashSet<Exp<F>>;

#[derive(Clone, Eq, Hash, PartialEq)]
pub enum Step<F: Function> {
    Intro(F, HashMap<String, Exp<F>>),
    Merge(HoleName, Exp<F>),
    Seq(Box<Step<F>>, Box<Step<F>>),
}

pub type StepSet<F> = HashSet<Step<F>>;

impl<F: Function> Exp<F> {
    pub fn has_subexpression(&self, e: &Exp<F>) -> bool {
        if self == e {
            return true;
        }
        match self {
            Exp::Hole(_) => false,
            Exp::App(_, args) => {
                args.iter().any(|(_, e_prime)| e_prime.has_subexpression(e))
            }
        }
    }

    pub fn substitute(&self, h: HoleName, e: &Exp<F>) -> Exp<F> {
        match self {
            Exp::Hole(h_prime) if *h_prime == h => e.clone(),
            Exp::Hole(_) => self.clone(),
            Exp::App(f, args) => Exp::App(
                f.clone(),
                args.iter()
                    .map(|(x, e_prime)| (x.clone(), e_prime.substitute(h, e)))
                    .collect(),
            ),
        }
    }

    fn max_hole(&self) -> HoleName {
        match self {
            Exp::Hole(h) => *h,
            Exp::App(_, args) => {
                args.iter().map(|(_, e)| e.max_hole()).max().unwrap_or(0)
            }
        }
    }

    pub fn fresh(&self) -> HoleName {
        self.max_hole() + 1
    }
}

impl<F: Function> Step<F> {
    pub fn step(&self, es: &ExpSet<F>) -> Option<ExpSet<F>> {
        match self {
            Step::Intro(f, args) => {
                let e = Exp::App(f.clone(), args.clone());
                if f.arity().len() == args.len() && !es.contains(&e) {
                    Some(es.update(e))
                } else {
                    None
                }
            }
            Step::Merge(h, e) => {
                let he = Exp::Hole(*h);
                if es.contains(e)
                    && !e.has_subexpression(&he)
                    && es.iter().any(|x| x.has_subexpression(&he))
                {
                    Some(
                        es.iter()
                            .filter_map(|x| {
                                if x == e {
                                    None
                                } else {
                                    Some(x.substitute(*h, e))
                                }
                            })
                            .collect(),
                    )
                } else {
                    None
                }
            }
            Step::Seq(s1, s2) => {
                s1.step(es).and_then(|es_prime| s2.step(&es_prime))
            }
        }
    }
}

pub trait StepProvider<F: Function>: Fn(Vector<Step<F>>) -> StepSet<F> {}
