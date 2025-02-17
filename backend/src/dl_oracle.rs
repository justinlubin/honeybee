use crate::core::{self, *};
use crate::datalog::{self, *};
use crate::top_down::*;
use crate::typecheck;
use crate::util::{Timer, TimerExpired};
use indexmap::IndexMap;

////////////////////////////////////////////////////////////////////////////////
// Compilation to datalog

struct CompileContext<'a>(typecheck::Context<'a>);

impl<'a> CompileContext<'a> {
    fn ret() -> FunParam {
        FunParam("&ret".to_owned())
    }

    fn value_type(&self, vt: &core::ValueType) -> datalog::ValueType {
        match vt {
            core::ValueType::Bool => datalog::ValueType::Bool,
            core::ValueType::Int => datalog::ValueType::Int,
            core::ValueType::Str => datalog::ValueType::Str,
        }
    }

    pub fn value(&self, v: &core::Value) -> datalog::Value {
        match v {
            core::Value::Bool(b) => datalog::Value::Bool(*b),
            core::Value::Int(x) => datalog::Value::Int(*x),
            core::Value::Str(s) => datalog::Value::Str(s.clone()),
        }
    }

    pub fn fact(&self, m: &Met<core::Value>) -> Fact {
        Fact {
            relation: Relation(m.name.0.clone()),
            args: m.args.values().map(|v| Some(self.value(v))).collect(),
        }
    }

    fn atomic_proposition(
        &self,
        fs: &FunctionSignature,
        ap: &AtomicProposition,
    ) -> Predicate {
        Predicate::Fact(Fact {
            relation: Relation(ap.name.0.clone()),
            args: ap
                .args
                .values()
                .map(|ofa| ofa.as_ref().map(|fa| self.formula_atom(fs, fa)))
                .collect(),
        })
    }

    fn var(
        &self,
        fp: &FunParam,
        mp: &MetParam,
        vt: &core::ValueType,
    ) -> datalog::Value {
        datalog::Value::Var {
            name: format!("{}*{}", fp.0, mp.0),
            typ: self.value_type(vt),
        }
    }

    fn free_fact(&self, fp: &FunParam, mn: &MetName) -> Fact {
        let sig = self.0 .0.library.types.get(mn).unwrap();
        Fact {
            relation: Relation(mn.0.clone()),
            args: sig
                .params
                .iter()
                .map(|(mp, vt)| Some(self.var(fp, mp, vt)))
                .collect(),
        }
    }

    fn formula_atom(
        &self,
        fs: &FunctionSignature,
        fa: &FormulaAtom,
    ) -> datalog::Value {
        let vt = self.0.infer_formula_atom(fs, fa).unwrap();
        match fa {
            FormulaAtom::Param(fp, mp) => self.var(fp, mp, &vt),
            FormulaAtom::Ret(mp) => self.var(&Self::ret(), mp, &vt),
            FormulaAtom::Lit(v) => self.value(v),
        }
    }

    fn formula(&self, fs: &FunctionSignature, f: &Formula) -> Vec<Predicate> {
        match f {
            Formula::True => vec![],
            Formula::Eq(left, right) => {
                vec![Predicate::PrimEq(
                    self.formula_atom(fs, left),
                    self.formula_atom(fs, right),
                )]
            }
            Formula::Lt(left, right) => {
                vec![Predicate::PrimLt(
                    self.formula_atom(fs, left),
                    self.formula_atom(fs, right),
                )]
            }
            Formula::Ap(ap) => {
                vec![self.atomic_proposition(fs, ap)]
            }
            Formula::And(f1, f2) => self
                .formula(fs, f1)
                .into_iter()
                .chain(self.formula(fs, f2))
                .collect(),
        }
    }

    pub fn signatures(&self) -> RelationLibrary {
        let mut lib = IndexMap::new();

        for (name, sig) in &self.0 .0.library.props {
            lib.insert(
                Relation(name.0.clone()),
                RelationSignature {
                    params: sig
                        .params
                        .values()
                        .map(|vt| self.value_type(vt))
                        .collect(),
                    kind: RelationKind::EDB,
                },
            );
        }

        for (name, sig) in &self.0 .0.library.types {
            lib.insert(
                Relation(name.0.clone()),
                RelationSignature {
                    params: sig
                        .params
                        .values()
                        .map(|vt| self.value_type(vt))
                        .collect(),
                    kind: RelationKind::IDB,
                },
            );
        }

        lib
    }

    pub fn header(&self) -> Vec<Rule> {
        self.0
             .0
            .library
            .functions
            .iter()
            .map(|(f, sig)| Rule {
                name: f.0.clone(),
                head: self.free_fact(&Self::ret(), &sig.ret),
                body: sig
                    .params
                    .iter()
                    .map(|(fp, mn)| Predicate::Fact(self.free_fact(fp, mn)))
                    .chain(self.formula(sig, &sig.condition))
                    .collect(),
            })
            .collect()
    }

