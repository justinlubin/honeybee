//! # Term enumeration
//!
//! This module implements term enumeration that is parameterized by a pruning
//! algorithm; the main struct is [`EnumerativeSynthesis`]. This struct
//! implements the traditional Any and All tasks (and thus can be used as a
//! Programming By Navigation step provider), and it *also* implements the
//! required interface for an inhabitation oracle, so it can be used as a
//! "fully-constructive" oracle for top-down classical-constructive program
//! synthesis.

use crate::core::*;
use crate::top_down::*;
use crate::traditional_synthesis::*;
use crate::util::{self, EarlyCutoff, Timer};
use crate::{eval, typecheck};

use indexmap::{IndexMap, IndexSet};
use std::collections::VecDeque;

/// The domain of values to use; use to construct the "support" of various
/// components, which is the set of possible values that could be filled in.
pub struct Support {
    ints: Vec<Value>,
    strings: Vec<Value>,
}

impl Support {
    /// Create a new support from a set of values
    fn new(values: IndexSet<Value>) -> Self {
        let mut ints = vec![];
        let mut strings = vec![];

        for v in values {
            match v {
                Value::Bool(_) => (),
                Value::Int(_) => ints.push(v),
                Value::Str(_) => strings.push(v),
            }
        }

        Self { ints, strings }
    }

    fn value_type(&self, vt: &ValueType) -> Vec<Value> {
        match vt {
            ValueType::Bool => vec![Value::Bool(true), Value::Bool(false)],
            ValueType::Int => self.ints.clone(),
            ValueType::Str => self.strings.clone(),
        }
    }

    fn met_signature(
        &self,
        timer: &Timer,
        sig: &MetSignature,
    ) -> Result<Vec<IndexMap<MetParam, Value>>, EarlyCutoff> {
        let choices = sig
            .params
            .iter()
            .map(|(k, vt)| (k.clone(), self.value_type(vt)))
            .collect();
        util::cartesian_product(timer, choices)
    }
}

/// The type of pruning algorithms. To be used for Programming By Navigation, a
/// pruning algorithm MUST NOT filter out any valid programs (enumeration must
/// be complete). It is entirely okay not to filter out invalid programs
/// (enumeration need not be sound), though, as all enumerated programs
/// ultimately go through a final post hoc validity check (which automatically
/// enforces soundness).
pub trait Prune {
    fn possible(
        &self,
        timer: &Timer,
        problem: &Problem,
        support: &Support,
        e: &Exp,
    ) -> Result<bool, EarlyCutoff>;
}

/// A pruner that does not filter out any programs.
pub struct NaivePruner;

impl Prune for NaivePruner {
    fn possible(
        &self,
        _: &Timer,
        _: &Problem,
        _: &Support,
        _: &Exp,
    ) -> Result<bool, EarlyCutoff> {
        Ok(true)
    }
}

/// A pruner that tries to identify when no set of values can satisfy the
/// provided formulas.
pub struct ExhaustivePruner;

