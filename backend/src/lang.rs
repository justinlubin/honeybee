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
    Value(Value),
}

impl FormulaAtom {
    pub fn eval(
        &self,
        args: &HashMap<String, HashMap<String, Value>>,
    ) -> Value {
        match self {
            FormulaAtom::Param { arg, subarg } => {
                args.get(arg).unwrap().get(subarg).unwrap().clone()
            }
            FormulaAtom::Value(v) => v.clone(),
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Formula {
    True,
    Eq(FormulaAtom, FormulaAtom),
    Lt(FormulaAtom, FormulaAtom),
    Fact(FactName, HashMap<String, FormulaAtom>),
    And(Box<Formula>, Box<Formula>),
}

impl Formula {
    pub fn sat(
        &self,
        args: &HashMap<String, HashMap<String, Value>>,
        facts: &FactSet,
    ) -> bool {
        match self {
            Formula::True => true,
            Formula::Eq(a1, a2) => a1.eval(args) == a2.eval(args),
            Formula::Lt(a1, a2) => match (a1.eval(args), a2.eval(args)) {
                (Value::Int(i1), Value::Int(i2)) => i1 < i2,
                _ => panic!(),
            },
            Formula::Fact(fact_name, fact_args) => facts.contains(&Fact {
                name: fact_name.clone(),
                args: fact_args
                    .iter()
                    .map(|(x, a)| (x.clone(), a.eval(args)))
                    .collect(),
            }),
            Formula::And(phi1, phi2) => {
                phi1.sat(args, facts) && phi2.sat(args, facts)
            }
        }
    }
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
    pub input: FactLibrary,
    pub output: FactLibrary,
    pub function: FunctionLibrary,
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
            // let fun_sig = lib.function.get(&f).unwrap();
            todo!()
        }
    }
}
