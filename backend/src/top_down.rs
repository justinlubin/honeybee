//! # Top-down steps for Programming By Navigation
//!
//! This module defines a particular kind of steps and expressions to work with
//! the Programming By Navigation framework. In the instantiation provided by
//! this module, expressions are sketches (function applications and holes), and
//! steps extend these holes with a new function application.
//!
//! A variety of notions of validity could be built on top of this concrete
//! instantiation; this module makes no requirement on which sketches are valid
//! beyoned the requirement that functions be applied to the correct number of
//! arguments (with the right keyword arguments).

use crate::pbn;
use crate::util::{self, EarlyCutoff, Timer};

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

////////////////////////////////////////////////////////////////////////////////
// Expressions

/// The type of hole names (used to identify holes).
pub type HoleName = usize;

/// The type of function parameter keys.
///
/// All functions will use keyword-only arguments; these keywords are
/// represented by values of this type.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct FunParam(pub String);

/// The type of functions.
///
/// The only requirement is that a function take in a fixed arity of keyword
/// arguments.
pub trait Function: Clone + Eq {
    fn arity(&self) -> Vec<FunParam>;
}

/// The type of sketches parameterized by a notion of functions.
///
/// Sketches can either be a hole or an application of the function to more
/// sketches. To be valid, the arguments to function applications must match
/// the function's arity; downstream applications are likely to put additional
/// constraints on what a sketch must be in order to be valid.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum Sketch<F: Function> {
    Hole(HoleName),
    App(F, IndexMap<FunParam, Self>),
}

impl<F: Function> Sketch<F> {
    /// Returns the blank expression
    pub fn blank() -> Self {
        Self::Hole(0)
    }

    /// Returns whether or not an expression is ground
    pub fn ground(&self) -> bool {
        match self {
            Sketch::Hole(_) => false,
            Sketch::App(_, args) => args.values().all(|s| s.ground()),
        }
    }

    /// Returns the size of an expression
    pub fn size(&self) -> usize {
        match self {
            Sketch::Hole(_) => 1,
            Sketch::App(_, args) => {
                1 + args.values().map(|x| x.size()).sum::<usize>()
            }
        }
    }
}

/// Pretty-print a hole
pub fn pretty_hole_string(h: HoleName) -> String {
    ansi_term::Color::Yellow
        .paint(plain_hole_string(h))
        .to_string()
}

/// Plain-print a hole
pub fn plain_hole_string(h: HoleName) -> String {
    format!("?{}", util::subscript_numbers(&h.to_string()))
}

/// Wrapper for checking whether or not an expression is ground
pub struct GroundChecker<F: Function> {
    function_type: PhantomData<F>,
}

impl<F: Function> GroundChecker<F> {
    pub fn new() -> Self {
        Self {
            function_type: PhantomData,
        }
    }
}

impl<F: Function> pbn::ValidityChecker for GroundChecker<F> {
    type Exp = Sketch<F>;

    fn check(&self, e: &Self::Exp) -> bool {
        e.ground()
    }
}

////////////////////////////////////////////////////////////////////////////////
// Steps

/// The type of top-down steps.
///
/// Top-down steps can either extend a hole with a new function application, or
/// they can be a sequence of other top-down steps.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TopDownStep<F: Function> {
    Extend(HoleName, F, IndexMap<FunParam, Sketch<F>>),
    Seq(Box<Self>, Box<Self>),
}

impl<F: Function> Sketch<F> {
    /// Creates a new function application whose arguments are fresh holes
    pub fn free(context: &Sketch<F>, f: &F) -> Sketch<F> {
        let holes = context.fresh().map(Sketch::Hole);
        Sketch::App(f.clone(), f.arity().into_iter().zip(holes).collect())
    }

    fn has_subterm(&self, e: &Self) -> bool {
        if self == e {
            return true;
        }
        match self {
            Self::Hole(_) => false,
            Self::App(_, args) => args.values().any(|v| v.has_subterm(e)),
        }
    }

