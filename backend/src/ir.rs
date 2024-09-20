use std::collections::HashMap;
use std::collections::HashSet;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ValueType {
    Int,
    Str,
    List(Box<ValueType>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Value {
    Int(i64),
    Str(String),
    List(Vec<Value>),
}

impl Value {
    pub fn infer(&self) -> ValueType {
        match self {
            Value::Int(_) => ValueType::Int,
            Value::Str(_) => ValueType::Str,
            Value::List(_) => todo!(),
        }
    }
}

pub type Assignment = HashMap<String, Value>;

pub type FactName = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FactKind {
    Input,
    Output,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactSignature {
    pub name: FactName,
    pub kind: FactKind,
    pub params: Vec<(String, ValueType)>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Fact {
    pub name: FactName,
    pub args: Vec<(String, Value)>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PredicateAtom {
    Select { selector: String, arg: String },
    Const(Value),
}

impl PredicateAtom {
    pub fn typ(
        &self,
        lib: &Library,
        params: &Vec<(String, FactName, Mode)>,
    ) -> Option<ValueType> {
        match self {
            PredicateAtom::Select { selector, arg } => params
                .iter()
                .find_map(|(x, f, _)| {
                    if x == arg {
                        lib.fact_signature(f)
                    } else {
                        None
                    }
                })
                .and_then(|fs| {
                    fs.params.iter().find_map(|(x, vt)| {
                        if x == selector {
                            Some(vt.clone())
                        } else {
                            None
                        }
                    })
                }),
            PredicateAtom::Const(_) => None,
        }
    }

    pub fn free_variables(&self) -> HashSet<String> {
        match self {
            PredicateAtom::Select { arg, .. } => {
                let mut hs: HashSet<String> = HashSet::new();
                hs.insert(arg.clone());
                hs
            }
            PredicateAtom::Const(_) => HashSet::new(),
        }
    }

    fn prefix_vars(&self, prefix: &str) -> PredicateAtom {
        match self {
            PredicateAtom::Select { selector, arg } => PredicateAtom::Select {
                selector: selector.clone(),
                arg: format!("{}{}", prefix, arg),
            },
            PredicateAtom::Const(v) => PredicateAtom::Const(v.clone()),
        }
    }

    pub fn substitute(
        &self,
        selector: &str,
        arg: &str,
        rhs: &Value,
    ) -> PredicateAtom {
        match self {
            PredicateAtom::Select {
                selector: s,
                arg: a,
            } => {
                if s == selector && a == arg {
                    PredicateAtom::Const(rhs.clone())
                } else {
                    self.clone()
                }
            }
            PredicateAtom::Const(v) => PredicateAtom::Const(v.clone()),
        }
    }

    // pub fn substitute_all(
    //     &self,
    //     subs: Vec<(&str, &str, &Value)>,
    // ) -> PredicateAtom {
    //     let mut ret = self.clone();
    //     for (selector, arg, rhs) in subs {
    //         ret = ret.substitute(selector, arg, rhs);
    //     }
    //     ret
    // }

    pub fn eval(&self, ret: &Fact, args: &Vec<(&String, &Fact)>) -> Value {
        // println!("{:?}\n{:?}\n{:?}", self, ret, args);
        match self {
            PredicateAtom::Select { arg, selector } => {
                let fact = if arg == Query::RET {
                    ret
                } else {
                    args.iter()
                        .find_map(
                            |(k, f)| {
                                if **k == *arg {
                                    Some(f)
                                } else {
                                    None
                                }
                            },
                        )
                        .expect("arg should match")
                };

                fact.args
                    .iter()
                    .find_map(
                        |(k, v)| if *k == *selector { Some(v) } else { None },
                    )
                    .expect("selector should match")
                    .clone()
            }
            PredicateAtom::Const(v) => v.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PredicateRelationBinOp {
    Eq,
    Lt,
    Lte,
    Contains,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PredicateRelation {
    BinOp(PredicateRelationBinOp, PredicateAtom, PredicateAtom),
}

impl PredicateRelation {
    pub fn prefix_vars(&self, prefix: &str) -> PredicateRelation {
        match self {
            PredicateRelation::BinOp(op, lhs, rhs) => PredicateRelation::BinOp(
                op.clone(),
                lhs.prefix_vars(prefix),
                rhs.prefix_vars(prefix),
            ),
        }
    }

    pub fn substitute(
        &self,
        selector: &str,
        arg: &str,
        rhs: &Value,
    ) -> PredicateRelation {
        match self {
            PredicateRelation::BinOp(op, left, right) => {
                PredicateRelation::BinOp(
                    op.clone(),
                    left.substitute(selector, arg, rhs),
                    right.substitute(selector, arg, rhs),
                )
            }
        }
    }

    pub fn substitute_all(
        &self,
        subs: Vec<(&str, &str, &Value)>,
    ) -> PredicateRelation {
        let mut ret = self.clone();
        for (selector, arg, rhs) in subs {
            ret = ret.substitute(selector, arg, rhs);
        }
        ret
    }

    pub fn free_variables(&self) -> HashSet<String> {
        match self {
            PredicateRelation::BinOp(_, left, right) => left
                .free_variables()
                .union(&right.free_variables())
                .cloned()
                .collect(),
        }
    }

    pub fn sat(&self, ret: &Fact, args: &Vec<(&String, &Fact)>) -> bool {
        match self {
            PredicateRelation::BinOp(op, a1, a2) => {
                match (op, a1.eval(ret, args), a2.eval(ret, args)) {
                    (PredicateRelationBinOp::Eq, v1, v2) => v1 == v2,
                    (
                        PredicateRelationBinOp::Lt,
                        Value::Int(i1),
                        Value::Int(i2),
                    ) => i1 < i2,
                    (
                        PredicateRelationBinOp::Lte,
                        Value::Int(i1),
                        Value::Int(i2),
                    ) => i1 <= i2,
                    (PredicateRelationBinOp::Contains, _, _) => todo!(),
                    (_, _, _) => false,
                }
            }
        }
    }
}

pub type Predicate = Vec<PredicateRelation>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Mode {
    Exists,
    ForAll,
    ForAllPlus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputationSignature {
    pub name: String,
    pub params: Vec<(String, FactName, Mode)>,
    pub ret: FactName,
    pub precondition: Predicate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Library {
    pub fact_signatures: Vec<FactSignature>,
    pub computation_signatures: Vec<ComputationSignature>,
}

#[derive(Debug, Clone)]
pub struct Program {
    pub annotations: Vec<Fact>,
    pub goal: Fact,
}

// Invariants:
// - The computation signature's return value must be the same as the fact
//   signature's name
// - All mode parameters for the computation signature must be Mode::Exists
#[derive(Debug, Clone)]
pub struct Query {
    pub fact_signature: FactSignature,
    pub computation_signature: ComputationSignature,
}

impl Query {
    pub const RET: &'static str = "ret";

    pub const GOAL_FACT_NAME: &'static str = "&GOAL";
    pub const GOAL_COMPUTATION_NAME: &'static str = "&goal";

    pub fn from_fact(fact: &Fact, tag: &str) -> Query {
        Query {
            fact_signature: FactSignature {
                name: Query::GOAL_FACT_NAME.to_owned(),
                kind: FactKind::Output,
                params: vec![],
            },
            computation_signature: ComputationSignature {
                name: Query::GOAL_COMPUTATION_NAME.to_owned(),
                params: vec![(tag.to_owned(), fact.name.clone(), Mode::Exists)],
                ret: Query::GOAL_FACT_NAME.to_owned(),
                precondition: fact
                    .args
                    .iter()
                    .map(|(n, v)| {
                        PredicateRelation::BinOp(
                            PredicateRelationBinOp::Eq,
                            PredicateAtom::Select {
                                selector: n.clone(),
                                arg: tag.to_owned(),
                            },
                            PredicateAtom::Const(v.clone()),
                        )
                    })
                    .collect(),
            },
        }
    }

    pub fn free(
        lib: &Library,
        params: Vec<(String, FactName)>,
        precondition: Predicate,
    ) -> Query {
        let mut fs_params = vec![];
        let mut cs_precondition = precondition.clone();
        for (n, f) in &params {
            for (nn, vt) in &lib.fact_signature(f).unwrap().params {
                let fv = format!("fv%{}*{}", n, nn);
                fs_params.push((fv.clone(), vt.clone()));
                cs_precondition.push(PredicateRelation::BinOp(
                    PredicateRelationBinOp::Eq,
                    PredicateAtom::Select {
                        selector: fv,
                        arg: Query::RET.to_owned(),
                    },
                    PredicateAtom::Select {
                        selector: nn.clone(),
                        arg: n.clone(),
                    },
                ));
            }
        }
        Query {
            fact_signature: FactSignature {
                name: Query::GOAL_FACT_NAME.to_owned(),
                kind: FactKind::Output,
                params: fs_params,
            },
            computation_signature: ComputationSignature {
                name: Query::GOAL_COMPUTATION_NAME.to_owned(),
                params: params
                    .into_iter()
                    .map(|(x, f)| (x, f, Mode::Exists))
                    .collect(),
                ret: Query::GOAL_FACT_NAME.to_owned(),
                precondition: cs_precondition,
            },
        }
    }

    pub fn closed(&self) -> bool {
        self.fact_signature.params.is_empty()
    }

    pub fn cut(
        &self,
        lib: &Library,
        cut_param: &str,
        lemma: &ComputationSignature,
    ) -> Query {
        Query {
            fact_signature: self.fact_signature.clone(),
            computation_signature: self
                .computation_signature
                .cut(lib, cut_param, lemma),
        }
    }
}

impl Library {
    pub fn fact_signature(&self, fact_name: &str) -> Option<&FactSignature> {
        self.fact_signatures.iter().find(|fs| fs.name == *fact_name)
    }

    pub fn computation_signature(
        &self,
        computation_name: &str,
    ) -> Option<&ComputationSignature> {
        self.computation_signatures
            .iter()
            .find(|cs| cs.name == *computation_name)
    }

    pub fn matching_computation_signatures(
        &self,
        fact_name: &str,
    ) -> Vec<&ComputationSignature> {
        self.computation_signatures
            .iter()
            .filter(|cs| cs.ret == fact_name)
            .collect()
    }
}

impl ComputationSignature {
    pub fn cut(
        &self,
        lib: &Library,
        cut_param: &str,
        lemma: &ComputationSignature,
    ) -> ComputationSignature {
        let mut cut_param_fact_name = None;
        let mut params = vec![];
        for (n, f, m) in &self.params {
            if n == cut_param {
                cut_param_fact_name = Some(f);
            } else {
                params.push((n.clone(), f.clone(), m.clone()));
            }
        }
        let cut_param_fact_name = cut_param_fact_name.unwrap();

        for (n, f, m) in &lemma.params {
            params.push((format!("lemma/{}", n), f.clone(), m.clone()));
        }

        ComputationSignature {
            name: format!("&cut_{}_{}", self.name, lemma.name),
            params,
            ret: self.ret.clone(),
            precondition: self
                .precondition
                .iter()
                .cloned()
                .chain(
                    lemma
                        .precondition
                        .iter()
                        .map(|pr| pr.prefix_vars("lemma/")),
                )
                .chain(
                    lib.fact_signature(&cut_param_fact_name)
                        .unwrap()
                        .params
                        .iter()
                        .map(|(n, _)| {
                            PredicateRelation::BinOp(
                                PredicateRelationBinOp::Eq,
                                PredicateAtom::Select {
                                    selector: n.clone(),
                                    arg: cut_param.to_owned(),
                                },
                                PredicateAtom::Select {
                                    selector: n.clone(),
                                    arg: "lemma/ret".to_owned(),
                                },
                            )
                        }),
                )
                .collect(),
        }
    }
}
