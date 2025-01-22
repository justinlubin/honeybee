//! # Honeybee core syntax
//!
//! This module defines the core syntax for Honeybee.

use crate::next::top_down::*;

use indexmap::IndexMap;
use indexmap::IndexSet;
use serde::{Deserialize, Serialize};

////////////////////////////////////////////////////////////////////////////////
// Errors

/// The type of errors used by this module.
pub struct Error(String);

impl Error {
    fn fp(fp: &FunParam) -> Self {
        Error(format!("unknown function parameter {:?}", fp))
    }

    fn mn(name: &MetName) -> Self {
        Error(format!("unknown metadata name {:?}", name))
    }

    fn mp(mp: &MetParam) -> Self {
        Error(format!("unknown metadata parameter {:?}", mp))
    }

    fn bf(bf: &BaseFunction) -> Self {
        Error(format!("unknown base function {:?}", bf))
    }

    fn argcount(got: usize, expected: usize) -> Self {
        Error(format!("got {} args, expected {}", got, expected))
    }
}

////////////////////////////////////////////////////////////////////////////////
// Values

/// The types that values may take on.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum ValueType {
    Int,
    Str,
}

/// The possible values.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Value {
    Int(i64),
    Str(String),
}

impl Value {
    fn infer(&self) -> ValueType {
        match self {
            Value::Int(_) => ValueType::Int,
            Value::Str(_) => ValueType::Str,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// Types

/// The type of metadata-indexed tuple names.
///
/// Types and atomic propositions are named by this type. Consequently, type
/// names serve as the keys for type libraries.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct MetName(String);

/// The type of metadata-indexed tuple parameter keys.
///
/// Types, type signatures, and atomic proposition formulas contain maps indexed
/// by metadata parameters. Generally, the values of these maps will be things
/// that "look like" metadata values or value types.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct MetParam(String);

/// Signatures for metadata-indexed tuples that define their arity.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct MetSignature {
    params: IndexMap<MetParam, ValueType>,
}

/// Libraries of metadata-indexed tuples.
pub type MetLibrary = IndexMap<MetName, MetSignature>;

/// The type of metadata-indexed tuples.
///
/// This struct is used for atomic propositions and types
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Met<T> {
    name: MetName,
    args: IndexMap<MetParam, T>,
}

impl Met<Value> {
    fn infer(&self, mlib: &MetLibrary) -> Result<MetSignature, Error> {
        let sig = mlib.get(&self.name).ok_or(Error::mn(&self.name))?;

        if self.args.len() != sig.params.len() {
            return Err(Error::argcount(self.args.len(), sig.params.len()));
        }

        for (mp, v) in &self.args {
            let expected_vt = sig.params.get(mp).ok_or(Error::mp(mp))?;
            let got_vt = v.infer();

            if got_vt != *expected_vt {
                return Err(Error(format!(
                    "argument {:?} of {:?} is type {:?} but expected {:?}",
                    v, self.name, got_vt, expected_vt
                )));
            }
        }

        Ok(sig.clone())
    }
}

////////////////////////////////////////////////////////////////////////////////
// Formulas

pub struct EvaluationContext<'a> {
    args: &'a IndexMap<FunParam, IndexMap<MetParam, Value>>,
    ret: &'a IndexMap<MetParam, Value>,
}

/// The type of formula atoms.
///
/// Conceptually, formula atoms "evaluate" to a value in a particular context.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum FormulaAtom {
    Param(FunParam, MetParam),
    Ret(MetParam),
    Lit(Value),
}

impl FormulaAtom {
    fn infer(
        &self,
        mlib: &MetLibrary,
        fs: &FunctionSignature,
    ) -> Result<ValueType, Error> {
        match self {
            FormulaAtom::Param(fp, mp) => {
                let name = fs.params.get(fp).ok_or(Error::fp(fp))?;

                mlib.get(name)
                    .ok_or(Error::mn(name))?
                    .params
                    .get(mp)
                    .ok_or(Error::mp(mp))
                    .cloned()
            }
            FormulaAtom::Ret(mp) => mlib
                .get(&fs.ret)
                .ok_or(Error::mn(&fs.ret))?
                .params
                .get(mp)
                .ok_or(Error::mp(mp))
                .cloned(),
            FormulaAtom::Lit(v) => Ok(v.infer()),
        }
    }

    fn eval(&self, ctx: &EvaluationContext) -> Result<Value, Error> {
        match self {
            FormulaAtom::Param(fp, mp) => ctx
                .args
                .get(fp)
                .ok_or(Error::fp(fp))?
                .get(mp)
                .ok_or(Error::mp(mp))
                .cloned(),
            FormulaAtom::Ret(mp) => {
                ctx.ret.get(mp).ok_or(Error::mp(mp)).cloned()
            }
            FormulaAtom::Lit(v) => Ok(v.clone()),
        }
    }

