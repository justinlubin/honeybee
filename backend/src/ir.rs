use std::collections::HashMap;
use std::collections::HashSet;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ValueType {
    Int,
    Str,
    List(Box<ValueType>),
}

#[derive(
    Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fact {
    pub name: FactName,
    pub args: Vec<(String, Value)>,
}

impl PartialEq for Fact {
    fn eq(&self, other: &Self) -> bool {
        let mut self_args = self.args.clone();
        self_args.sort_by(|(k1, _), (k2, _)| k1.cmp(k2));
        let mut other_args = other.args.clone();
        other_args.sort_by(|(k1, _), (k2, _)| k1.cmp(k2));
        self.name == other.name && self_args == other_args
    }
}

impl Eq for Fact {}

impl Fact {
    pub fn check(
        &self,
        lib: &Library,
        expected_kind: Option<FactKind>,
    ) -> Result<(), String> {
        match lib.fact_signature(&self.name) {
            Some(fs) => {
                match expected_kind {
                    Some(k) => {
                        if fs.kind != k {
                            return Err(format!(
                                "{} '{:?}', got '{:?}' fact {:?}",
                                "expected fact kind", k, fs.kind, self,
                            ));
                        };
                    }
                    None => (),
                };
                if self.args.len() != fs.params.len() {
                    return Err(format!(
                        "fact {:?} should have {} arguments",
                        self,
                        fs.params.len()
                    ));
                };
                for (k, v) in &self.args {
                    match fs.params.iter().find(|(k2, _)| *k2 == *k) {
                        Some((_, vt)) => {
                            if v.infer() != *vt {
                                return Err(format!(
                                    "argument '{}' is {:?} {} {:?}",
                                    k, v, "but should have type", vt
                                ));
                            };
                        }
                        None => {
                            return Err(format!(
                                "unknown fact argument '{}'",
                                k
                            ));
                        }
                    }
                }
                Ok(())
            }
            None => Err(format!("unknown fact name: {}", self.name)),
        }
    }
}

impl std::hash::Hash for Fact {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        let mut self_args = self.args.clone();
        self_args.sort_by(|(k1, _), (k2, _)| k1.cmp(k2));
        self_args.hash(state);
    }
}

#[derive(
    Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
)]
pub enum PredicateAtom {
    Select { selector: String, arg: String },
    Const(Value),
}

impl PredicateAtom {
    // TODO: needs ret somehow?
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

    // TODO redundant with above?
    pub fn infer(
        &self,
        ret: &FactSignature,
        params: &Vec<(String, FactSignature)>,
    ) -> Result<ValueType, String> {
        match self {
            PredicateAtom::Select { selector, arg } => {
                let fs = if arg == Query::RET {
                    ret
                } else {
                    params
                        .iter()
                        .find_map(
                            |(k, fs)| if k == arg { Some(fs) } else { None },
                        )
                        .ok_or(format!("cannot find argument '{}'", arg))?
                };
                fs.params
                    .iter()
                    .find_map(|(k, vt)| {
                        if k == selector {
                            Some(vt.clone())
                        } else {
                            None
                        }
                    })
                    .ok_or(format!(
                        "cannot find selector '{}' in argument '{}'",
                        selector, arg
                    ))
            }
            PredicateAtom::Const(v) => Ok(v.infer()),
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

    pub fn eval(&self, ret: &Fact, args: &Vec<(String, Fact)>) -> Value {
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
                        .expect(&format!(
                            "arg '{}' should match in {:?}",
                            *arg, args
                        ))
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

#[derive(
    Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
)]
pub enum PredicateRelationBinOp {
    Eq,
    Lt,
    Lte,
    Contains,
}

#[derive(
    Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
)]
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
        subs: &Vec<(String, String, Value)>,
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

    pub fn sat(&self, ret: &Fact, args: &Vec<(String, Fact)>) -> bool {
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

    pub fn check(
        &self,
        ret: &FactSignature,
        params: &Vec<(String, FactSignature)>,
    ) -> Result<(), String> {
        match self {
            PredicateRelation::BinOp(op, a1, a2) => {
                let vt1 = a1.infer(ret, params)?;
                let vt2 = a2.infer(ret, params)?;
                match op {
                    PredicateRelationBinOp::Eq => {
                        if vt1 != vt2 {
                            let lhs = format!("LHS {:?} is {:?}", a1, vt1);
                            let rhs = format!("RHS {:?} is {:?}", a2, vt2);
                            return Err(format!(
                                "unequal types: {}, {}",
                                lhs, rhs
                            ));
                        };
                    }
                    PredicateRelationBinOp::Lt
                    | PredicateRelationBinOp::Lte => {
                        if vt1 != ValueType::Int {
                            return Err(format!("LHS not an Int: {:?}", a1));
                        };
                        if vt2 != ValueType::Int {
                            return Err(format!("RHS not an Int: {:?}", a2));
                        };
                    }
                    PredicateRelationBinOp::Contains => todo!(),
                }
            }
        }
        Ok(())
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

    pub fn check(&self) -> Result<(), String> {
        for cs in &self.computation_signatures {
            cs.check(self).map_err(|e| {
                format!("Type error in computation '{}': {}", cs.name, e)
            })?;
        }
        Ok(())
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

    pub fn check(&self, lib: &Library) -> Result<(), String> {
        let ret_sig = match lib.fact_signature(&self.ret) {
            Some(fs) => match fs.kind {
                FactKind::Input => {
                    return Err(format!("ret fact kind must be Output"))
                }
                FactKind::Output => fs,
            },
            None => return Err(format!("unknown ret fact name: {}", self.ret)),
        };
        let mut param_sigs = vec![];
        for (p, fact_name, _mode) in &self.params {
            match lib.fact_signature(fact_name) {
                Some(fs) => param_sigs.push((p.clone(), fs.clone())),
                None => {
                    return Err(format!(
                        "unknown fact name for param '{}': {}",
                        p, self.name
                    ));
                }
            }
        }
        for pr in &self.precondition {
            pr.check(&ret_sig, &param_sigs)?;
        }
        Ok(())
    }
}

impl Program {
    pub fn check(&self, lib: &Library) -> Result<(), String> {
        self.goal
            .check(lib, Some(FactKind::Output))
            .map_err(|e| format!("goal error: {}", e))?;
        for a in &self.annotations {
            a.check(lib, Some(FactKind::Input))
                .map_err(|e| format!("annotation error: {}", e))?;
        }
        Ok(())
    }
}
