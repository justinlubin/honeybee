use crate::next::core::*;
use crate::next::datalog::*;
use crate::next::timer::*;
use crate::next::top_down::*;

use indexmap::IndexMap;

fn value_type(vt: &super::core::ValueType) -> super::datalog::ValueType {
    match vt {
        super::core::ValueType::Int => super::datalog::ValueType::Int,
        super::core::ValueType::Str => super::datalog::ValueType::Str,
    }
}

fn value(v: &super::core::Value) -> super::datalog::Value {
    match v {
        super::core::Value::Int(x) => super::datalog::Value::Int(x.clone()),
        super::core::Value::Str(s) => super::datalog::Value::Str(s.clone()),
    }
}

fn fact(m: &Met<crate::next::core::Value>) -> Fact {
    Fact {
        relation: Relation(m.name.0.clone()),
        args: m.args.values().map(|v| value(v)).collect(),
    }
}

fn signatures(props: &MetLibrary, types: &MetLibrary) -> RelationLibrary {
    let mut lib = IndexMap::new();

    for (name, sig) in props {
        lib.insert(
            Relation(name.0.clone()),
            RelationSignature {
                params: sig.params.values().map(|vt| value_type(vt)).collect(),
                kind: RelationKind::EDB,
            },
        )
        .unwrap();
    }

    for (name, sig) in types {
        lib.insert(
            Relation(name.0.clone()),
            RelationSignature {
                params: sig.params.values().map(|vt| value_type(vt)).collect(),
                kind: RelationKind::IDB,
            },
        )
        .unwrap();
    }

    lib
}

fn header(functions: &FunctionLibrary) -> Vec<Rule> {
    todo!();
}

fn add_goal(functions: &mut FunctionLibrary, goal: &Met<super::core::Value>) {
    let goal_function = BaseFunction("&goal".to_owned());
    let goal_param = FunParam("&goalparam".to_owned());
    let goal_name = MetName("&Goal".to_owned());

    functions.insert(
        goal_function,
        FunctionSignature {
            condition: Formula::conjunct(goal.args.iter().map(|(mp, v)| {
                Formula::Eq(
                    FormulaAtom::Param(goal_param.clone(), mp.clone()),
                    FormulaAtom::Lit(v.clone()),
                )
            })),
            ret: goal_name,
            params: IndexMap::from([(goal_param, goal.name.clone())]),
        },
    );
}

// fn remove_goal(functions: &mut FunctionLibrary) {
//     let _ = functions
//         .shift_remove(&BaseFunction("&goal".to_owned()))
//         .unwrap();
// }

pub struct Oracle<Eng: Engine> {
    engine: Eng,
    problem: Problem,
    header_rules: Vec<Rule>,
}

impl<Eng: Engine> Oracle<Eng> {
    pub fn new(
        engine: Eng,
        mut problem: Problem,
    ) -> Result<Self, super::datalog::Error> {
        let datalog_program =
            super::datalog::Program::new(lib, dom, header_rules, ground_facts)?;

        engine.load(datalog_program);

        add_goal(&mut problem.library.functions, &problem.program.goal);

        Ok(Self {
            engine,
            problem,
            header_rules,
        })
    }
}

impl<Eng: Engine> InhabitationOracle for Oracle<Eng> {
    type F = ParameterizedFunction;

    fn expansions<E>(
        &mut self,
        e: &Sketch<Self::F>,
        timer: &impl Timer<E>,
    ) -> Vec<(HoleName, Self::F)> {
        for (query, sig, j, h) in
            queries(self.problem.library.functions, todo!())
        {
            for (rule, f) in self.header_rules {
                if let Some(cut_rule) = query.cut(rule, j) {
                    let f_sig = self.problem.library.functions.get(&f).unwrap();
                    self.engine
                        .query(sig, cut_rule)
                        .into_iter()
                        .map(|vals| {
                            (
                                h.clone(),
                                ParameterizedFunction::from_sig(
                                    f_sig
                                    f,
                                    f_sig.params,
                                ),
                            )
                        })
                        .collect()
                }
            }
        }
    }
}
