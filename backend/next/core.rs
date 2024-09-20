use crate::pbn;

use im::hashmap::HashMap;
use im::hashset::HashSet;
use im::vector::Vector;

// Values

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

impl Value {
    pub fn infer(&self) -> ValueType {
        match self {
            Value::Int(_) => ValueType::Int,
            Value::Str(_) => ValueType::Str,
        }
    }
}

// Parameters

pub type FactParam = String;
pub type FunctionParam = String;

// Assignments

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct AssignmentType {
    pub map: HashMap<FactParam, ValueType>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Assignment {
    pub map: HashMap<FactParam, Value>,
}

impl Assignment {
    pub fn infer(&self) -> AssignmentType {
        AssignmentType {
            map: self
                .map
                .iter()
                .map(|(fp, v)| (fp.clone(), v.infer()))
                .collect(),
        }
    }
}

// Facts

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct FactType {
    pub params: AssignmentType,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Fact {
    pub name: FactName,
    pub args: Assignment,
}

pub type FactName = String;
pub type FactLibrary = HashMap<FactName, FactType>;

pub type FactSet = HashSet<Fact>;

// Formulas

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum FormulaAtom {
    Param {
        function_param: FunctionParam,
        fact_param: FactParam,
    },
    Ret {
        fact_param: FactParam,
    },
    Value(Value),
}

impl FormulaAtom {
    pub fn infer(
        &self,
        olib: &FactLibrary,
        params: &HashMap<FunctionParam, FactName>,
        ret_name: &FactName,
    ) -> Option<ValueType> {
        Some(match self {
            FormulaAtom::Param {
                function_param,
                fact_param,
            } => olib
                .get(params.get(function_param)?)?
                .params
                .map
                .get(fact_param)?
                .clone(),
            FormulaAtom::Ret { fact_param } => {
                olib.get(ret_name)?.params.map.get(fact_param)?.clone()
            }
            FormulaAtom::Value(v) => v.infer(),
        })
    }

    pub fn eval(
        &self,
        args: &HashMap<FunctionParam, Assignment>,
        ret: &Assignment,
    ) -> Value {
        match self {
            FormulaAtom::Param {
                function_param,
                fact_param,
            } => args
                .get(function_param)
                .unwrap()
                .map
                .get(fact_param)
                .unwrap()
                .clone(),
            FormulaAtom::Ret { fact_param } => {
                ret.map.get(fact_param).unwrap().clone()
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
    pub fn check(
        &self,
        olib: &FactLibrary,
        ilib: &FactLibrary,
        params: &HashMap<FunctionParam, FactName>,
        ret_name: &FactName,
    ) -> bool {
        match self {
            Formula::True => true,
            Formula::Eq(a1, a2) => match (
                a1.infer(olib, params, ret_name),
                a2.infer(olib, params, ret_name),
            ) {
                (Some(vt1), Some(vt2)) => vt1 == vt2,
                _ => false,
            },
            Formula::Lt(a1, a2) => match (
                a1.infer(olib, params, ret_name),
                a2.infer(olib, params, ret_name),
            ) {
                (Some(ValueType::Int), Some(ValueType::Int)) => true,
                _ => false,
            },
            Formula::Fact(fact_name, fact_args) => {
                let expected = match ilib.get(fact_name) {
                    Some(ft) => &ft.params.map,
                    None => return false,
                };

                let actual = HashMap::new();
                for (x, a) in fact_args {
                    match a.infer(olib, params, ret_name) {
                        Some(vt) => actual.update(x.clone(), vt),
                        None => return false,
                    };
                }

                *expected == actual
            }
            Formula::And(phi1, phi2) => {
                phi1.check(olib, ilib, params, ret_name)
                    && phi2.check(olib, ilib, params, ret_name)
            }
        }
    }

    pub fn sat(
        &self,
        args: &HashMap<FunctionParam, Assignment>,
        ret: &Assignment,
        facts: &FactSet,
    ) -> bool {
        match self {
            Formula::True => true,
            Formula::Eq(a1, a2) => a1.eval(args, ret) == a2.eval(args, ret),
            Formula::Lt(a1, a2) => {
                match (a1.eval(args, ret), a2.eval(args, ret)) {
                    (Value::Int(i1), Value::Int(i2)) => i1 < i2,
                    _ => panic!(),
                }
            }
            Formula::Fact(fact_name, fact_args) => facts.contains(&Fact {
                name: fact_name.clone(),
                args: Assignment {
                    map: fact_args
                        .iter()
                        .map(|(x, a)| (x.clone(), a.eval(args, ret)))
                        .collect(),
                },
            }),
            Formula::And(phi1, phi2) => {
                phi1.sat(args, ret, facts) && phi2.sat(args, ret, facts)
            }
        }
    }
}

// Functions

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct FunctionType {
    pub params: HashMap<FunctionParam, FactName>,
    pub ret: FactName,
    pub precondition: Formula,
}

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

type FunctionName = String;
pub type FunctionLibrary = HashMap<FunctionName, FunctionType>;

// Libraries

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Library {
    pub input: FactLibrary,
    pub output: FactLibrary,
    pub function: FunctionLibrary,
}

// Judgments

pub fn check(
    lib: Library,
    input_facts: FactSet,
    e: pbn::Exp<Function>,
    alpha: Fact,
) -> bool {
    match e {
        pbn::Exp::Hole(_) => false,
        pbn::Exp::App(f, args) => {
            let fun_sig = lib.function.get(&f.name).unwrap();
            if f.metadata != alpha.args {
                return false;
            }
            todo!()
        }
    }
}
