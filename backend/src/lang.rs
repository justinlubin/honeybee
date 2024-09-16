use crate::pbn;

use im::hashmap::HashMap;
use im::hashset::HashSet;
use im::vector::Vector;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum ValueType {
    Int,
    Str,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Value {
    Int(i64),
    Str(String),
}

pub type Assignment = HashMap<String, Value>;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct FactSignature {
    pub params: HashMap<String, ValueType>,
}
pub type FactName = String;

pub type FactLibrary = HashMap<FactName, FactSignature>;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum FormulaAtom {
    Param { arg: String, subarg: String },
    FreeVar(String),
    Value(Value),
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Formula {
    True,
    Eq(FormulaAtom, FormulaAtom),
    Lt(FormulaAtom, FormulaAtom),
    Fact(FactName, Vector<FormulaAtom>),
    Not(Box<Formula>),
    And(Box<Formula>, Box<Formula>),
    Or(Box<Formula>, Box<Formula>),
    Forall(String, Box<Formula>),
    Exists(String, Box<Formula>),
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct FunctionSignature {
    pub params: HashMap<String, FactName>,
    pub ret: FactName,
    pub precondition: Formula,
}

type FunctionName = String;

pub type FunctionLibrary = HashMap<FunctionName, FunctionSignature>;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Library {
    pub inputs: FactLibrary,
    pub outputs: FactLibrary,
    pub functions: FunctionLibrary,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Fact {
    pub name: FactName,
    pub args: HashMap<String, Value>,
}

pub type FactSet = HashSet<Fact>;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Function {
    pub name: FunctionName,
    pub metadata: Assignment,
    _arity: HashSet<String>,
}

impl pbn::Function for Function {
    fn arity(&self) -> HashSet<String> {
        self._arity.clone()
    }
}

pub fn infer(
    lib: Library,
    input_facts: FactSet,
    e: pbn::Exp<Function>,
) -> Option<Fact> {
    match e {
        pbn::Exp::Hole(_) => None,
        pbn::Exp::App(f, args) => {
            todo!()
        }
    }
}