    fn vals(&self) -> IndexSet<Value> {
        match self {
            FormulaAtom::Param(_, _) => IndexSet::new(),
            FormulaAtom::Ret(_) => IndexSet::new(),
            FormulaAtom::Lit(v) => IndexSet::from([v.clone()]),
        }
    }
}

impl Met<FormulaAtom> {
    fn check(
        &self,
        mlib: &MetLibrary,
        fs: &FunctionSignature,
    ) -> Result<(), Error> {
        let sig = mlib.get(&self.name).ok_or(Error::mn(&self.name))?;

        if self.args.len() != sig.params.len() {
            return Err(Error::argcount(self.args.len(), sig.params.len()));
        }

        for (mp, fa) in &self.args {
            let expected_vt = sig.params.get(mp).ok_or(Error::mp(mp))?;
            let got_vt = fa.infer(mlib, fs)?;

            if got_vt != *expected_vt {
                return Err(Error(format!(
                    "argument {:?} of atomic proposition {:?} is type {:?} but expected {:?}",
                    fa,
                    self.name,
                    got_vt,
                    expected_vt
                )));
            }
        }

        Ok(())
    }

    fn eval(&self, ctx: &EvaluationContext) -> Result<Met<Value>, Error> {
        let mut args = IndexMap::new();
        for (mp, fa) in &self.args {
            args.insert(mp.clone(), fa.eval(ctx)?);
        }
        Ok(Met {
            name: self.name.clone(),
            args,
        })
    }

    fn vals(&self) -> IndexSet<Value> {
        let mut ret = IndexSet::new();
        for fa in self.args.values() {
            ret.extend(fa.vals());
        }
        ret
    }
}

/// The type of formulas.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum Formula {
    True,
    Eq(FormulaAtom, FormulaAtom),
    Lt(FormulaAtom, FormulaAtom),
    AtomicProposition(Met<FormulaAtom>),
    And(Box<Formula>, Box<Formula>),
}

impl Formula {
    fn check_equal_types(
        mlib: &MetLibrary,
        fs: &FunctionSignature,
        fa1: &FormulaAtom,
        fa2: &FormulaAtom,
    ) -> Result<(), Error> {
        let vt1 = fa1.infer(mlib, fs)?;
        let vt2 = fa2.infer(mlib, fs)?;
        if vt1 != vt2 {
            return Err(Error(format!(
                "formula atom {:?} has different type ({:?}) than formula atom {:?} ({:?})",
                fa1, vt1, fa2, vt2
            )));
        }
        Ok(())
    }

    fn check(
        &self,
        mlib: &MetLibrary,
        fs: &FunctionSignature,
    ) -> Result<(), Error> {
        match self {
            Formula::True => Ok(()),
            Formula::Eq(fa1, fa2) => {
                Self::check_equal_types(mlib, fs, fa1, fa2)
            }
            Formula::Lt(fa1, fa2) => {
                let vt1 = fa1.infer(mlib, fs)?;
                if vt1 != ValueType::Int {
                    return Err(Error(format!(
                        "formula atom {:?} has type {:?}, expected Int",
                        fa1, vt1,
                    )));
                }
                Self::check_equal_types(mlib, fs, fa1, fa2)
            }
            Formula::AtomicProposition(ap) => ap.check(mlib, fs),
            Formula::And(phi1, phi2) => {
                phi1.check(mlib, fs)?;
                phi2.check(mlib, fs)
            }
        }
    }

    fn sat(
        &self,
        props: &Vec<Met<Value>>,
        ctx: &EvaluationContext,
    ) -> Result<bool, Error> {
        match self {
            Formula::True => Ok(true),
            Formula::Eq(fa1, fa2) => Ok(fa1.eval(ctx)? == fa2.eval(ctx)?),
            Formula::Lt(fa1, fa2) => match (fa1.eval(ctx)?, fa2.eval(ctx)?) {
                (Value::Int(x1), Value::Int(x2)) => Ok(x1 < x2),
                (v1, v2) => Err(Error(format!(
                    "Lt only supported for ints, got {:?} and {:?}",
                    v1, v2,
                ))),
            },
            Formula::AtomicProposition(ap) => {
                let prop = ap.eval(ctx)?;
                Ok(props.iter().any(|p| *p == prop))
            }
            Formula::And(phi1, phi2) => {
                Ok(phi1.sat(props, ctx)? && phi2.sat(props, ctx)?)
            }
        }
    }

