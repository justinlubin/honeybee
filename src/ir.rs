#[derive(Debug, Clone)]
pub enum ValueType {
    Int,
    Str,
    List(Box<ValueType>),
}

#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    Str(String),
    List(Vec<Value>),
}

pub type Assignment = std::collections::HashMap<String, Value>;

pub type FactName = String;

#[derive(Debug, Clone)]
pub enum FactKind {
    Annotation,
    Analysis,
}

#[derive(Debug, Clone)]
pub struct FactSignature {
    pub name: FactName,
    pub kind: FactKind,
    pub params: Vec<(String, ValueType)>,
}

#[derive(Debug, Clone)]
pub struct Fact {
    pub name: FactName,
    pub args: Vec<(String, Value)>,
}

#[derive(Debug, Clone)]
pub enum PredicateAtom {
    Select { selector: String, arg: String },
    Const(Value),
}

impl PredicateAtom {
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

    pub fn substitute_all(
        &self,
        subs: Vec<(&str, &str, &Value)>,
    ) -> PredicateAtom {
        let mut ret = self.clone();
        for (selector, arg, rhs) in subs {
            ret = ret.substitute(selector, arg, rhs);
        }
        ret
    }
}

#[derive(Debug, Clone)]
pub enum PredicateRelationBinOp {
    Eq,
    Lt,
    Lte,
    Contains,
}

#[derive(Debug, Clone)]
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
}

pub type Predicate = Vec<PredicateRelation>;

#[derive(Debug, Clone)]
pub struct ComputationSignature {
    pub name: String,
    pub params: Vec<(String, FactName)>,
    pub ret: FactName,
    pub precondition: Predicate,
}

#[derive(Debug, Clone)]
pub struct Library {
    pub fact_signatures: Vec<FactSignature>,
    pub computation_signatures: Vec<ComputationSignature>,
}

#[derive(Debug, Clone)]
pub struct Program {
    pub annotations: Vec<Fact>,
    pub goal: Fact,
}

#[derive(Debug, Clone)]
pub struct Query {
    pub fact_signature: FactSignature,
    pub computation_signature: ComputationSignature,
}

impl Query {
    pub const RET: &'static str = "ret";

    const GOAL_FACT_NAME: &'static str = "&GOAL";
    const GOAL_COMPUTATION_NAME: &'static str = "&goal";

    pub fn from_fact(fact: &Fact) -> Query {
        Query {
            fact_signature: FactSignature {
                name: Query::GOAL_FACT_NAME.to_owned(),
                kind: FactKind::Analysis,
                params: vec![],
            },
            computation_signature: ComputationSignature {
                name: Query::GOAL_COMPUTATION_NAME.to_owned(),
                params: vec![("q".to_owned(), fact.name.clone())],
                ret: Query::GOAL_FACT_NAME.to_owned(),
                precondition: fact
                    .args
                    .iter()
                    .map(|(n, v)| {
                        PredicateRelation::BinOp(
                            PredicateRelationBinOp::Eq,
                            PredicateAtom::Select {
                                selector: n.clone(),
                                arg: "q".to_owned(),
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
                kind: FactKind::Analysis,
                params: fs_params,
            },
            computation_signature: ComputationSignature {
                name: Query::GOAL_COMPUTATION_NAME.to_owned(),
                params,
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

    pub fn singleton(fact_signature: &FactSignature) -> Library {
        Library {
            fact_signatures: vec![fact_signature.clone()],
            computation_signatures: vec![],
        }
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
        for (n, f) in &self.params {
            if n == cut_param {
                cut_param_fact_name = Some(f);
            } else {
                params.push((n.clone(), f.clone()));
            }
        }
        let cut_param_fact_name = cut_param_fact_name.unwrap();

        for (n, f) in &lemma.params {
            params.push((format!("lemma/{}", n), f.clone()));
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
