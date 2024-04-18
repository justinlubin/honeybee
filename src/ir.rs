#[derive(Debug, Clone)]
pub enum ValueType {
    Int,
    Str,
}

#[derive(Debug, Clone)]
pub enum Value {
    Int(u32),
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

pub enum Signature {
    Fact(FactSignature),
    Computation(ComputationSignature),
}

pub type Library = Vec<Signature>;

#[derive(Debug, Clone)]
pub struct Program {
    pub annotations: Vec<Fact>,
    pub goal: Fact,
}
