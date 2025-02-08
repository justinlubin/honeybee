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
            args: m.args.values().map(|v| value(v)).collect(),
        }
    }

    fn atomic_proposition(
        lib: &MetLibrary,
        fs: &FunctionSignature,
        ap: &Met<core::FormulaAtom>,
    ) -> Predicate {
        Predicate::Fact(Fact {
            relation: Relation(ap.name.0.clone()),
            args: ap
                .args
                .values()
                .map(|fa| formula_atom(lib, fs, fa))
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
            args: sig.params.iter().map(|(mp, vt)| var(fp, mp, vt)).collect(),
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
            Formula::AtomicProposition(ap) => {
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
        let mut body: Vec<Predicate> = vec![];
        let mut heads: Vec<(FunParam, MetName, usize, HoleName)> = vec![];
        let mut rec_calls: Vec<(Rule, RelationSignature, usize, HoleName)> =
            vec![];
        let fs = lib.functions.get(&f.name).unwrap();

        let mut idx = 0;
        for (fp, e) in args {
            let mn = fs.params.get(fp).unwrap();
            match e {
                Sketch::App(g, g_args) => {
                    for (mp, v) in &g.metadata {
                        body.push(Predicate::PrimEq(
                            var(fp, mp, &v.infer()),
                            value(v),
                        ));
                        rec_calls.extend(queries(lib, g, g_args));
                        idx += 1;
                    }
                }
                Sketch::Hole(h) => {
                    body.push(Predicate::Fact(free_fact(&lib.types, fp, mn)));
                    heads.push((fp.clone(), mn.clone(), idx, *h));
                    idx += 1;
                }
            }
        }

        body.extend(f.metadata.iter().map(|(mp, v)| {
            Predicate::PrimEq(var(&ret(), mp, &v.infer()), value(v))
        }));

        body.extend(formula(&lib.types, &fs, &fs.condition));

        heads
            .into_iter()
            .map(|(fp, mn, j, h)| {
                let mut head = free_fact(&lib.types, &fp, &mn);
                head.relation = Relation(format!("&Query_{}_{}", j, h));
                (
                    Rule {
                        name: format!("&query_{}_{}", j, h),
                        head,
                        body: body.clone(),
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
        for (query, query_sig, j, h) in
            compile::queries(&self.problem.library, &goal_pf, &goal_args)
        {
            for rule in &self.header {
                timer.tick()?;
                if let Some(cut_rule) = query.cut(rule, j) {
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
