//! # Honeybee core syntax
//!
//! This module defines the core syntax for Honeybee.

use crate::top_down::*;

use indexmap::{IndexMap, IndexSet};
use serde::{Deserialize, Serialize};

////////////////////////////////////////////////////////////////////////////////
// Values

/// The types that values may take on.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum ValueType {
    Bool,
    Int,
    Str,
}

/// The possible values.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Value {
    Bool(bool),
    Int(i64),
    Str(String),
}

////////////////////////////////////////////////////////////////////////////////
// Types

/// The type of metadata-indexed tuple names.
///
/// Types and atomic propositions are named by this type. Consequently, type
/// names serve as the keys for type libraries.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct MetName(pub String);

/// The type of metadata-indexed tuple parameter keys.
///
/// Types, type signatures, and atomic proposition formulas contain maps indexed
/// by metadata parameters. Generally, the values of these maps will be things
/// that "look like" metadata values or value types.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct MetParam(pub String);

/// Signatures for metadata-indexed tuples that define their arity.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MetSignature {
    /// The arity
    pub params: IndexMap<MetParam, ValueType>,

    /// Optional additional info that may be helpful for the end user
    pub info: Option<toml::Table>,
}

/// Libraries of metadata-indexed tuples.
pub type MetLibrary = IndexMap<MetName, MetSignature>;

/// The type of metadata-indexed tuples.
///
/// This struct is used for atomic propositions and types
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Met<T> {
    pub name: MetName,
    pub args: IndexMap<MetParam, T>,
}

impl<T> Met<T> {
    /// The context string for a Met.
    pub fn context(&self) -> String {
        format!("metadata tuple '{}'", self.name.0).to_owned()
    }
}

////////////////////////////////////////////////////////////////////////////////
// Formulas

/// The type of formula atoms.
///
/// Conceptually, formula atoms "evaluate" to a value in a particular context.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum FormulaAtom {
    Param(FunParam, MetParam),
    Ret(MetParam),
    Lit(Value),
}

impl FormulaAtom {
    /// Returns the set of values in a formula atom.
    pub fn vals(&self) -> IndexSet<Value> {
        match self {
            FormulaAtom::Param(_, _) => IndexSet::new(),
            FormulaAtom::Ret(_) => IndexSet::new(),
            FormulaAtom::Lit(v) => IndexSet::from([v.clone()]),
        }
    }
}

/// The type of atomic propositions (essentially, facts that may have
/// omitted arguments).
pub type AtomicProposition = Met<Option<FormulaAtom>>;

impl AtomicProposition {
    /// Returns the set of values in an atomic proposition.
    pub fn vals(&self) -> IndexSet<Value> {
        let mut ret = IndexSet::new();
        for ofa in self.args.values() {
            let fa = match ofa {
                Some(fa) => fa,
                None => continue,
            };
            ret.extend(fa.vals());
        }
        ret
    }
}

/// The type of formulas.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(try_from = "Vec<String>")]
pub enum Formula {
    True,
    Eq(FormulaAtom, FormulaAtom),
    Lt(FormulaAtom, FormulaAtom),
    Neq(FormulaAtom, FormulaAtom),
    Ap(AtomicProposition),
    And(Box<Formula>, Box<Formula>),
}

impl Formula {
    /// Create a conjunct of formulas.
    pub fn conjunct(fs: impl Iterator<Item = Formula>) -> Formula {
        let mut phi = Self::True;
        for f in fs {
            phi = Self::And(Box::new(phi), Box::new(f))
        }
        phi
    }