impl Prune for ExhaustivePruner {
    fn possible(
        &self,
        timer: &Timer,
        problem: &Problem,
        support: &Support,
        e: &Exp,
    ) -> Result<bool, EarlyCutoff> {
        timer.tick()?;

        match e {
            Sketch::Hole(_) => Ok(true),
            Sketch::App(f, args) => {
                for arg in args.values() {
                    if !self.possible(timer, problem, support, arg)? {
                        return Ok(false);
                    }
                }

                let mut choices = IndexMap::new();
                let sig = problem.library.functions.get(&f.name).unwrap();
                for (fp, arg) in args {
                    timer.tick()?;
                    match arg {
                        Sketch::Hole(_) => choices.insert(
                            fp.clone(),
                            support.met_signature(
                                timer,
                                problem
                                    .library
                                    .types
                                    .get(sig.params.get(fp).unwrap())
                                    .unwrap(),
                            )?,
                        ),
                        Sketch::App(g, _) => {
                            choices.insert(fp.clone(), vec![g.metadata.clone()])
                        }
                    };
                }
                for arg_choice in util::cartesian_product(timer, choices)? {
                    timer.tick()?;
                    let ctx = eval::Context {
                        props: &problem.program.props,
                        args: &arg_choice,
                        ret: &f.metadata,
                    };

                    if ctx.sat(&sig.condition) {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
        }
    }
}

/// The main enumerative synthesis struct.
pub struct EnumerativeSynthesis<P: Prune> {
    problem: Problem,
    goal: Goal,
    pruner: P,
    support: Support,
}

impl<P: Prune> EnumerativeSynthesis<P> {
    /// Create a new enumerative synthesis instance
    pub fn new(mut problem: Problem, pruner: P) -> Self {
        let goal = Goal::new(&problem.program.goal);
        goal.add_to_library(&mut problem.library.functions);

        Self {
            support: Support::new(problem.vals()),
            problem,
            goal,
            pruner,
        }
    }

    fn support_hole(
        &self,
        timer: &Timer,
        typ: &Met<Value>,
    ) -> Result<Vec<ParameterizedFunction>, EarlyCutoff> {
        let mut funcs = vec![];
        for (g, gsig) in &self.problem.library.functions {
            if gsig.ret != typ.name {
                continue;
            }
            let gfunc = ParameterizedFunction::from_sig(
                gsig,
                g.clone(),
                typ.args.clone(),
            );
            let gfunc_app = Sketch::free(&Sketch::blank(), &gfunc);
            if !self.pruner.possible(
                timer,
                &self.problem,
                &self.support,
                &gfunc_app,
            )? {
                continue;
            }
            funcs.push(gfunc);
        }
        Ok(funcs)
    }

    fn support_fun(
        &self,
        timer: &Timer,
        f: &ParameterizedFunction,
        args: &IndexMap<FunParam, Exp>,
    ) -> Result<IndexMap<HoleName, Vec<ParameterizedFunction>>, EarlyCutoff>
    {
        let mut expansions = IndexMap::new();
        let fs = self.problem.library.functions.get(&f.name).unwrap();
        for (fp, mn) in &fs.params {
            timer.tick()?;
            match args.get(fp).unwrap() {
                Sketch::Hole(h) => {
                    let mut h_expansions = vec![];
                    let ms = self.problem.library.types.get(mn).unwrap();
                    for metadata in self.support.met_signature(timer, ms)? {
                        timer.tick()?;
                        h_expansions.extend(self.support_hole(
                            timer,
                            &Met {
                                name: mn.clone(),
                                args: metadata,
                            },
                        )?);
                    }
                    match expansions.insert(*h, h_expansions) {
                        Some(_) => panic!("Duplicate hole name {}", h),
                        None => (),
                    }
                }
                Sketch::App(g, g_args) => {
                    expansions.extend(self.support_fun(timer, g, g_args)?)
                }
            };
        }

        Ok(expansions)
    }

    fn wrap(
        &self,
        e: &Exp,
    ) -> (ParameterizedFunction, IndexMap<FunParam, Exp>) {
        match e {
            Sketch::Hole(_) => self.goal.app(e),
            Sketch::App(f, args) => (f.clone(), args.clone()),
        }
    }

    fn unwrap(&self, e: Exp) -> Exp {
        match e {
            Sketch::Hole(_) => panic!(),
            Sketch::App(f, mut args) => {
                if f.name == self.goal.function {
                    args.swap_remove(&self.goal.param).unwrap()
                } else {
                    Sketch::App(f, args)
                }
            }
        }
    }

    fn enumerate_worklist(
        &self,
        timer: &Timer,
        mut worklist: VecDeque<Exp>,
        max_solutions: usize,
    ) -> Result<Vec<Exp>, EarlyCutoff> {
        let type_context = typecheck::Context(&self.problem);
        let mut solutions = vec![];
        while let Some(e) = worklist.pop_front() {
            timer.tick()?;

            if e.size() > util::MAX_EXP_SIZE {
                return Err(EarlyCutoff::OutOfMemory);
            }

            let sup = match &e {
                Sketch::Hole(_) => panic!(),
                Sketch::App(f, args) => self.support_fun(timer, f, args)?,
            };

            if sup.is_empty() {
                if type_context.infer_exp(&e).is_ok() {
                    solutions.push(e);
                }
                if solutions.len() >= max_solutions {
                    break;
                }
                continue;
            }

            let sup_prod = util::cartesian_product(timer, sup)?;

            for choice in sup_prod {
                timer.tick()?;
                let mut new_e = e.clone();
                for (h, f) in choice {
                    let app = Sketch::free(&new_e, &f);
                    new_e = new_e.substitute(h, &app);
                }
                if !self.pruner.possible(
                    timer,
                    &self.problem,
                    &self.support,
                    &new_e,
                )? {
                    continue;
                }
                worklist.push_back(new_e)
            }
        }
        Ok(solutions)
    }

    fn enumerate(
        &self,
        timer: &Timer,
        start: &Exp,
        max_solutions: usize,
    ) -> Result<Vec<HoleFilling<ParameterizedFunction>>, EarlyCutoff> {
        let (f, args) = self.wrap(start);
        let worklist = VecDeque::from([Sketch::App(f, args)]);
        Ok(self
            .enumerate_worklist(timer, worklist, max_solutions)?
            .into_iter()
            .map(|e| start.pattern_match(&self.unwrap(e)).unwrap())
            .collect())
    }
}

impl<P: Prune> AnySynthesizer for EnumerativeSynthesis<P> {
    type F = ParameterizedFunction;

    fn provide_any(
        &mut self,
        timer: &Timer,
        start: &Exp,
    ) -> Result<Option<HoleFilling<ParameterizedFunction>>, EarlyCutoff> {
        Ok(self.enumerate(timer, start, 1)?.into_iter().next())
    }
}

impl<P: Prune> AllSynthesizer for EnumerativeSynthesis<P> {
    type F = ParameterizedFunction;

    fn provide_all(
        &mut self,
        timer: &Timer,
        start: &Exp,
    ) -> Result<Vec<HoleFilling<ParameterizedFunction>>, EarlyCutoff> {
        self.enumerate(timer, start, usize::MAX)
    }
}

// "Fully-constructive" oracle
impl<P: Prune> InhabitationOracle for EnumerativeSynthesis<P> {
    type F = ParameterizedFunction;

    fn expansions(
        &mut self,
        timer: &Timer,
        e: &Sketch<Self::F>,
    ) -> Result<Vec<(HoleName, Self::F)>, EarlyCutoff> {
        let (top_f, top_args) = self.wrap(e);
        let mut expansions = vec![];
        for (h, h_expansions) in self.support_fun(timer, &top_f, &top_args)? {
            for f in h_expansions {
                timer.tick()?;

                let app = Sketch::free(&e, &f);
                let new_e = e.substitute(h, &app);

                if !self.pruner.possible(
                    timer,
                    &self.problem,
                    &self.support,
                    &new_e,
                )? {
                    continue;
                }

                if self.provide_any(timer, &new_e)?.is_some() {
                    expansions.push((h, f))
                }
            }
        }
        Ok(expansions)
    }
}
