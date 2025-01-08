use indexmap::IndexMap;
use indexmap::IndexSet;
use std::hash::Hash;

pub trait Expression {
    fn valid(&self) -> bool;
}

pub trait Step {
    type Exp: Expression;
    fn step(&self, e: &Self::Exp) -> Option<Self::Exp>;
}

pub type StepSet<S: Step> = IndexSet<S>;

/////////////

pub type HoleName = usize;

pub trait Function {
    fn arity(&self) -> IndexSet<String>;
}

pub enum Sketch<F: Function> {
    Hole(HoleName),
    App(F, IndexMap<String, Self>),
}

pub enum TDStep<F: Function> {
    Extend(HoleName, F, IndexMap<String, Sketch<F>>),
    Seq(Box<Self>, Box<Self>),
}

impl<F: Function> Sketch<F> {
    pub fn has_subterm(&self, e: &Self) -> bool {
        if self == e {
            return true;
        }
        match self {
            Self::Hole(_) => false,
            Self::App(_, args) => args.values().any(|v| v.has_subexpression(e)),
        }
    }

    pub fn substitute(&self, h: HoleName, e: &Self) -> Self {
        match self {
            Self::Hole(h2) => {
                if *h2 == h {
                    e.clone()
                } else {
                    Self::Hole(h2)
                }
            }
            Self::App(f, args) => Self::App(
                f.clone(),
                args.iter()
                    .map(|(k, v)| (k.clone(), v.substitute(h, e)))
                    .collect(),
            ),
        }
    }

    fn max_hole(&self) -> HoleName {
        match self {
            Self::Hole(h) => *h,
            Self::App(_, args) => {
                args.values().map(|v| v.max_hole()).max().unwrap_or(0)
            }
        }
    }

    pub fn fresh(&self) -> HoleName {
        self.max_hole() + 1
    }
}

impl<F: Function> Step for TDStep<F> {
    type Exp = Sketch<F>;

    fn step(&self, e: &Self::Exp) -> Option<Self::Exp> {
        match self {
            Self::Extend(h, f, args) => {
                if f.arity().len() == args.len()
                    && e.has_subterm(&Self::Exp::Hole(h))
                {
                    Some(e.substitute(h, &Self::Exp::App(f, args)))
                } else {
                    None
                }
            }
            Self::Seq(s1, s2) => s1.step(e).and_then(|e2| s2.step(&e2)),
        }
    }
}

//pub trait StepProvider<F: Function>: Fn(Vector<Step<F>>) -> StepSet<F> {}
