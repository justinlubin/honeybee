//! # Honeybee core syntax
//!
//! This module defines the core syntax for Honeybee.

use crate::top_down::*;

use indexmap::{IndexMap, IndexSet};
use serde::{Deserialize, Serialize};

/// The type of type errors used by this module.
#[derive(Debug)]
pub struct LangError {
    pub context: Vec<String>,
    pub message: String,
    _private: (),
}

impl LangError {
    pub fn with_context(mut self, ctx: String) -> Self {
        self.context.push(ctx);
        self
    }

    pub fn new(message: String) -> Self {
        Self {
            context: vec![],
            message,
            _private: (),
        }
    }

    pub fn fp(fp: &FunParam) -> Self {
        Self::new(format!("unknown function parameter '{}'", fp.0))
    }

    pub fn mn(name: &MetName) -> Self {
        Self::new(format!("unknown metadata name '{}'", name.0))
    }

    pub fn mp(mp: &MetParam) -> Self {
        Self::new(format!("unknown metadata parameter '{}'", mp.0))
    }

    pub fn bf(bf: &BaseFunction) -> Self {
        Self::new(format!("unknown base function '{}'", bf.0))
    }

    pub fn argcount(got: usize, expected: usize) -> Self {
        Self::new(format!("got {} args, expected {}", got, expected))
    }
}

////////////////////////////////////////////////////////////////////////////////
// Values

/// The types that values may take on.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
pub struct MetName(pub String);

/// The type of metadata-indexed tuple parameter keys.
///
/// Types, type signatures, and atomic proposition formulas contain maps indexed
/// by metadata parameters. Generally, the values of these maps will be things
/// that "look like" metadata values or value types.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct MetParam(pub String);

/// Signatures for metadata-indexed tuples that define their arity.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct MetSignature {
    pub params: IndexMap<MetParam, ValueType>,
}

/// Libraries of metadata-indexed tuples.
pub type MetLibrary = IndexMap<MetName, MetSignature>;

/// The type of metadata-indexed tuples.
///
/// This struct is used for atomic propositions and types
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Met<T> {
    pub name: MetName,
    pub args: IndexMap<MetParam, T>,
}

impl<T> Met<T> {
    pub fn context(&self) -> String {
        format!("metadata tuple '{}'", self.name.0).to_owned()
    }
}

////////////////////////////////////////////////////////////////////////////////
// Formulas

pub struct EvaluationContext<'a> {
    pub args: &'a IndexMap<FunParam, IndexMap<MetParam, Value>>,
    pub ret: &'a IndexMap<MetParam, Value>,
}

/// The type of formula atoms.
///
/// Conceptually, formula atoms "evaluate" to a value in a particular context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormulaAtom {
    Param(FunParam, MetParam),
    Ret(MetParam),
    Lit(Value),
}

impl FormulaAtom {
    fn eval(&self, ctx: &EvaluationContext) -> Result<Value, LangError> {
        match self {
            FormulaAtom::Param(fp, mp) => ctx
                .args
                .get(fp)
                .ok_or_else(|| LangError::fp(fp))?
                .get(mp)
                .ok_or_else(|| LangError::mp(mp))
                .cloned(),
            FormulaAtom::Ret(mp) => {
                ctx.ret.get(mp).ok_or_else(|| LangError::mp(mp)).cloned()
            }
            FormulaAtom::Lit(v) => Ok(v.clone()),
        }
    }

    pub fn vals(&self) -> IndexSet<Value> {
        match self {
            FormulaAtom::Param(_, _) => IndexSet::new(),
            FormulaAtom::Ret(_) => IndexSet::new(),
            FormulaAtom::Lit(v) => IndexSet::from([v.clone()]),
        }
    }
}

pub type AtomicProposition = Met<Option<FormulaAtom>>;

impl AtomicProposition {
    fn matches(
        &self,
        p: &Met<Value>,
        ctx: &EvaluationContext,
    ) -> Result<bool, LangError> {
        if self.name != p.name {
            return Ok(false);
        }
        if self.args.len() != p.args.len() {
            return Ok(false);
        }
        for (mp, ofa) in &self.args {
            let v = match p.args.get(mp) {
                Some(v) => v,
                None => return Ok(false),
            };
            let fa = match ofa {
                Some(fa) => fa,
                None => continue,
            };
            if fa.eval(ctx)? != *v {
                return Ok(false);
            }
        }
        Ok(true)
    }

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
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(try_from = "Vec<String>")]
pub enum Formula {
    True,
    Eq(FormulaAtom, FormulaAtom),
    Lt(FormulaAtom, FormulaAtom),
    Ap(AtomicProposition),
    And(Box<Formula>, Box<Formula>),
}

