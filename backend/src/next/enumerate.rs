use crate::next::core::*;
use crate::next::top_down::*;
use crate::next::traditional_synthesis::*;
use crate::next::util::{self, Timer};

use indexmap::{IndexMap, IndexSet};
use std::collections::VecDeque;

struct Support {
    ints: Vec<Value>,
    strings: Vec<Value>,
}

impl Support {
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

    fn met_signature<T: Timer>(
        &self,
        timer: &T,
        sig: &MetSignature,
    ) -> Result<Vec<IndexMap<MetParam, Value>>, T::Expired> {
        let choices = sig
            .params
            .iter()
            .map(|(k, vt)| (k.clone(), self.value_type(vt)))
            .collect();
        util::cartesian_product(timer, choices)
    }
}

trait Prune {
    fn possible<T: Timer>(
        &self,
        timer: &T,
        problem: &Problem,
        support: &Support,
        e: &Exp,
    ) -> Result<bool, T::Expired>;
}

struct NaivePruner {}

impl Prune for NaivePruner {
    fn possible<T: Timer>(
        &self,
        _: &T,
        _: &Problem,
        _: &Support,
        _: &Exp,
    ) -> Result<bool, T::Expired> {
        Ok(true)
    }
}

struct ExhaustivePruner {}

impl Prune for ExhaustivePruner {
    fn possible<T: Timer>(
        &self,
        timer: &T,
        problem: &Problem,
        support: &Support,
        e: &Exp,
    ) -> Result<bool, T::Expired> {
        match e {
            Sketch::Hole(_) => Ok(true),
            Sketch::App(f, args) => {
                let mut choices = IndexMap::new();
                let sig = problem.library.functions.get(&f.name).unwrap();
                for (fp, arg) in args {
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
                    if sig
                        .condition
                        .sat(
                            &problem.program.props,
                            &EvaluationContext {
                                args: &arg_choice,
                                ret: &f.metadata,
                            },
                        )
                        .unwrap()
                    {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
        }
    }
}

struct EnumerativeSynthesis<P: Prune> {
    problem: Problem,
    goal: Goal,
    pruner: P,
    support: Support,
}

impl<P: Prune> EnumerativeSynthesis<P> {
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

    fn support_hole(&self, typ: &Met<Value>) -> Vec<ParameterizedFunction> {
        let mut funcs = vec![];
        for (g, gsig) in &self.problem.library.functions {
            if gsig.ret != typ.name {
                continue;
            }
            funcs.push(ParameterizedFunction::from_sig(
                gsig,
                g.clone(),
                typ.args.clone(),
            ));
        }
        funcs
    }

    fn support_fun<T: Timer>(
        &self,
        timer: &T,
        f: &ParameterizedFunction,
        args: &IndexMap<FunParam, Exp>,
    ) -> Result<Vec<(HoleName, ParameterizedFunction, Exp)>, T::Expired> {
        let mut expansions = vec![];
        let fs = self.problem.library.functions.get(&f.name).unwrap();
        for (fp, mn) in &fs.params {
            match args.get(fp).unwrap() {
                Sketch::Hole(h) => {
                    let ms = self.problem.library.types.get(mn).unwrap();
                    for metadata in self.support.met_signature(timer, ms)? {
                        expansions.extend(
                            self.support_hole(&Met {
                                name: mn.clone(),
                                args: metadata,
                            })
                            .into_iter()
                            .map(|g| {
                                (*h, g, Sketch::App(f.clone(), args.clone()))
                            }),
                        );
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

    // TODO: add memoization? (seen list)
    fn enumerate_worklist<T: Timer>(
        &self,
        timer: &T,
        mut worklist: VecDeque<Exp>,
        max_solutions: usize,
    ) -> Result<Vec<Exp>, T::Expired> {
        let mut solutions = vec![];
        while let Some(e) = worklist.pop_front() {
            let sup = match &e {
                Sketch::Hole(_) => panic!(),
                Sketch::App(f, args) => self.support_fun(timer, f, args)?,
            };
            if sup.is_empty() {
                solutions.push(e);
                if solutions.len() >= max_solutions {
                    break;
                }
                continue;
            }
            for (h, f, parent) in sup {
                let app = Sketch::free(&e, &f);
                let new_e = e.substitute(h, &app);
                if !self.pruner.possible(
                    timer,
                    &self.problem,
                    &self.support,
                    &parent.substitute(h, &app),
                )? {
                    continue;
                }
                worklist.push_back(new_e)
            }
        }
        Ok(solutions)
    }

    fn enumerate<T: Timer>(
        &self,
        timer: &T,
        start: &Exp,
        max_solutions: usize,
    ) -> Result<Vec<HoleFilling<ParameterizedFunction>>, T::Expired> {
        let (f, args) = self.wrap(start);
        let worklist = VecDeque::from([Sketch::App(f, args)]);
        Ok(self
            .enumerate_worklist(timer, worklist, max_solutions)?
            .iter()
            .map(|e| start.pattern_match(e).unwrap())
            .collect())
    }
}

impl<P: Prune> AnySynthesizer for EnumerativeSynthesis<P> {
    type F = ParameterizedFunction;

    fn provide_any<T: Timer>(
        &self,
        start: &Exp,
        timer: &T,
    ) -> Result<Option<HoleFilling<ParameterizedFunction>>, T::Expired> {
        Ok(self.enumerate(timer, start, 1)?.into_iter().next())
    }
}

impl<P: Prune> AllSynthesizer for EnumerativeSynthesis<P> {
    type F = ParameterizedFunction;

    fn provide_all<T: Timer>(
        &self,
        start: &Exp,
        timer: &T,
    ) -> Result<Vec<HoleFilling<ParameterizedFunction>>, T::Expired> {
        self.enumerate(timer, start, usize::MAX)
    }
}

// Constructive oracle
impl<P: Prune> InhabitationOracle for EnumerativeSynthesis<P> {
    type F = ParameterizedFunction;

    fn expansions<T: Timer>(
        &mut self,
        e: &Sketch<Self::F>,
        timer: &T,
    ) -> Result<Vec<(HoleName, Self::F)>, T::Expired> {
        let (top_f, top_args) = self.wrap(e);
        let mut expansions = vec![];
        for (h, f, parent) in self.support_fun(timer, &top_f, &top_args)? {
            let app = Sketch::free(&parent, &f);
            let new_e = e.substitute(h, &app);
            if !self.pruner.possible(
                timer,
                &self.problem,
                &self.support,
                &parent.substitute(h, &app),
            )? {
                continue;
            }
            match self.provide_any(&new_e, timer)? {
                Some(_) => expansions.push((h, f)),
                None => (),
            }
        }
        Ok(expansions)
    }
}
