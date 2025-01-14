//! # Datalog interface
//!
//! This module defines a high-level interface for working with Datalog
//! programs. In particular, it defines the types and operations of Datalog
//! programs but does not define how they are executed, which is the job of a
//! Datalog engine.

use indexmap::IndexMap;
use indexmap::IndexSet;

/// The type of errors used by this module.
pub type Error = String;

/// The types that primitive values may take on.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ValueType {
    Int,
    Str,
}

/// The possible primitive values.
///
/// A value is considered *abstract* if it is a variable and *ground* otherwise.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Value {
    Int(i64),
    Str(String),
    Var { name: String, typ: ValueType },
}

/// The type of value domains.
///
/// This type is used to finitize the domain of possible values that program
/// variables may take on.
pub type Domain = IndexSet<Value>;

impl Value {
    fn check_domain(&self, dom: &Domain) -> Result<(), Error> {
        if !dom.contains(self) {
            return Err(format!("value {:?} not in provided domain", self));
        }
        Ok(())
    }

    fn infer(&self, dom: &Domain) -> Result<ValueType, Error> {
        match self {
            Value::Int(_) => {
                self.check_domain(dom)?;
                Ok(ValueType::Int)
            }
            Value::Str(_) => {
                self.check_domain(dom)?;
                Ok(ValueType::Str)
            }
            Value::Var { typ, .. } => Ok(typ.clone()),
        }
    }

    fn is_ground(&self) -> bool {
        match self {
            Value::Int(_) => true,
            Value::Str(_) => true,
            Value::Var { .. } => false,
        }
    }

    fn is_abstract(&self) -> bool {
        match self {
            Value::Int(_) => false,
            Value::Str(_) => false,
            Value::Var { .. } => true,
        }
    }
}

/// The type of relation "names".
///
/// For example, `R` would be the relation in the fact `R(1, 2)`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Relation(String);

/// The type of relation kinds; either extrinsic (EDB) or intrinsic (IDB).
///
/// EDBs are defined by data, and IDBs are defined by rules.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelationKind {
    EDB,
    IDB,
}

/// Signatures for relations that define their arity and kind.
pub struct RelationSignature {
    pub params: Vec<ValueType>,
    pub kind: RelationKind,
}

/// Libraries of defined relations.
pub type RelationLibrary = IndexMap<Relation, RelationSignature>;

/// The type of facts.
///
/// For example, `R(1, x, 2)` is a fact whose relation is `R` and args are `1`,
/// `x` and `2`.
///
/// A fact is considered *ground* if all its arguments are ground and *abstract*
/// if all its arguments are abstract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fact {
    pub relation: Relation,
    pub args: Vec<Value>,
}

impl Fact {
    fn infer<'a>(
        &'a self,
        lib: &'a RelationLibrary,
        dom: &'a Domain,
    ) -> Result<&'a RelationSignature, Error> {
        let sig = lib
            .get(&self.relation)
            .ok_or(format!("unknown relation {:?}", self.relation))?;

        if self.args.len() != sig.params.len() {
            return Err(format!(
                "got {} args, expected {}",
                self.args.len(),
                sig.params.len()
            ));
        }

        for (got_v, expected_vt) in self.args.iter().zip(sig.params.iter()) {
            let got_vt = got_v.infer(dom)?;
            if got_vt != *expected_vt {
                return Err(format!(
                    "argument {:?} of relation {:?} is type {:?} but expected {:?}",
                    got_v,
                    self.relation,
                    got_vt,
                    expected_vt
                ));
            }

            if !dom.contains(got_v) {
                return Err(format!(
                    "argument {:?} of relation {:?} not in domain",
                    got_v, self.relation,
                ));
            }
        }

        Ok(sig)
    }

    fn is_ground(&self) -> bool {
        self.args.iter().all(|v| v.is_ground())
    }

    fn is_abstract(&self) -> bool {
        self.args.iter().all(|v| v.is_abstract())
    }
}

