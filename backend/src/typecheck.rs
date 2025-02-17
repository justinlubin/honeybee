use crate::core::*;
use indexmap::IndexSet;

pub struct Context<'a> {
    pub library: &'a Library,
}

type Check = Result<(), LangError>;
type Infer<T> = Result<T, LangError>;

pub fn problem(problem: &Problem) -> Check {
    let context = Context {
        library: &problem.library,
    };

    context
        .check()
        .map_err(|e| e.with_context("library".to_owned()))?;

    context
        .check_program(&problem.program)
        .map_err(|e| e.with_context("program".to_owned()))
}

impl<'a> Context<'a> {
    fn check(&self) -> Check {
        let pnames: IndexSet<_> = self.library.props.keys().cloned().collect();
        let tnames: IndexSet<_> = self.library.types.keys().cloned().collect();
        let ambiguous_names: IndexSet<_> =
            pnames.intersection(&tnames).collect();

        if !ambiguous_names.is_empty() {
            return Err(LangError::new(format!(
                "ambiguous prop/type names: {:?}",
                ambiguous_names
            )));
        }

        for (f, fs) in &self.library.functions {
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
                .library
                .types
                .get(type_name)
                .ok_or_else(|| LangError::mn(type_name))?;
        }
        let _ = self
            .library
            .types
            .get(&fs.ret)
            .ok_or_else(|| LangError::mn(&fs.ret))?;
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
                    return Err(LangError::new(format!(
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
    ) -> Result<(), LangError> {
        let vt1 = self.infer_formula_atom(fs, fa1)?;
        let vt2 = self.infer_formula_atom(fs, fa2)?;
        if vt1 != vt2 {
            return Err(LangError::new(format!(
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
            .library
            .props
            .get(&ap.name)
            .ok_or_else(|| LangError::mn(&ap.name))?;

        if ap.args.len() != sig.params.len() {
            return Err(LangError::argcount(ap.args.len(), sig.params.len())
                .with_context(ap.context()));
        }

        for (mp, ofa) in &ap.args {
            let expected_vt = sig
                .params
                .get(mp)
                .ok_or_else(|| LangError::mp(mp).with_context(ap.context()))?;

            let fa = match ofa {
                Some(fa) => fa,
                None => continue,
            };

            let got_vt = self.infer_formula_atom(fs, fa)?;

            if got_vt != *expected_vt {
                return Err(LangError::new(format!(
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
                let name =
                    fs.params.get(fp).ok_or_else(|| LangError::fp(fp))?;

                self.library
                    .types
                    .get(name)
                    .ok_or_else(|| LangError::mn(name))?
                    .params
                    .get(mp)
                    .ok_or_else(|| LangError::mp(mp))
                    .cloned()
            }
            FormulaAtom::Ret(mp) => self
                .library
                .types
                .get(&fs.ret)
                .ok_or_else(|| LangError::mn(&fs.ret))?
                .params
                .get(mp)
                .ok_or_else(|| LangError::mp(mp))
                .cloned(),
            FormulaAtom::Lit(v) => Ok(self.infer_value(v)),
        }
    }

    fn infer_met(
        &self,
        mlib: &MetLibrary,
        met: &Met<Value>,
    ) -> Infer<MetSignature> {
        let sig = mlib
            .get(&met.name)
            .ok_or_else(|| LangError::mn(&met.name))?;

        if met.args.len() != sig.params.len() {
            return Err(LangError::argcount(met.args.len(), sig.params.len())
                .with_context(met.context()));
        }

        for (mp, v) in &met.args {
            let expected_vt = sig
                .params
                .get(mp)
                .ok_or_else(|| LangError::mp(mp).with_context(met.context()))?;
            let got_vt = self.infer_value(v);

            if got_vt != *expected_vt {
                return Err(LangError::new(format!(
                    "argument {:?} of {:?} is type {:?} but expected {:?}",
                    v, met.name, got_vt, expected_vt
                ))
                .with_context(met.context()));
            }
        }

        Ok(sig.clone())
    }

    pub fn infer_proposition(&self, met: &Met<Value>) -> Infer<MetSignature> {
        self.infer_met(&self.library.props, met)
    }

    pub fn infer_type(&self, met: &Met<Value>) -> Infer<MetSignature> {
        self.infer_met(&self.library.types, met)
    }

    pub fn infer_value(&self, v: &Value) -> ValueType {
        match v {
            Value::Bool(_) => ValueType::Bool,
            Value::Int(_) => ValueType::Int,
            Value::Str(_) => ValueType::Str,
        }
    }
}
