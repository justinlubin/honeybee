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

#[derive(Debug, Clone)]
pub enum PredicateRelation {
    Eq(PredicateAtom, PredicateAtom),
    Lt(PredicateAtom, PredicateAtom),
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
pub enum Signature {
    Fact(FactSignature),
    Computation(ComputationSignature),
}

#[derive(Debug, Clone)]
pub struct Library {
    pub signatures: Vec<Signature>,
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
    pub entries: Vec<BasicQuery>,
}

impl Fact {
    pub fn to_query(&self) -> Query {
        Query {
            entries: vec![BasicQuery {
                name: self.name.clone(),
                args: self
                    .args
                    .clone()
                    .into_iter()
                    .map(|(p, v)| (p, Expression::Val(v)))
                    .collect(),
            }],
        }
    }
}

impl Library {
    pub fn lookup(&self, name: &str) -> Option<&Signature> {
        self.signatures.iter().find(|s| match s {
            Signature::Fact(fs) => fs.name == *name,
            Signature::Computation(cs) => cs.name == *name,
        })
    }

    pub fn matching_computations(&self, fact_name: &str) -> Vec<&ComputationSignature> {
        self.signatures
            .iter()
            .filter_map(|s| match s {
                Signature::Computation(cs) if cs.ret == fact_name => Some(cs),
                _ => None,
            })
            .collect()
    }
}

impl Query {
    pub fn free_variables(&self, lib: &Library) -> Vec<(String, ValueType)> {
        let mut fvs = vec![];
        for bq in &self.entries {
            let vts = match lib.lookup(&bq.name) {
                Some(Signature::Fact(fs)) => &fs.params,
                _ => panic!(),
            };
            for (x, e) in &bq.args {
                match e {
                    Expression::Val(_) => continue,
                    Expression::Var(var) => fvs.push((
                        var.clone(),
                        vts.iter()
                            .find_map(|(y, vt)| if x == y { Some(vt.clone()) } else { None })
                            .unwrap(),
                    )),
                };
            }
        }
        fvs
    }
}
