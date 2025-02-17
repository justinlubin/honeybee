use crate::core::*;
use crate::top_down::FunParam;

use indexmap::IndexMap;

pub struct Context<'a> {
    pub props: &'a Vec<Met<Value>>,
    pub args: &'a IndexMap<FunParam, IndexMap<MetParam, Value>>,
    pub ret: &'a IndexMap<MetParam, Value>,
}

impl<'a> Context<'a> {
    fn formula_atom(&self, fa: &FormulaAtom) -> Value {
        match fa {
            FormulaAtom::Param(fp, mp) => {
                self.args.get(fp).unwrap().get(mp).unwrap().clone()
            }
            FormulaAtom::Ret(mp) => self.ret.get(mp).unwrap().clone(),
            FormulaAtom::Lit(v) => v.clone(),
        }
    }

    fn atomic_proposition_matches(
        &self,
        ap: &AtomicProposition,
        prop: &Met<Value>,
    ) -> bool {
        if ap.name != prop.name {
            return false;
        }
        if ap.args.len() != prop.args.len() {
            return false;
        }
        for (mp, ofa) in &ap.args {
            let v = match prop.args.get(mp) {
                Some(v) => v,
                None => return false,
            };
            let fa = match ofa {
                Some(fa) => fa,
                None => continue,
            };
            if self.formula_atom(fa) != *v {
                return false;
            }
        }
        true
    }

    pub fn sat(&self, phi: &Formula) -> bool {
        match phi {
            Formula::True => true,
            Formula::Eq(fa1, fa2) => {
                self.formula_atom(fa1) == self.formula_atom(fa2)
            }
            Formula::Lt(fa1, fa2) => {
                match (self.formula_atom(fa1), self.formula_atom(fa2)) {
                    (Value::Int(x1), Value::Int(x2)) => x1 < x2,
                    (v1, v2) => panic!(
                        "Lt only supported for ints, got {:?} and {:?}",
                        v1, v2,
                    ),
                }
            }
            Formula::Ap(ap) => {
                for prop in self.props {
                    if self.atomic_proposition_matches(ap, prop) {
                        return true;
                    }
                }
                false
            }
            Formula::And(phi1, phi2) => self.sat(phi1) && self.sat(phi2),
        }
    }
}
