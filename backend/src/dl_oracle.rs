use crate::core::*;
use crate::datalog::{self, *};
use crate::top_down::*;
use crate::util::Timer;

////////////////////////////////////////////////////////////////////////////////
// Compilation to datalog

mod compile {
    use crate::core::{self, *};
    use crate::datalog::{self, *};
    use crate::top_down::*;

    use indexmap::IndexMap;

    fn ret() -> FunParam {
        FunParam("&ret".to_owned())
    }

    fn value_type(vt: &core::ValueType) -> datalog::ValueType {
        match vt {
            core::ValueType::Bool => datalog::ValueType::Bool,
            core::ValueType::Int => datalog::ValueType::Int,
            core::ValueType::Str => datalog::ValueType::Str,
        }
    }

    pub fn value(v: &core::Value) -> datalog::Value {
        match v {
            core::Value::Bool(b) => datalog::Value::Bool(b.clone()),
            core::Value::Int(x) => datalog::Value::Int(x.clone()),
            core::Value::Str(s) => datalog::Value::Str(s.clone()),
        }
    }

    pub fn fact(m: &Met<core::Value>) -> Fact {
        Fact {
            relation: Relation(m.name.0.clone()),
            args: m.args.values().map(|v| Some(value(v))).collect(),
        }
    }

    fn atomic_proposition(
        lib: &MetLibrary,
        fs: &FunctionSignature,
        ap: &AtomicProposition,
    ) -> Predicate {
        Predicate::Fact(Fact {
            relation: Relation(ap.name.0.clone()),
            args: ap
                .args
                .values()
                .map(|ofa| ofa.as_ref().map(|fa| formula_atom(lib, fs, fa)))
                .collect(),
        })
    }

    fn var(
        fp: &FunParam,
        mp: &MetParam,
        vt: &core::ValueType,
    ) -> datalog::Value {
        datalog::Value::Var {
            name: format!("{}*{}", fp.0, mp.0),
            typ: value_type(vt),
        }
    }

    fn free_fact(lib: &MetLibrary, fp: &FunParam, mn: &MetName) -> Fact {
        let sig = lib.get(mn).unwrap();
        Fact {
            relation: Relation(mn.0.clone()),
            args: sig
                .params
                .iter()
                .map(|(mp, vt)| Some(var(fp, mp, vt)))
                .collect(),
        }
    }

    fn formula_atom(
        lib: &MetLibrary,
        fs: &FunctionSignature,
        fa: &FormulaAtom,
    ) -> datalog::Value {
        let vt = fa.infer(lib, fs).unwrap();
        match fa {
            FormulaAtom::Param(fp, mp) => var(fp, mp, &vt),
            FormulaAtom::Ret(mp) => var(&ret(), mp, &vt),
            FormulaAtom::Lit(v) => value(v),
        }
    }

    fn formula(
        lib: &MetLibrary,
        fs: &FunctionSignature,
        f: &Formula,
    ) -> Vec<Predicate> {
        match f {
            Formula::True => vec![],
            Formula::Eq(left, right) => {
                vec![Predicate::PrimEq(
                    formula_atom(lib, fs, left),
                    formula_atom(lib, fs, right),
                )]
            }
            Formula::Lt(left, right) => {
                vec![Predicate::PrimLt(
                    formula_atom(lib, fs, left),
                    formula_atom(lib, fs, right),
                )]
            }
            Formula::Ap(ap) => {
                vec![atomic_proposition(lib, fs, ap)]
            }
            Formula::And(f1, f2) => formula(lib, fs, f1)
                .into_iter()
                .chain(formula(lib, fs, f2))
                .collect(),
        }
    }

    pub fn signatures(
        props: &MetLibrary,
        types: &MetLibrary,
    ) -> RelationLibrary {
        let mut lib = IndexMap::new();

        for (name, sig) in props {
            lib.insert(
                Relation(name.0.clone()),
                RelationSignature {
                    params: sig
                        .params
                        .values()
                        .map(|vt| value_type(vt))
                        .collect(),
                    kind: RelationKind::EDB,
                },
            );
        }

        for (name, sig) in types {
            lib.insert(
                Relation(name.0.clone()),
                RelationSignature {
                    params: sig
                        .params
                        .values()
                        .map(|vt| value_type(vt))
                        .collect(),
                    kind: RelationKind::IDB,
                },
            );
        }

        lib
    }

    pub fn header(lib: &Library) -> Vec<Rule> {
        lib.functions
            .iter()
            .map(|(f, sig)| Rule {
                name: f.0.clone(),
                head: free_fact(&lib.types, &ret(), &sig.ret),
                body: sig
                    .params
                    .iter()
                    .map(|(fp, mn)| {
                        Predicate::Fact(free_fact(&lib.types, fp, mn))
                    })
                    .into_iter()
                    .chain(formula(&lib.types, &sig, &sig.condition))
                    .collect(),
            })
            .collect()
    }

    pub fn queries(
        lib: &Library,
        f: &ParameterizedFunction,
        args: &IndexMap<FunParam, Exp>,
    ) -> Vec<(Rule, RelationSignature, usize, HoleName)> {
        let mut facts = vec![];
        let mut prims = vec![];
        let mut heads = vec![];
        let mut rec_calls = vec![];

        let fs = lib.functions.get(&f.name).unwrap();

        for (j, (fp, e)) in args.iter().enumerate() {
            let mn = fs.params.get(fp).unwrap();
            facts.push(free_fact(&lib.types, fp, mn));
            match e {
                Sketch::App(g, g_args) => {
                    for (mp, v) in &g.metadata {
                        prims.push(Predicate::PrimEq(
                            var(fp, mp, &v.infer()),
                            value(v),
                        ));
                    }
                    rec_calls.extend(queries(lib, g, g_args));
                }
                Sketch::Hole(h) => {
                    heads.push((fp.clone(), mn.clone(), j, *h));
                }
            }
        }

        prims.extend(f.metadata.iter().map(|(mp, v)| {
            Predicate::PrimEq(var(&ret(), mp, &v.infer()), value(v))
        }));

        prims.extend(formula(&lib.types, &fs, &fs.condition));

        heads
            .into_iter()
            .map(|(fp, mn, j, h)| {
                let mut head = free_fact(&lib.types, &fp, &mn);
                head.relation = Relation(format!("&Query_{}_{}", j, h));
                (
                    Rule {
                        name: format!("&query_{}_{}", j, h),
                        head,
                        body: facts
                            .clone()
                            .into_iter()
                            .map(|f| Predicate::Fact(f))
                            .chain(prims.clone())
                            .collect(),
                    },
                    RelationSignature {
                        params: lib
                            .types
                            .get(&mn)
                            .unwrap()
                            .params
                            .values()
                            .map(|vt| value_type(vt))
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
            datalog::Value::Bool(b) => core::Value::Bool(b.clone()),
            datalog::Value::Int(x) => core::Value::Int(x.clone()),
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
        let header = compile::header(&problem.library);

        let datalog_program = datalog::Program::new(
            compile::signatures(&problem.library.props, &problem.library.types),
            problem.vals().iter().map(compile::value).collect(),
            header.clone(),
            problem.program.props.iter().map(compile::fact).collect(),
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

    fn expansions<T: Timer>(
        &mut self,
        e: &Sketch<Self::F>,
        timer: &T,
    ) -> Result<Vec<Expansion<Self::F>>, T::Expired> {
        let mut ret = vec![];

        let (goal_pf, goal_args) = self.goal.app(e);

        let queries =
            compile::queries(&self.problem.library, &goal_pf, &goal_args);

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
                                    h.clone(),
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
