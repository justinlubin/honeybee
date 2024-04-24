#[derive(Debug, Clone)]
pub enum ValueType {
    Int,
    Str,
}

#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    Str(String),
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
}

impl PredicateAtom {
    fn prefix_vars(&self, prefix: &str) -> PredicateAtom {
        match self {
            PredicateAtom::Select { selector, arg } => PredicateAtom::Select {
                selector: selector.clone(),
                arg: format!("{}{}", prefix, arg),
            },
        }
    }
}

#[derive(Debug, Clone)]
pub enum PredicateRelation {
    Eq(PredicateAtom, PredicateAtom),
    Lt(PredicateAtom, PredicateAtom),
}

impl PredicateRelation {
    pub fn prefix_vars(&self, prefix: &str) -> PredicateRelation {
        match self {
            PredicateRelation::Eq(lhs, rhs) => PredicateRelation::Eq(
                lhs.prefix_vars(prefix),
                rhs.prefix_vars(prefix),
            ),
            PredicateRelation::Lt(lhs, rhs) => PredicateRelation::Lt(
                lhs.prefix_vars(prefix),
                rhs.prefix_vars(prefix),
            ),
        }
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

impl ComputationSignature {
    fn cut(
        &self,
        lib: &Library,
        selector: &str,
        lemma: &ComputationSignature,
    ) -> ComputationSignature {
        let mut selector_fact_name = None;
        let mut new_params = vec![];
        for (n, f) in &self.params {
            if n == selector {
                selector_fact_name = Some(f);
            } else {
                new_params.push((format!("self/{}", n), f.clone()))
            }
        }
        let selector_fact_name = selector_fact_name.unwrap();

        for (n, f) in &lemma.params {
            new_params.push((format!("lemma/{}", n), f.clone()))
        }

        ComputationSignature {
            name: format!("{}+{}", self.name, lemma.name),
            params: self
                .params
                .iter()
                .filter_map(|(n, t)| {
                    if n == selector {
                        None
                    } else {
                        Some((n.clone(), t.clone()))
                    }
                })
                .chain(lemma.params.iter().cloned())
                .collect(),
            precondition: self
                .precondition
                .iter()
                .map(|pr| pr.prefix_vars("self/"))
                .chain(
                    lemma
                        .precondition
                        .iter()
                        .map(|pr| pr.prefix_vars("lemma/")),
                )
                .chain(
                    lib.fact_signature(selector_fact_name)
                        .unwrap()
                        .params
                        .iter()
                        .map(|(n, _)| {
                            PredicateRelation::Eq(
                                PredicateAtom::Select {
                                    selector: n.clone(),
                                    arg: format!("self/{}", selector),
                                },
                                PredicateAtom::Select {
                                    selector: n.clone(),
                                    arg: format!("lemma/{}", selector),
                                },
                            )
                        }),
                )
                .collect(),
            ret: self.ret.clone(),
        }
    }
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
pub enum Expression {
    Val(Value),
    Var(String),
}

#[derive(Debug, Clone)]
pub struct BasicQuery {
    pub name: FactName,
    pub args: Vec<(String, Expression)>,
}

pub struct Query {
    pub entries: Vec<(String, BasicQuery)>,
    pub side_condition: Predicate,
}

impl Fact {
    pub fn to_basic_query(&self) -> BasicQuery {
        BasicQuery {
            name: self.name.clone(),
            args: self
                .args
                .clone()
                .into_iter()
                .map(|(p, v)| (p, Expression::Val(v)))
                .collect(),
        }
    }

    pub fn to_query(&self) -> Query {
        Query {
            entries: vec![("q".to_owned(), self.to_basic_query())],
            side_condition: vec![],
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

impl BasicQuery {
    pub fn free_variables(&self, lib: &Library) -> Vec<(String, ValueType)> {
        let mut fvs = vec![];
        let vts = &lib.fact_signature(&self.name).unwrap().params;
        for (x, e) in &self.args {
            match e {
                Expression::Val(_) => continue,
                Expression::Var(var) => {
                    fvs.push((
                        var.clone(),
                        vts.iter()
                            .find_map(|(y, vt)| {
                                if x == y {
                                    Some(vt.clone())
                                } else {
                                    None
                                }
                            })
                            .unwrap(),
                    ))
                }
            };
        }
        fvs
    }
}

impl Query {
    pub fn free_variables(&self, lib: &Library) -> Vec<(String, ValueType)> {
        self.entries
            .iter()
            .flat_map(|(_, bq)| bq.free_variables(lib))
            .collect()
    }
}
