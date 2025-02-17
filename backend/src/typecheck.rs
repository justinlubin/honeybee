use crate::core::*;
use crate::eval;
use crate::top_down::{FunParam, Sketch};

use indexmap::{IndexMap, IndexSet};

pub struct Context<'a>(pub &'a Problem);

#[derive(Debug)]
pub struct Error {
    pub context: Vec<String>,
    pub message: String,
    _private: (),
}

impl Error {
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

type Check = Result<(), Error>;
type Infer<T> = Result<T, Error>;

pub fn problem(problem: &Problem) -> Check {
    let context = Context(problem);

    context
        .check()
        .map_err(|e| e.with_context("library".to_owned()))?;

    context
        .check_program(&problem.program)
        .map_err(|e| e.with_context("program".to_owned()))
}

impl Context<'_> {
    fn check(&self) -> Check {
        let pnames: IndexSet<_> =
            self.0.library.props.keys().cloned().collect();
        let tnames: IndexSet<_> =
            self.0.library.types.keys().cloned().collect();
        let ambiguous_names: IndexSet<_> =
            pnames.intersection(&tnames).collect();

        if !ambiguous_names.is_empty() {
            return Err(Error::new(format!(
                "ambiguous prop/type names: {:?}",
                ambiguous_names
            )));
        }

        for (f, fs) in &self.0.library.functions {
            self.check_function_signature(fs).map_err(|e| {
                e.with_context(format!("function signature '{}'", f.0))
            })?;
        }

        Ok(())
    }

    fn check_program(&self, program: &Program) -> Check {
        for p in &program.props {
            let _ = self
                .infer_proposition(p)
                .map_err(|e| e.with_context("propositions".to_owned()))?;
        }

        let _ = self
            .infer_type(&program.goal)
            .map_err(|e| e.with_context("goal".to_owned()))?;

        Ok(())
    }

    fn check_function_signature(&self, fs: &FunctionSignature) -> Check {
        for type_name in fs.params.values() {
            let _ = self
                .0
                .library
                .types
                .get(type_name)
                .ok_or_else(|| Error::mn(type_name))?;
        }
        let _ = self
            .0
            .library
            .types
            .get(&fs.ret)
            .ok_or_else(|| Error::mn(&fs.ret))?;
        self.check_formula(fs, &fs.condition)
    }

    fn check_formula(&self, fs: &FunctionSignature, phi: &Formula) -> Check {
        match phi {
            Formula::True => Ok(()),
            Formula::Eq(fa1, fa2) => {
                self.check_formula_atom_types_equal(fs, fa1, fa2)
            }
            Formula::Lt(fa1, fa2) => {
                let vt1 = self.infer_formula_atom(fs, fa1)?;
                if vt1 != ValueType::Int {
                    return Err(Error::new(format!(
                        "formula atom {:?} has type {:?}, expected Int",
                        fa1, vt1,
                    )));
                }
                self.check_formula_atom_types_equal(fs, fa1, fa2)
            }
            Formula::Ap(ap) => self.check_atomic_proposition(fs, ap),
            Formula::And(phi1, phi2) => {
                self.check_formula(fs, phi1)?;
                self.check_formula(fs, phi2)
            }
        }
    }

    fn check_formula_atom_types_equal(
        &self,
        fs: &FunctionSignature,
        fa1: &FormulaAtom,
        fa2: &FormulaAtom,
    ) -> Result<(), Error> {
        let vt1 = self.infer_formula_atom(fs, fa1)?;
        let vt2 = self.infer_formula_atom(fs, fa2)?;
        if vt1 != vt2 {
            return Err(Error::new(format!(
                "formula atom {:?} has different type ({:?}) than formula atom {:?} ({:?})",
                fa1, vt1, fa2, vt2
            )));
        }
        Ok(())
    }

    pub fn check_atomic_proposition(
        &self,
        fs: &FunctionSignature,
        ap: &AtomicProposition,
    ) -> Check {
        let sig = self
            .0
            .library
            .props
            .get(&ap.name)
            .ok_or_else(|| Error::mn(&ap.name))?;

        if ap.args.len() != sig.params.len() {
            return Err(Error::argcount(ap.args.len(), sig.params.len())
                .with_context(ap.context()));
        }

        for (mp, ofa) in &ap.args {
            let expected_vt = sig
                .params
                .get(mp)
                .ok_or_else(|| Error::mp(mp).with_context(ap.context()))?;

            let fa = match ofa {
                Some(fa) => fa,
                None => continue,
            };

            let got_vt = self.infer_formula_atom(fs, fa)?;

            if got_vt != *expected_vt {
                return Err(Error::new(format!(
                    "argument {:?} of atomic proposition {:?} is type {:?} but expected {:?}",
                    fa,
                    ap.name,
                    got_vt,
                    expected_vt
                )).with_context(ap.context()));
            }
        }

        Ok(())
    }