    /// Substitute a hole for an expression
    pub fn substitute(&self, h: HoleName, e: &Self) -> Self {
        match self {
            Self::Hole(h2) => {
                if *h2 == h {
                    e.clone()
                } else {
                    Self::Hole(*h2)
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

    /// Substitute a unique hole for an expression and also return the parent
    /// (if not the root)
    #[allow(dead_code)]
    pub fn substitute_unique(
        &self,
        h: HoleName,
        e: &Self,
    ) -> (Self, Option<Self>) {
        let mut parent = None;
        let mut modified = false;
        let res =
            self.substitute_unique_helper(h, e, &mut parent, &mut modified);
        (res, parent)
    }

    fn substitute_unique_helper(
        &self,
        h: HoleName,
        e: &Self,
        parent: &mut Option<Self>,
        modified: &mut bool,
    ) -> Self {
        match self {
            Self::Hole(h2) => {
                if *h2 == h {
                    *modified = true;
                    e.clone()
                } else {
                    Self::Hole(*h2)
                }
            }
            Self::App(f, args) => {
                let res = Self::App(
                    f.clone(),
                    args.iter()
                        .map(|(k, v)| {
                            (
                                k.clone(),
                                v.substitute_unique_helper(
                                    h, e, parent, modified,
                                ),
                            )
                        })
                        .collect(),
                );
                if *modified && *parent == None {
                    *parent = Some(res.clone());
                }
                res
            }
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

    /// Returns an iterator of fresh hole names
    pub fn fresh(&self) -> impl Iterator<Item = HoleName> {
        return (self.max_hole() + 1)..;
    }

    /// Pattern match a ground expression against self
    pub fn pattern_match(
        &self,
        ground: &Self,
    ) -> Option<IndexMap<HoleName, Self>> {
        match self {
            Sketch::Hole(h) => Some(IndexMap::from([(*h, ground.clone())])),
            Sketch::App(f, fargs) => match ground {
                Sketch::Hole(_) => None,
                Sketch::App(g, gargs) => {
                    if f == g {
                        let mut ret = IndexMap::new();
                        for (fp, farg) in fargs {
                            let garg = gargs.get(fp)?;
                            ret.extend(farg.pattern_match(garg)?);
                        }
                        Some(ret)
                    } else {
                        None
                    }
                }
            },
        }
    }
}

impl<F: Function> pbn::Step for TopDownStep<F> {
    type Exp = Sketch<F>;

    fn apply(&self, e: &Self::Exp) -> Option<Self::Exp> {
        match self {
            Self::Extend(h, f, args) => {
                if f.arity().len() == args.len()
                    && e.has_subterm(&Self::Exp::Hole(*h))
                {
                    Some(e.substitute(
                        *h,
                        &Self::Exp::App(f.clone(), args.clone()),
                    ))
                } else {
                    None
                }
            }
            Self::Seq(s1, s2) => s1.apply(e).and_then(|e2| s2.apply(&e2)),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// Top-Down Classical-Constructive Synthesis
//   (Solving the Programming By Navigation Synthesis Problem)

/// The type of expansions
pub type Expansion<F> = (HoleName, F);

/// The type of inhabitation oracles for use in top-down classical-constructive
/// synthesis.
pub trait InhabitationOracle {
    type F: Function;
    fn expansions(
        &mut self,
        timer: &Timer,
        e: &Sketch<Self::F>,
    ) -> Result<Vec<Expansion<Self::F>>, EarlyCutoff>;
}

/// Top-down classical-constructive synthesis, a solution to the Programming By
/// Navigation Synthesis Problem.
pub struct ClassicalConstructiveSynthesis<O: InhabitationOracle> {
    pub oracle: O,
}

impl<O: InhabitationOracle> ClassicalConstructiveSynthesis<O> {
    pub fn new(oracle: O) -> Self {
        Self { oracle }
    }
}

impl<O: InhabitationOracle> pbn::StepProvider
    for ClassicalConstructiveSynthesis<O>
{
    type Step = TopDownStep<O::F>;
    fn provide(
        &mut self,
        timer: &Timer,
        e: &<Self::Step as pbn::Step>::Exp,
    ) -> Result<Vec<Self::Step>, EarlyCutoff> {
        let mut ret = vec![];
        for (h, f) in self.oracle.expansions(timer, e)? {
            let holes = e.fresh().map(Sketch::Hole);
            ret.push(TopDownStep::Extend(
                h,
                f.clone(),
                f.arity().into_iter().zip(holes).collect(),
            ));
        }
        Ok(ret)
    }
}
