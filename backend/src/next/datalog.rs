use indexmap::IndexMap;
use indexmap::IndexSet;

pub type Error = String;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ValueType {
    Int,
    Str,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Value {
    Int(i64),
    Str(String),
    Var { name: String, typ: ValueType },
}

pub type Domain = IndexSet<Value>;

impl Value {
    fn check_domain(&self, dom: &Domain) -> Result<(), Error> {
        if !dom.contains(self) {
            return Err(format!("Value {:?} not in provided domain", self));
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Relation(String);

pub enum RelationKind {
    EDB,
    IDB,
}

pub struct RelationSignature {
    pub params: Vec<ValueType>,
    pub kind: RelationKind,
}

pub type RelationLibrary = IndexMap<Relation, RelationSignature>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fact {
    pub relation: Relation,
    pub args: Vec<Value>,
}

impl Fact {
    fn check(&self, lib: &RelationLibrary, dom: &Domain) -> Result<(), Error> {
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

        Ok(())
    }

    fn is_ground(&self) -> bool {
        self.args.iter().all(|v| v.is_ground())
    }

    fn is_abstract(&self) -> bool {
        self.args.iter().all(|v| v.is_abstract())
    }
}

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
            Predicate::Fact(f) => f.check(lib, dom),
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

pub struct Rule {
    name: String,
    head: Fact,
    body: Vec<Predicate>,
}

impl Rule {
    fn check(&self, lib: &RelationLibrary, dom: &Domain) -> Result<(), Error> {
        self.head.check(lib, dom)?;

        if !self.head.is_abstract() {
            return Err(format!("head of rule {} must be abstract", self.name));
        }

        for p in &self.body {
            p.check(lib, dom)?;
        }

        Ok(())
    }
}

pub struct Program {
    lib: RelationLibrary,
    dom: Domain,
    rules: Vec<Rule>,
    ground_facts: Vec<Fact>,
}

impl Program {
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
                return Err(format!("Value {:?} in domain is not ground", v));
            }
        }

        for r in &self.rules {
            r.check(&self.lib, &self.dom)?;
        }

        for f in &self.ground_facts {
            if !f.is_ground() {
                return Err(format!(
                    "Ground fact {:?} is not actually ground",
                    f
                ));
            }
            f.check(&self.lib, &self.dom)?;
        }

        Ok(())
    }
}

pub trait Engine {
    fn query(
        &self,
        program: Program,
        query_signature: RelationSignature,
        query_rule: Rule,
    ) -> Vec<Vec<Value>>;
}

pub trait Encode {
    fn encode(&self) -> Program;
}