    pub fn queries(
        &self,
        f: &ParameterizedFunction,
        args: &IndexMap<FunParam, Exp>,
    ) -> Vec<(Rule, RelationSignature, usize, HoleName)> {
        let mut facts = vec![];
        let mut prims = vec![];
        let mut heads = vec![];
        let mut rec_calls = vec![];

        let fs = self.0 .0.library.functions.get(&f.name).unwrap();

        for (j, (fp, e)) in args.iter().enumerate() {
            let mn = fs.params.get(fp).unwrap();
            facts.push(self.free_fact(fp, mn));
            match e {
                Sketch::App(g, g_args) => {
                    for (mp, v) in &g.metadata {
                        prims.push(Predicate::PrimEq(
                            self.var(fp, mp, &self.0.infer_value(&v)),
                            self.value(v),
                        ));
                    }
                    rec_calls.extend(self.queries(g, g_args));
                }
                Sketch::Hole(h) => {
                    heads.push((fp.clone(), mn.clone(), j, *h));
                }
            }
        }

        prims.extend(f.metadata.iter().map(|(mp, v)| {
            Predicate::PrimEq(
                self.var(&Self::ret(), mp, &self.0.infer_value(v)),
                self.value(v),
            )
        }));

        prims.extend(self.formula(fs, &fs.condition));

        heads
            .into_iter()
            .map(|(fp, mn, j, h)| {
                let mut head = self.free_fact(&fp, &mn);
                head.relation = Relation(format!("&Query_{}_{}", j, h));
                (
                    Rule {
                        name: format!("&query_{}_{}", j, h),
                        head,
                        body: facts
                            .clone()
                            .into_iter()
                            .map(Predicate::Fact)
                            .chain(prims.clone())
                            .collect(),
                    },
                    RelationSignature {
                        params: self
                            .0
                             .0
                            .library
                            .types
                            .get(&mn)
                            .unwrap()
                            .params
                            .values()
                            .map(|vt| self.value_type(vt))
                            .collect(),
                        kind: RelationKind::IDB,
                    },
                    j,
                    h,
                )
            })
            .chain(rec_calls)
            .collect()
    }
}

////////////////////////////////////////////////////////////////////////////////
// Decompilation from datalog

mod decompile {
    use crate::core;
    use crate::datalog;

    pub fn value(v: &datalog::Value) -> core::Value {
        match v {
            datalog::Value::Bool(b) => core::Value::Bool(*b),
            datalog::Value::Int(x) => core::Value::Int(*x),
            datalog::Value::Str(s) => core::Value::Str(s.clone()),
            datalog::Value::Var { .. } => panic!(),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// Oracle

pub struct Oracle<Eng: Engine> {
    engine: Eng,
    problem: Problem,
    header: Vec<Rule>,
    goal: Goal,
}

impl<Eng: Engine> Oracle<Eng> {
    pub fn new(
        mut engine: Eng,
        mut problem: Problem,
    ) -> Result<Self, datalog::Error> {
        let compile = CompileContext(typecheck::Context(&problem));

        let header = compile.header();

        let datalog_program = datalog::Program::new(
            compile.signatures(),
            problem.vals().iter().map(|v| compile.value(v)).collect(),
            header.clone(),
            problem
                .program
                .props
                .iter()
                .map(|f| compile.fact(f))
                .collect(),
        )?;

        engine.load(datalog_program);

        let goal = Goal::new(&problem.program.goal);
        goal.add_to_library(&mut problem.library.functions);

        Ok(Self {
            engine,
            problem,
            header,
            goal,
        })
    }
}

impl<Eng: Engine> InhabitationOracle for Oracle<Eng> {
    type F = ParameterizedFunction;

    fn expansions(
        &mut self,
        timer: &Timer,
        e: &Sketch<Self::F>,
    ) -> Result<Vec<Expansion<Self::F>>, TimerExpired> {
        let compile = CompileContext(typecheck::Context(&self.problem));

        let mut ret = vec![];

        let (goal_pf, goal_args) = self.goal.app(e);

        let queries = compile.queries(&goal_pf, &goal_args);

        log::debug!(
            "Found {} queries: {}",
            queries.len(),
            queries
                .iter()
                .map(|(_, _, j, h)| format!("(j={}, h={})", j, h))
                .collect::<Vec<_>>()
                .join(", ")
        );

        for (query, query_sig, j, h) in queries {
            log::debug!("Trying query with (j={j:}, h={h:}):\n{query:#?}");
            for rule in &self.header {
                log::debug!("Trying header rule '{}'", rule.name);
                timer.tick()?;
                if let Some(cut_rule) = query.cut(rule, j) {
                    log::debug!("Header rule '{}' matches", rule.name);
                    let f = BaseFunction(rule.name.clone());
                    let f_sig = self.problem.library.functions.get(&f).unwrap();
                    let f_ret_sig =
                        self.problem.library.types.get(&f_sig.ret).unwrap();

                    ret.extend(
                        self.engine
                            .query(&query_sig, &cut_rule)
                            .into_iter()
                            .map(|vals| {
                                (
                                    h,
                                    ParameterizedFunction::from_sig(
                                        f_sig,
                                        f.clone(),
                                        f_ret_sig
                                            .params
                                            .keys()
                                            .cloned()
                                            .zip(
                                                vals.iter()
                                                    .map(decompile::value),
                                            )
                                            .collect(),
                                    ),
                                )
                            }),
                    )
                }
            }
        }

        Ok(ret)
    }
}