    /// Returns the set of values in a formula.
    pub fn vals(&self) -> IndexSet<Value> {
        match self {
            Formula::True => IndexSet::new(),
            Formula::Eq(fa1, fa2)
            | Formula::Lt(fa1, fa2)
            | Formula::Neq(fa1, fa2) => {
                let mut ret = fa1.vals();
                ret.extend(fa2.vals());
                ret
            }
            Formula::Ap(ap) => ap.vals(),
            Formula::And(phi1, phi2) => {
                let mut ret = phi1.vals();
                ret.extend(phi2.vals());
                ret
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// Function signatures

/// The type of base function names.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct BaseFunction(pub String);

/// The type of signatures of parameterized functions.
///
/// The condition formula refers to the metadata values on the parameter types
/// and return type.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FunctionSignature {
    pub params: IndexMap<FunParam, MetName>,
    pub ret: MetName,
    pub condition: Formula,

    /// Optional additional info that may be helpful for the end user (not
    /// checked in Eq instance)
    pub info: Option<toml::Table>,
}

impl FunctionSignature {
    /// Returns the set of values in a function signature.
    pub fn vals(&self) -> IndexSet<Value> {
        self.condition.vals()
    }
}

impl PartialEq for FunctionSignature {
    fn eq(&self, other: &Self) -> bool {
        self.params == other.params
            && self.ret == other.ret
            && self.condition == other.condition
    }
}

impl Eq for FunctionSignature {}

/// Libraries of defined parameterized functions.
pub type FunctionLibrary = IndexMap<BaseFunction, FunctionSignature>;

////////////////////////////////////////////////////////////////////////////////
// Composite libraries and programs

/// The libraries necessary for a Honeybee problem.
#[derive(Clone, Deserialize, Serialize)]
pub struct Library {
    #[serde(rename = "Prop")]
    pub props: MetLibrary,
    #[serde(rename = "Type")]
    pub types: MetLibrary,
    #[serde(rename = "Function")]
    pub functions: FunctionLibrary,
}

/// The type of Honeybee programs.
#[derive(Clone, Deserialize)]
pub struct Program {
    #[serde(rename = "Prop")]
    pub props: Vec<Met<Value>>,
    #[serde(rename = "Goal")]
    pub goal: Met<Value>,
}

////////////////////////////////////////////////////////////////////////////////
// Parameterized functions and expressions

/// The type of parameterized functions.
///
/// Parameterized functions consist of a base name as well as metadata arguments
/// that correspond to the metadata values for its return type.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct ParameterizedFunction {
    pub name: BaseFunction,
    pub metadata: IndexMap<MetParam, Value>,
    // A copy of the function's arity is stored along with the function so that
    // the impl Function block below can work without reference to a library.
    arity: Vec<FunParam>,
}

impl ParameterizedFunction {
    /// Creates a parameterized function from a function signature (usually
    /// resulting from selecting the function signature from a library.)
    pub fn from_sig(
        sig: &FunctionSignature,
        name: BaseFunction,
        metadata: IndexMap<MetParam, Value>,
    ) -> Self {
        ParameterizedFunction {
            name: name.clone(),
            metadata,
            arity: sig.params.keys().cloned().collect(),
        }
    }
}

impl Function for ParameterizedFunction {
    fn arity(&self) -> Vec<FunParam> {
        self.arity.clone()
    }
}

/// The type of expressions used for core Honeybee.
pub type Exp = Sketch<ParameterizedFunction>;

/// The type of steps used for core Honeybee.
pub type Step = TopDownStep<ParameterizedFunction>;

////////////////////////////////////////////////////////////////////////////////
// Synthesis problem

/// The type of synthesis problems for Honeybee core syntax.
///
/// The problem requires the synthesis of an expression at the goal type
/// assuming the provided libraries and atomic propositions. See
/// [`Exp::infer`] for more information about what it means for an
/// expression to be well-typed.
#[derive(Clone)]
pub struct Problem {
    pub library: Library,
    pub program: Program,
}

impl Problem {
    /// Returns the set of values in a problem.
    pub fn vals(&self) -> IndexSet<Value> {
        let mut dom = IndexSet::new();
        for fs in self.library.functions.values() {
            dom.extend(fs.vals());
        }
        for p in &self.program.props {
            dom.extend(p.args.values().cloned());
        }
        dom.extend(self.program.goal.args.values().cloned());
        dom
    }
}

////////////////////////////////////////////////////////////////////////////////
// Goal convenience wrapper

/// A convenience wrapper type for handling goal-related types and functions.
pub struct Goal {
    pub function: BaseFunction,
    pub param: FunParam,
    pub signature: FunctionSignature,
}

impl Goal {
    /// Create a new goal with the specified metadata
    pub fn new(goal: &Met<Value>) -> Self {
        let function = BaseFunction("&goal".to_owned());
        let param = FunParam("&goalparam".to_owned());
        let ret = MetName("&Goal".to_owned());

        let signature = FunctionSignature {
            condition: Formula::conjunct(goal.args.iter().map(|(mp, v)| {
                Formula::Eq(
                    FormulaAtom::Param(param.clone(), mp.clone()),
                    FormulaAtom::Lit(v.clone()),
                )
            })),
            ret: ret.clone(),
            params: IndexMap::from([(param.clone(), goal.name.clone())]),
            info: None,
        };

        Self {
            function,
            param,
            signature,
        }
    }

    /// Add a goal to a library
    pub fn add_to_library(&self, functions: &mut FunctionLibrary) {
        functions.insert(self.function.clone(), self.signature.clone());
    }

    /// Create an application whose head is the goal function
    pub fn app(
        &self,
        e: &Exp,
    ) -> (ParameterizedFunction, IndexMap<FunParam, Exp>) {
        (
            ParameterizedFunction::from_sig(
                &self.signature,
                self.function.clone(),
                IndexMap::new(),
            ),
            IndexMap::from([(self.param.clone(), e.clone())]),
        )
    }
}