    pub fn infer_formula_atom(
        &self,
        fs: &FunctionSignature,
        fa: &FormulaAtom,
    ) -> Infer<ValueType> {
        match fa {
            FormulaAtom::Param(fp, mp) => {
                let name = fs.params.get(fp).ok_or_else(|| Error::fp(fp))?;

                self.0
                    .library
                    .types
                    .get(name)
                    .ok_or_else(|| Error::mn(name))?
                    .params
                    .get(mp)
                    .ok_or_else(|| Error::mp(mp))
                    .cloned()
            }
            FormulaAtom::Ret(mp) => self
                .0
                .library
                .types
                .get(&fs.ret)
                .ok_or_else(|| Error::mn(&fs.ret))?
                .params
                .get(mp)
                .ok_or_else(|| Error::mp(mp))
                .cloned(),
            FormulaAtom::Lit(v) => Ok(self.infer_value(v)),
        }
    }

    fn infer_met(
        &self,
        mlib: &MetLibrary,
        met: &Met<Value>,
    ) -> Infer<MetSignature> {
        let sig = mlib.get(&met.name).ok_or_else(|| Error::mn(&met.name))?;

        if met.args.len() != sig.params.len() {
            return Err(Error::argcount(met.args.len(), sig.params.len())
                .with_context(met.context()));
        }

        for (mp, v) in &met.args {
            let expected_vt = sig
                .params
                .get(mp)
                .ok_or_else(|| Error::mp(mp).with_context(met.context()))?;
            let got_vt = self.infer_value(v);

            if got_vt != *expected_vt {
                return Err(Error::new(format!(
                    "argument {:?} of {:?} is type {:?} but expected {:?}",
                    v, met.name, got_vt, expected_vt
                ))
                .with_context(met.context()));
            }
        }

        Ok(sig.clone())
    }

    pub fn infer_proposition(&self, met: &Met<Value>) -> Infer<MetSignature> {
        self.infer_met(&self.0.library.props, met)
    }

    pub fn infer_type(&self, met: &Met<Value>) -> Infer<MetSignature> {
        self.infer_met(&self.0.library.types, met)
    }

    pub fn infer_value(&self, v: &Value) -> ValueType {
        match v {
            Value::Bool(_) => ValueType::Bool,
            Value::Int(_) => ValueType::Int,
            Value::Str(_) => ValueType::Str,
        }
    }

    /// The typing relation for Honeybee core syntax.
    ///
    /// Holes are not well-typed. Function applications are well-typed if their
    /// arguments are well-typed and have metadata satisfying the validity
    /// condition of the function (and the metadata is contained within the
    /// allowable domain defined by the presence of values in the libraries).
    pub fn infer_exp(&self, e: &Exp) -> Infer<Met<Value>> {
        match e {
            Sketch::Hole(_) => {
                Err(Error::new("holes are not well-typed".to_string()))
            }
            Sketch::App(f, args) => {
                // Get signature
                let sig = self
                    .0
                    .library
                    .functions
                    .get(&f.name)
                    .ok_or_else(|| Error::bf(&f.name))?;

                // Compute domain
                let mut vals = IndexSet::new();
                for fs in self.0.library.functions.values() {
                    vals.extend(fs.vals());
                }
                for p in &self.0.program.props {
                    vals.extend(p.args.values().cloned());
                }
                vals.extend(f.metadata.values().cloned());

                // Recursively infer values and check proper domain
                let mut ctx_args = IndexMap::new();
                for (fp, arg) in args {
                    let tau = self.infer_exp(arg)?;
                    let metadata = tau.args;
                    for v in metadata.values() {
                        if !vals.contains(v) {
                            return Err(Error::new(format!(
                                "value {:?} not in domain",
                                v
                            )));
                        }
                    }
                    ctx_args.insert(fp.clone(), metadata);
                }

                // Check condition
                let ctx = eval::Context {
                    props: &self.0.program.props,
                    args: &ctx_args,
                    ret: &f.metadata,
                };
                if !ctx.sat(&sig.condition) {
                    return Err(Error::new(format!(
                        "condition {:?} not satisfied for {:?}",
                        sig.condition, e
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
