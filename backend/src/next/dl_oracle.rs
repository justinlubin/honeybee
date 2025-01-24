use crate::next::core::{self, *};
use crate::next::datalog::{self, *};
use crate::next::timer::*;
use crate::next::top_down::*;

use indexmap::{IndexMap, IndexSet};

////////////////////////////////////////////////////////////////////////////////
// Compilation to datalog

mod compile {
    use crate::next::core::{self, *};
    use crate::next::datalog::{self, *};
    use crate::next::top_down::*;

    use indexmap::IndexMap;

    pub fn value_type(vt: &core::ValueType) -> datalog::ValueType {
        match vt {
            core::ValueType::Int => datalog::ValueType::Int,
            core::ValueType::Str => datalog::ValueType::Str,
        }
    }

    pub fn value(v: &core::Value) -> datalog::Value {
        match v {
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
            )
            .unwrap();
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
            )
            .unwrap();
        }

        lib
    }

    pub fn header(_functions: &FunctionLibrary) -> Vec<(Rule, BaseFunction)> {
        todo!();
    }

    pub fn queries(
        _functions: &FunctionLibrary,
        _e: &Exp,
    ) -> Vec<(Rule, RelationSignature, usize, HoleName)> {
        todo!()
    }
}

////////////////////////////////////////////////////////////////////////////////
// Decompilation from datalog

mod decompile {
    use crate::next::core;
    use crate::next::datalog;

    pub fn value(v: &datalog::Value) -> core::Value {
        match v {
            datalog::Value::Int(x) => core::Value::Int(x.clone()),
            datalog::Value::Str(s) => core::Value::Str(s.clone()),
            datalog::Value::Var { .. } => panic!(),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// Goal convenience wrapper

struct Goal {
    function: BaseFunction,
    param: FunParam,
    ret: MetName,
    signature: FunctionSignature,
}

impl Goal {
    pub fn new(goal: &Met<core::Value>) -> Self {
        let function = BaseFunction("&goal".to_owned());
        let param = FunParam("&goalparam".to_owned());
        let ret = MetName("&Goal".to_owned());

        let signature = FunctionSignature {
            condition: Formula::conjunct(goal.args.iter().map(|(mp, v)| {
                Formula::Eq(
                    FormulaAtom::Param(param.clone(), mp.clone()),
                    FormulaAtom::Lit(v.clone()),
                )
            })),
            ret: ret.clone(),
            params: IndexMap::from([(param.clone(), goal.name.clone())]),
        };

        Self {
            function,
            param,
            ret,
            signature,
        }
    }

    pub fn add_to_library(&self, functions: &mut FunctionLibrary) {
        functions.insert(self.function.clone(), self.signature.clone());
    }

    pub fn app(&self, e: &Exp) -> Exp {
        Sketch::App(
            ParameterizedFunction::from_sig(
                &self.signature,
                self.function.clone(),
                IndexMap::new(),
            ),
            IndexMap::from([(self.param.clone(), e.clone())]),
        )
    }
}

////////////////////////////////////////////////////////////////////////////////
// Oracle

pub struct Oracle<Eng: Engine> {
    engine: Eng,
    problem: Problem,
    header: Vec<(Rule, BaseFunction)>,
    goal: Goal,
}

impl<Eng: Engine> Oracle<Eng> {
    pub fn new(
        mut engine: Eng,
        mut problem: Problem,
    ) -> Result<Self, datalog::Error> {
        let header = compile::header(&problem.library.functions);

        let datalog_program = {
            let lib = compile::signatures(
                &problem.library.props,
                &problem.library.types,
            );

            let mut dom = IndexSet::new();
            for fs in problem.library.functions.values() {
                dom.extend(fs.vals().iter().map(compile::value));
            }
            for p in &problem.program.props {
                dom.extend(p.args.values().map(compile::value));
            }
            dom.extend(problem.program.goal.args.values().map(compile::value));

            let rules = header.iter().map(|(r, _)| r.clone()).collect();

            let ground_facts =
                problem.program.props.iter().map(compile::fact).collect();

            datalog::Program::new(lib, dom, rules, ground_facts)?
        };

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

    fn expansions<E>(
        &mut self,
        e: &Sketch<Self::F>,
        timer: &impl Timer<E>,
    ) -> Result<Vec<(HoleName, Self::F)>, E> {
        let mut ret = vec![];

        for (query, query_sig, j, h) in
            compile::queries(&self.problem.library.functions, &self.goal.app(e))
        {
            for (rule, f) in &self.header {
                timer.tick()?;
                if let Some(cut_rule) = query.cut(rule, j) {
                    let f_sig = self.problem.library.functions.get(f).unwrap();
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