    fn vals(&self) -> IndexSet<Value> {
        match self {
            Formula::True => IndexSet::new(),
            Formula::Eq(fa1, fa2) | Formula::Lt(fa1, fa2) => {
                let mut ret = fa1.vals();
                ret.extend(fa2.vals());
                ret
            }
            Formula::AtomicProposition(ap) => ap.vals(),
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
pub struct BaseFunction(String);

/// The type of signatures of parameterized functions.
///
/// The condition formula refers to the metadata values on the parameter types
/// and return type.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct FunctionSignature {
    params: IndexMap<FunParam, MetName>,
    ret: MetName,
    condition: Formula,
}

impl FunctionSignature {
    fn check(&self, mlib: &MetLibrary) -> Result<(), Error> {
        for type_name in self.params.values() {
            let _ = mlib.get(type_name).ok_or(Error::mn(type_name))?;
        }
        let _ = mlib.get(&self.ret).ok_or(Error::mn(&self.ret))?;
        self.condition.check(mlib, self)
    }

    fn vals(&self) -> IndexSet<Value> {
        self.condition.vals()
    }
}

/// Libraries of defined parameterized functions.
pub type FunctionLibrary = IndexMap<BaseFunction, FunctionSignature>;

////////////////////////////////////////////////////////////////////////////////
// Composite libraries and programs

/// The libraries necessary for a Honeybee problem.
#[derive(Deserialize, Serialize)]
pub struct Library {
    #[serde(rename = "Prop")]
    props: MetLibrary,
    #[serde(rename = "Type")]
    types: MetLibrary,
    #[serde(rename = "Function")]
    functions: FunctionLibrary,
}

impl Library {
    fn check(&self) -> Result<(), Error> {
        let pnames: IndexSet<_> = self.props.keys().cloned().collect();
        let tnames: IndexSet<_> = self.types.keys().cloned().collect();
        let ambiguous_names: IndexSet<_> =
            pnames.intersection(&tnames).collect();

        if !ambiguous_names.is_empty() {
            return Err(Error(format!(
                "ambiguous prop/type names: {:?}",
                ambiguous_names
            )));
        }

        for fs in self.functions.values() {
            fs.check(&self.types)?;
        }

        Ok(())
    }
}

/// The type of Honeybee programs.
#[derive(Deserialize, Serialize)]
pub struct Program {
    #[serde(rename = "Prop")]
    props: Vec<Met<Value>>,
    #[serde(rename = "Goal")]
    goal: Met<Value>,
}

impl Program {
    fn check(&self, lib: &Library) -> Result<(), Error> {
        for p in &self.props {
            let _ = p.infer(&lib.props)?;
        }

        let _ = self.goal.infer(&lib.types)?;

        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////
// Parameterized functions and expressions

/// The type of parameterized functions.
///
/// Parameterized functions consist of a base name as well as metadata arguments
/// that correspond to the metadata values for its return type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParameterizedFunction {
    name: BaseFunction,
    metadata: IndexMap<MetParam, Value>,
    // A copy of the function's arity is stored along with the function so that
    // the impl Function block below can work without reference to a library.
    arity: Vec<FunParam>,
}

impl ParameterizedFunction {
    pub fn new(
        flib: &FunctionLibrary,
        name: BaseFunction,
        metadata: IndexMap<MetParam, Value>,
    ) -> Result<Self, Error> {
        let arity = flib
            .get(&name)
            .ok_or(Error::bf(&name))?
            .params
            .keys()
            .cloned()
            .collect();
        Ok(ParameterizedFunction {
            name,
            metadata,
            arity,
        })
    }
}

impl Function for ParameterizedFunction {
    fn arity(&self) -> Vec<FunParam> {
        return self.arity.clone();
    }
}

/// The type of expressions used for core Honeybee.
pub type Exp = Sketch<ParameterizedFunction>;

impl Exp {
    /// The typing relation for Honeybee core syntax.
    ///
    /// Holes are not well-typed. Function applications are well-typed if their
    /// arguments are well-typed and have metadata satisfying the validity
    /// condition of the function (and the metadata is contained within the
    /// allowable domain defined by the presence of values in the libraries).
    fn infer(
        &self,
        flib: &FunctionLibrary,
        props: &Vec<Met<Value>>,
    ) -> Result<Met<Value>, Error> {
        match self {
            Sketch::Hole(_) => Err(Error(format!("holes are not well-typed"))),
            Sketch::App(f, args) => {
                // Get signature
                let sig = flib.get(&f.name).ok_or(Error::bf(&f.name))?;

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
                            return Err(Error(format!(
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
                    return Err(Error(format!(
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
/// [`Exp::well_typed`] for more information about what it means for an
/// expression to be well-typed.
pub struct Problem {
    lib: Library,
    prog: Program,
}

impl Problem {
    pub fn new(lib: Library, prog: Program) -> Result<Self, Error> {
        let ret = Self { lib, prog };
        ret.check()?;
        Ok(ret)
    }

    fn check(&self) -> Result<(), Error> {
        self.lib.check()?;
        self.prog.check(&self.lib)
    }
}
