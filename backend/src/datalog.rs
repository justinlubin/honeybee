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
    Bool,
    Int,
    Str,
}

/// The possible primitive values.
///
/// A value is considered *abstract* if it is a variable and *ground* otherwise.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Value {
    Bool(bool),
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
        match self {
            Value::Bool(_) | Value::Var { .. } => return Ok(()),
            Value::Int(_) | Value::Str(_) => (),
        }
        if !dom.contains(self) {
            return Err(format!("value {:?} not in provided domain", self));
        }
        Ok(())
    }

    pub fn unsafe_infer(&self) -> ValueType {
        match self {
            Value::Bool(_) => ValueType::Bool,
            Value::Int(_) => ValueType::Int,
            Value::Str(_) => ValueType::Str,
            Value::Var { typ, .. } => typ.clone(),
        }
    }

    pub fn infer(&self, dom: &Domain) -> Result<ValueType, Error> {
        self.check_domain(dom)?;
        Ok(self.unsafe_infer())
    }

    fn is_ground(&self) -> bool {
        match self {
            Value::Bool(_) => true,
            Value::Int(_) => true,
            Value::Str(_) => true,
            Value::Var { .. } => false,
        }
    }

    fn is_abstract(&self) -> bool {
        match self {
            Value::Bool(_) => false,
            Value::Int(_) => false,
            Value::Str(_) => false,
            Value::Var { .. } => true,
        }
    }

    fn prefix_vars(&self, prefix: &str) -> Self {
        match self {
            Value::Bool(_) | Value::Int(_) | Value::Str(_) => self.clone(),
            Value::Var { name, typ } => Self::Var {
                name: format!("{}{}", prefix, name),
                typ: typ.clone(),
            },
        }
    }
}

/// The type of relation "names".
///
/// For example, `R` would be the relation in the fact `R(1, 2)`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Relation(pub String);

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
    pub args: Vec<Option<Value>>,
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

        for (ogot_v, expected_vt) in self.args.iter().zip(sig.params.iter()) {
            let got_v = match ogot_v {
                Some(got_v) => got_v,
                None => continue,
            };
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
        }

        Ok(sig)
    }

    fn is_ground(&self) -> bool {
        self.args.iter().all(|ov| match ov {
            Some(v) => v.is_ground(),
            None => false,
        })
    }

    fn is_abstract(&self) -> bool {
        self.args.iter().all(|ov| match ov {
            Some(v) => v.is_abstract(),
            None => false,
        })
    }

    fn prefix_vars(&self, prefix: &str) -> Self {
        Self {
            relation: self.relation.clone(),
            args: self
                .args
                .iter()
                .map(|ov| ov.as_ref().map(|v| v.prefix_vars(prefix)))
                .collect(),
        }
    }

    fn vals(&self) -> IndexSet<Value> {
        self.args.iter().filter_map(|ov| ov.clone()).collect()
    }
}

/// The possible predicates for the right-hand side (antecedent) of a rule.
///
/// A predicate is either an abstract fact or a primitive such as built-in
/// equality.
#[derive(Debug, Clone)]
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

    fn prefix_vars(&self, prefix: &str) -> Self {
        match self {
            Predicate::Fact(f) => Predicate::Fact(f.prefix_vars(prefix)),
            Predicate::PrimEq(left, right) => Predicate::PrimEq(
                left.prefix_vars(prefix),
                right.prefix_vars(prefix),
            ),
            Predicate::PrimLt(left, right) => Predicate::PrimLt(
                left.prefix_vars(prefix),
                right.prefix_vars(prefix),
            ),
        }
    }

    fn vals(&self) -> IndexSet<Value> {
        match self {
            Predicate::Fact(f) => f.vals(),
            Predicate::PrimEq(left, right) => {
                IndexSet::from([left.clone(), right.clone()])
            }
            Predicate::PrimLt(left, right) => {
                IndexSet::from([left.clone(), right.clone()])
            }
        }
    }
}

/// The type of rules.
///
/// The head (or left-hand side) of a rule is its consequent. The body (or
/// right-hand side) of a rule is its antecedent. The body is represented as a
/// conjunction of predicates.
#[derive(Debug, Clone)]
pub struct Rule {
    pub name: String,
    pub head: Fact,
    pub body: Vec<Predicate>,
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

    const X_PREFIX: &'static str = "&x_";
    const Y_PREFIX: &'static str = "&y_";

    pub fn cut(&self, other: &Rule, j: usize) -> Option<Rule> {
        let mut self_body_without_cut_fact = self.body.clone();
        let self_cut_fact = match self_body_without_cut_fact.remove(j) {
            Predicate::Fact(f) => f,
            Predicate::PrimEq(_, _) | Predicate::PrimLt(_, _) => {
                log::debug!(
                    "Can't cut {}/{}/{} because predicate {} of {} is a primitive",
                    other.name,
                    j,
                    self.name,
                    j,
                    self.name,
                );
                return None;
            }
        };
        if self_cut_fact.relation != other.head.relation {
            return None;
        }

        let link = self_cut_fact
            .args
            .into_iter()
            .zip(other.head.args.iter())
            .filter_map(|(oy, ox)| {
                let y = match oy {
                    Some(y) => y,
                    None => return None,
                };

                // Heads must be abstract
                let x = ox.as_ref().unwrap();

                Some(Predicate::PrimEq(
                    y.prefix_vars(Self::Y_PREFIX),
                    x.prefix_vars(Self::X_PREFIX),
                ))
            });

        Some(Rule {
            name: format!("&cut_{}/{}", other.name, self.name),
            head: self.head.prefix_vars(Self::Y_PREFIX),
            body: self_body_without_cut_fact
                .iter()
                .map(|p| p.prefix_vars(Self::Y_PREFIX))
                .chain(other.body.iter().map(|p| p.prefix_vars(Self::X_PREFIX)))
                .chain(link)
                .collect(),
        })
    }

    pub fn vals(&self) -> IndexSet<Value> {
        let mut ret = self.head.vals();
        for f in &self.body {
            ret.extend(f.vals())
        }
        ret
    }
}

/// The type of Datalog programs.
///
/// A Datalog program consists of:
/// - A library of relations that define their signatures
/// - A domain of possible primitive values that variables in the program may
///   take on
/// - A set of rules that define the inhabitation of IDB facts
/// - A set of ground facts that define the inhabitation of EDB facts
pub struct Program {
    pub lib: RelationLibrary,
    pub dom: Domain,
    pub rules: Vec<Rule>,
    pub ground_facts: Vec<Fact>,
    _private: (),
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
            _private: (),
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
    fn load(&mut self, program: Program);

    fn query(
        &mut self,
        signature: &RelationSignature,
        rule: &Rule,
    ) -> Vec<Vec<Value>>;
}
