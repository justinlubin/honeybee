// Formalizing validity

use crate::next::pbn::*;

use indexmap::IndexMap;

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

    fn argcount(got: usize, expected: usize) -> Self {
        Error(format!("got {} args, expected {}", got, expected))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MetParam(String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BaseFunction(String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypeName(String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValueType {
    Int,
    Str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeSignature {
    params: IndexMap<MetParam, ValueType>,
    kind: TypeKind,
}

pub type TypeLibrary = IndexMap<TypeName, TypeSignature>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeKind {
    Atomic,
    Derived,
}

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParameterizedFunction {
    pub name: BaseFunction,
    pub metadata: IndexMap<MetParam, Value>,
    arity: Vec<FunParam>,
}

impl Function for ParameterizedFunction {
    fn arity(&self) -> Vec<FunParam> {
        return self.arity.clone();
    }
}

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
                let name = &fs.params.get(fp).ok_or(Error::fp(fp))?.name;

                tlib.get(name)
                    .ok_or(Error::tn(name))?
                    .params
                    .get(mp)
                    .ok_or(Error::mp(mp))
                    .cloned()
            }
            FormulaAtom::Ret(mp) => tlib
                .get(&fs.ret.name)
                .ok_or(Error::tn(&fs.ret.name))?
                .params
                .get(mp)
                .ok_or(Error::mp(mp))
                .cloned(),
            FormulaAtom::Lit(v) => Ok(v.infer()),
        }
    }

    fn eval(&self, fs: &FunctionSignature) -> Result<Value, Error> {
        match self {
            FormulaAtom::Param(fp, mp) => fs
                .params
                .get(fp)
                .ok_or(Error::fp(fp))?
                .args
                .get(mp)
                .ok_or(Error::mp(mp))
                .cloned(),
            FormulaAtom::Ret(mp) => {
                fs.ret.args.get(mp).ok_or(Error::mp(mp)).cloned()
            }
            FormulaAtom::Lit(v) => Ok(v.clone()),
        }
    }
}

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
}

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
}

pub struct FunctionSignature {
    params: IndexMap<FunParam, Type>,
    ret: Type,
    condition: Formula,
}

impl FunctionSignature {
    fn check(&self, tlib: &TypeLibrary) -> Result<(), Error> {
        for t in self.params.values() {
            let _ = t.infer(tlib)?;
        }
        let _ = self.ret.infer(tlib)?;
        self.condition.check(tlib, self)
    }
}

pub type FunctionLibrary = IndexMap<BaseFunction, FunctionSignature>;

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
