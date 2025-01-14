//! # Honeybee core syntax
//!
//! This module defines the core syntax for Honeybee.

use crate::next::pbn::*;
use crate::next::top_down::*;

use indexmap::IndexMap;
use indexmap::IndexSet;

////////////////////////////////////////////////////////////////////////////////
// Errors

/// The type of errors used by this module.
pub struct Error(String);

impl Error {
    fn fp(fp: &FunParam) -> Self {
        Error(format!("unknown function parameter {:?}", fp))
    }

    fn tn(name: &TypeName) -> Self {
        Error(format!("unknown type name {:?}", name))
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValueType {
    Int,
    Str,
}

/// The possible values.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

/// The type of type names.
///
/// Types and atomic propositions are named by this type. Consequently, type
/// names serve as the keys for type libraries.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypeName(String);

/// The type of metadata parameter keys.
///
/// Types, type signatures, and atomic proposition formulas contain maps indexed
/// by metadata parameters. Generally, the values of these maps will be things
/// that "look like" metadata values or value types.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MetParam(String);

/// The possible kinds that a type may be.
///
/// `Atomic` denotes atomic propositions (supplied by the user) and `Derived`
/// denotes types returned by functions in the library.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeKind {
    Atomic,
    Derived,
}

/// Signatures for types that define their arity and kind
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeSignature {
    params: IndexMap<MetParam, ValueType>,
    kind: TypeKind,
}

/// Libraries of defined types.
pub type TypeLibrary = IndexMap<TypeName, TypeSignature>;

/// The type of types.
///
/// Types are a type name with associated metadata arguments.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Type {
    name: TypeName,
    args: IndexMap<MetParam, Value>,
}