impl Formula {
    pub fn conjunct(fs: impl Iterator<Item = Formula>) -> Formula {
        let mut phi = Self::True;
        for f in fs {
            phi = Self::And(Box::new(phi), Box::new(f))
        }
        phi
    }

    pub fn sat(
        &self,
        props: &Vec<Met<Value>>,
        ctx: &EvaluationContext,
    ) -> Result<bool, LangError> {
        match self {
            Formula::True => Ok(true),
            Formula::Eq(fa1, fa2) => Ok(fa1.eval(ctx)? == fa2.eval(ctx)?),
            Formula::Lt(fa1, fa2) => match (fa1.eval(ctx)?, fa2.eval(ctx)?) {
                (Value::Int(x1), Value::Int(x2)) => Ok(x1 < x2),
                (v1, v2) => Err(LangError::new(format!(
                    "Lt only supported for ints, got {:?} and {:?}",
                    v1, v2,
                ))),
            },
            Formula::Ap(ap) => {
                for p in props {
                    if ap.matches(p, ctx)? {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            Formula::And(phi1, phi2) => {
                Ok(phi1.sat(props, ctx)? && phi2.sat(props, ctx)?)
            }
        }
    }

    pub fn vals(&self) -> IndexSet<Value> {
        match self {
            Formula::True => IndexSet::new(),
            Formula::Eq(fa1, fa2) | Formula::Lt(fa1, fa2) => {
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
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct FunctionSignature {
    pub params: IndexMap<FunParam, MetName>,
    pub ret: MetName,
    pub condition: Formula,
}

impl FunctionSignature {
    pub fn vals(&self) -> IndexSet<Value> {
        self.condition.vals()
    }
}

/// Libraries of defined parameterized functions.
pub type FunctionLibrary = IndexMap<BaseFunction, FunctionSignature>;

////////////////////////////////////////////////////////////////////////////////
// Composite libraries and programs

/// The libraries necessary for a Honeybee problem.
#[derive(Clone, Deserialize)]
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

    pub fn new(
        flib: &FunctionLibrary,
        name: BaseFunction,
        metadata: IndexMap<MetParam, Value>,
    ) -> Result<Self, LangError> {
        Ok(Self::from_sig(
            flib.get(&name).ok_or_else(|| LangError::bf(&name))?,
            name,
            metadata,
        ))
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

impl Exp {
    /// The typing relation for Honeybee core syntax.
    ///
    /// Holes are not well-typed. Function applications are well-typed if their
    /// arguments are well-typed and have metadata satisfying the validity
    /// condition of the function (and the metadata is contained within the
    /// allowable domain defined by the presence of values in the libraries).
    pub fn infer(
        &self,
        flib: &FunctionLibrary,
        props: &Vec<Met<Value>>,
    ) -> Result<Met<Value>, LangError> {
        match self {
            Sketch::Hole(_) => {
                Err(LangError::new("holes are not well-typed".to_string()))
            }
            Sketch::App(f, args) => {
                // Get signature
                let sig =
                    flib.get(&f.name).ok_or_else(|| LangError::bf(&f.name))?;

                // Compute domain
                let mut vals = IndexSet::new();
                for fs in flib.values() {
                    vals.extend(fs.vals());
                }
                for p in props {
                    vals.extend(p.args.values().cloned());
                }
                vals.extend(f.metadata.values().cloned());

                // Recursively infer values and check proper domain
                let mut ctx_args = IndexMap::new();
                for (fp, e) in args {
                    let tau = e.infer(flib, props)?;
                    let metadata = tau.args;
                    for v in metadata.values() {
                        if !vals.contains(v) {
                            return Err(LangError::new(format!(
                                "value {:?} not in domain",
                                v
                            )));
                        }
                    }
                    ctx_args.insert(fp.clone(), metadata);
                }

                // Check condition
                let ctx = EvaluationContext {
                    args: &ctx_args,
                    ret: &f.metadata,
                };
                if !sig.condition.sat(props, &ctx)? {
                    return Err(LangError::new(format!(
                        "condition {:?} not satisfied for {:?}",
                        sig.condition, self
                    )));
                }

                Ok(Met {
                    name: sig.ret.clone(),
                    args: f.metadata.clone(),
                })
            }
        }
    }
}

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

pub struct Goal {
    pub function: BaseFunction,
    pub param: FunParam,
    signature: FunctionSignature,
}

impl Goal {
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
        };

        Self {
            function,
            param,
            signature,
        }
    }

    pub fn add_to_library(&self, functions: &mut FunctionLibrary) {
        functions.insert(self.function.clone(), self.signature.clone());
    }

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
