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