impl Type {
    fn infer(&self, tlib: &TypeLibrary) -> Result<TypeSignature, Error> {
        let sig = tlib.get(&self.name).ok_or(Error::tn(&self.name))?;

        if self.args.len() != sig.params.len() {
            return Err(Error::argcount(self.args.len(), sig.params.len()));
        }

        for (mp, v) in &self.args {
            let expected_vt = sig.params.get(mp).ok_or(Error::mp(mp))?;
            let got_vt = v.infer();

            if got_vt != *expected_vt {
                return Err(Error(format!(
                    "argument {:?} of type {:?} is type {:?} but expected {:?}",
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormulaAtom {
    Param(FunParam, MetParam),
    Ret(MetParam),
    Lit(Value),
}

impl FormulaAtom {
    fn infer(
        &self,
        tlib: &TypeLibrary,
        fs: &FunctionSignature,
    ) -> Result<ValueType, Error> {
        match self {
            FormulaAtom::Param(fp, mp) => {
                let name = fs.params.get(fp).ok_or(Error::fp(fp))?;

                tlib.get(name)
                    .ok_or(Error::tn(name))?
                    .params
                    .get(mp)
                    .ok_or(Error::mp(mp))
                    .cloned()
            }
            FormulaAtom::Ret(mp) => tlib
                .get(&fs.ret)
                .ok_or(Error::tn(&fs.ret))?
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

/// The type of atomic proposition formulas.
///
/// Atomic proposition formulas are exactly like types, but their arguments are
/// formula atoms rather than values. Consequently, atomic proposition formulas
/// can be "evaluated" into types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtomicPropositionFormula {
    name: TypeName,
    args: IndexMap<MetParam, FormulaAtom>,
}

impl AtomicPropositionFormula {
    fn check(
        &self,
        tlib: &TypeLibrary,
        fs: &FunctionSignature,
    ) -> Result<(), Error> {
        let sig = tlib.get(&self.name).ok_or(Error::tn(&self.name))?;

        if self.args.len() != sig.params.len() {
            return Err(Error::argcount(self.args.len(), sig.params.len()));
        }

        for (mp, fa) in &self.args {
            let expected_vt = sig.params.get(mp).ok_or(Error::mp(mp))?;
            let got_vt = fa.infer(tlib, fs)?;

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

    fn eval(&self, ctx: &EvaluationContext) -> Result<Type, Error> {
        let mut args = IndexMap::new();
        for (mp, fa) in &self.args {
            args.insert(mp.clone(), fa.eval(ctx)?);
        }
        Ok(Type {
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Formula {
    True,
    Eq(FormulaAtom, FormulaAtom),
    Lt(FormulaAtom, FormulaAtom),
    Ap(AtomicPropositionFormula),
    And(Box<Formula>, Box<Formula>),
}

impl Formula {
    fn check_equal_types(
        tlib: &TypeLibrary,
        fs: &FunctionSignature,
        fa1: &FormulaAtom,
        fa2: &FormulaAtom,
    ) -> Result<(), Error> {
        let vt1 = fa1.infer(tlib, fs)?;
        let vt2 = fa2.infer(tlib, fs)?;
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
        tlib: &TypeLibrary,
        fs: &FunctionSignature,
    ) -> Result<(), Error> {
        match self {
            Formula::True => Ok(()),
            Formula::Eq(fa1, fa2) => {
                Self::check_equal_types(tlib, fs, fa1, fa2)
            }
            Formula::Lt(fa1, fa2) => {
                let vt1 = fa1.infer(tlib, fs)?;
                if vt1 != ValueType::Int {
                    return Err(Error(format!(
                        "formula atom {:?} has type {:?}, expected Int",
                        fa1, vt1,
                    )));
                }
                Self::check_equal_types(tlib, fs, fa1, fa2)
            }
            Formula::Ap(ap) => ap.check(tlib, fs),
            Formula::And(phi1, phi2) => {
                phi1.check(tlib, fs)?;
                phi2.check(tlib, fs)
            }
        }
    }

    fn sat(
        &self,
        props: &Vec<Type>,
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
            Formula::Ap(ap) => {
                let t = ap.eval(ctx)?;
                Ok(props.iter().any(|prop| *prop == t))
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
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BaseFunction(String);

/// The type of signatures of parameterized functions.
///
/// The condition formula refers to the metadata values on the parameter types
/// and return type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionSignature {
    params: IndexMap<FunParam, TypeName>,
    ret: TypeName,
    condition: Formula,
}

impl FunctionSignature {
    fn check(&self, tlib: &TypeLibrary) -> Result<(), Error> {
        for type_name in self.params.values() {
            let sig = tlib.get(type_name).ok_or(Error::tn(type_name))?;
            if sig.kind != TypeKind::Derived {
                return Err(Error(format!(
                    "type {:?} in function signature {:?} must be derived",
                    type_name, self
                )));
            }
        }
        let ret_sig = tlib.get(&self.ret).ok_or(Error::tn(&self.ret))?;
        if ret_sig.kind != TypeKind::Derived {
            return Err(Error(format!(
                "return type {:?} in function signature {:?} must be derived",
                self.ret, self
            )));
        }
        self.condition.check(tlib, self)
    }

    fn vals(&self) -> IndexSet<Value> {
        self.condition.vals()
    }
}

/// Libraries of defined parameterized functions.
pub type FunctionLibrary = IndexMap<BaseFunction, FunctionSignature>;

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
        tlib: &TypeLibrary,
        flib: &FunctionLibrary,
        props: &Vec<Type>,
    ) -> Result<Type, Error> {
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
                for t in props {
                    vals.extend(t.args.values().cloned());
                }
                vals.extend(f.metadata.values().cloned());

                // Recursively infer values and check proper domain
                let mut ctx_args = IndexMap::new();
                for (fp, e) in args {
                    let tau = e.infer(tlib, flib, props)?;
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

                Ok(Type {
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
    tlib: TypeLibrary,
    flib: FunctionLibrary,
    props: Vec<Type>,
    goal: Type,
}

impl Problem {
    pub fn new(
        tlib: TypeLibrary,
        flib: FunctionLibrary,
        props: Vec<Type>,
        goal: Type,
    ) -> Result<Self, Error> {
        let ret = Self {
            tlib,
            flib,
            props,
            goal,
        };
        ret.check()?;
        Ok(ret)
    }

    fn check(&self) -> Result<(), Error> {
        for fs in self.flib.values() {
            fs.check(&self.tlib)?;
        }

        for t in &self.props {
            let sig = t.infer(&self.tlib)?;
            if sig.kind != TypeKind::Atomic {
                return Err(Error(format!(
                    "atomic proposition {:?} is not actually atomic",
                    t
                )));
            }
        }

        let goal_sig = self.goal.infer(&self.tlib)?;
        if goal_sig.kind != TypeKind::Derived {
            return Err(Error(format!(
                "goal type {:?} is not derived",
                self.goal
            )));
        }

        Ok(())
    }
}