/// The possible predicates for the right-hand side (antecedent) of a rule.
///
/// A predicate is either an abstract fact or a primitive such as built-in
/// equality.
pub enum Predicate {
    Fact(Fact),
    PrimEq(Value, Value),
    PrimLt(Value, Value),
}

impl Predicate {
    fn check_equal_types(
        dom: &Domain,
        v1: &Value,
        v2: &Value,
    ) -> Result<(), Error> {
        let vt1 = v1.infer(dom)?;
        let vt2 = v2.infer(dom)?;
        if vt1 != vt2 {
            return Err(format!(
                "value {:?} has different type ({:?}) than value {:?} ({:?})",
                v1, vt1, v2, vt2
            ));
        }
        Ok(())
    }

    fn check(&self, lib: &RelationLibrary, dom: &Domain) -> Result<(), Error> {
        match self {
            Predicate::Fact(f) => {
                if !f.is_abstract() {
                    return Err(format!(
                        "antecedent fact {:?} is not abstract",
                        f,
                    ));
                }
                let _ = f.infer(lib, dom)?;
                Ok(())
            }
            Predicate::PrimEq(v1, v2) => Self::check_equal_types(dom, v1, v2),
            Predicate::PrimLt(v1, v2) => {
                let vt1 = v1.infer(dom)?;
                if vt1 != ValueType::Int {
                    return Err(format!(
                        "value {:?} has type {:?}, expected Int",
                        v1, vt1,
                    ));
                }
                Self::check_equal_types(dom, v1, v2)
            }
        }
    }
}

/// The type of rules.
///
/// The head (or left-hand side) of a rule is its consequent. The body (or
/// right-hand side) of a rule is its antecedent. The body is represented as a
/// conjunction of predicates.
pub struct Rule {
    name: String,
    head: Fact,
    body: Vec<Predicate>,
}

impl Rule {
    fn check(&self, lib: &RelationLibrary, dom: &Domain) -> Result<(), Error> {
        let head_sig = self.head.infer(lib, dom)?;

        if head_sig.kind != RelationKind::IDB {
            return Err(format!("head of rule {} must be an IDB", self.name));
        }

        if !self.head.is_abstract() {
            return Err(format!("head of rule {} must be abstract", self.name));
        }

        for p in &self.body {
            p.check(lib, dom)?;
        }

        Ok(())
    }
}

/// The type of Datalog programs.
///
/// A Datalog program consists of:
/// - A library of relations that define their signatures
/// - A domain of possible primitive values that variables in the program may
///   take on
/// - A set of rules that define the inhabitance of IDB facts
/// - A set of ground facts that define the inhabitance of EDB facts
pub struct Program {
    lib: RelationLibrary,
    dom: Domain,
    rules: Vec<Rule>,
    ground_facts: Vec<Fact>,
}

impl Program {
    /// Constructs a new Datalog program
    pub fn new(
        lib: RelationLibrary,
        dom: Domain,
        rules: Vec<Rule>,
        ground_facts: Vec<Fact>,
    ) -> Result<Self, Error> {
        let ret = Self {
            lib,
            dom,
            rules,
            ground_facts,
        };
        ret.check()?;
        Ok(ret)
    }

    fn check(&self) -> Result<(), Error> {
        for v in &self.dom {
            if !v.is_ground() {
                return Err(format!("value {:?} in domain is not ground", v));
            }
        }

        for r in &self.rules {
            r.check(&self.lib, &self.dom)?;
        }

        for f in &self.ground_facts {
            let sig = f.infer(&self.lib, &self.dom)?;
            if sig.kind != RelationKind::EDB {
                return Err(format!("ground fact {:?} must be an EDB", f));
            }
            if !f.is_ground() {
                return Err(format!(
                    "ground fact {:?} is not actually ground",
                    f
                ));
            }
        }

        Ok(())
    }
}

/// The interface for Datalog engines.
pub trait Engine {
    fn query(
        &self,
        program: Program,
        query_signature: RelationSignature,
        query_rule: Rule,
    ) -> Vec<Vec<Value>>;
}
