use indexmap::IndexMap;

// Programming By Navigation

pub trait Step {
    type Exp;
    fn step(&self, e: &Self::Exp) -> Option<Self::Exp>;
}

pub trait StepProvider {
    type Step: Step;
    fn provide(&self, e: &<Self::Step as Step>::Exp) -> Vec<Self::Step>;
}

// Top-Down Steps

pub type HoleName = usize;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FParam(String);

pub trait Function: Clone + Eq {
    fn arity(&self) -> Vec<FParam>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Sketch<F: Function> {
    Hole(HoleName),
    App(F, IndexMap<FParam, Self>),
}

pub enum TDStep<F: Function> {
    Extend(HoleName, F, IndexMap<FParam, Sketch<F>>),
    Seq(Box<Self>, Box<Self>),
}

impl<F: Function> Sketch<F> {
    pub fn has_subterm(&self, e: &Self) -> bool {
        if self == e {
            return true;
        }
        match self {
            Self::Hole(_) => false,
            Self::App(_, args) => args.values().any(|v| v.has_subterm(e)),
        }
    }

    pub fn substitute(&self, h: HoleName, e: &Self) -> Self {
        match self {
            Self::Hole(h2) => {
                if *h2 == h {
                    e.clone()
                } else {
                    Self::Hole(*h2)
                }
            }
            Self::App(f, args) => Self::App(
                f.clone(),
                args.iter()
                    .map(|(k, v)| (k.clone(), v.substitute(h, e)))
                    .collect(),
            ),
        }
    }

    fn max_hole(&self) -> HoleName {
        match self {
            Self::Hole(h) => *h,
            Self::App(_, args) => {
                args.values().map(|v| v.max_hole()).max().unwrap_or(0)
            }
        }
    }

    pub fn fresh(&self) -> impl Iterator<Item = HoleName> {
        return (self.max_hole() + 1)..;
    }
}

impl<F: Function> Step for TDStep<F> {
    type Exp = Sketch<F>;

    fn step(&self, e: &Self::Exp) -> Option<Self::Exp> {
        match self {
            Self::Extend(h, f, args) => {
                if f.arity().len() == args.len()
                    && e.has_subterm(&Self::Exp::Hole(*h))
                {
                    Some(e.substitute(
                        *h,
                        &Self::Exp::App(f.clone(), args.clone()),
                    ))
                } else {
                    None
                }
            }
            Self::Seq(s1, s2) => s1.step(e).and_then(|e2| s2.step(&e2)),
        }
    }
}

pub trait InhabitationOracle {
    type F: Function;
    fn expansions(&self, e: &Sketch<Self::F>) -> Vec<(HoleName, Self::F)>;
}

struct TDCCSynthesis<O: InhabitationOracle> {
    oracle: O,
}

impl<O: InhabitationOracle> StepProvider for TDCCSynthesis<O> {
    type Step = TDStep<O::F>;
    fn provide(&self, e: &<Self::Step as Step>::Exp) -> Vec<Self::Step> {
        let mut ret = vec![];
        for (h, f) in self.oracle.expansions(e) {
            let holes = e.fresh().map(|h| Sketch::Hole(h));
            ret.push(TDStep::Extend(
                h,
                f.clone(),
                f.arity().into_iter().zip(holes).collect(),
            ));
        }
        ret
    }
}

// Formalizing validity

pub mod core {
    use indexmap::IndexMap;

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct MParam(String);

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct BaseFunction(String);

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct TypeName(String);

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct PropName(String);

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

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Function {
        pub name: BaseFunction,
        pub metadata: IndexMap<MParam, Value>,
        arity: Vec<super::FParam>,
    }

    pub struct Type {
        name: TypeName,
        args: IndexMap<MParam, Value>,
    }

    pub struct AtomicProposition {
        name: PropName,
        args: IndexMap<MParam, Value>,
    }

    impl super::Function for Function {
        fn arity(&self) -> Vec<super::FParam> {
            return self.arity.clone();
        }
    }

    pub enum FormulaAtom {
        Param(super::FParam, MParam),
        Ret,
        Lit(Value),
    }

    pub enum Formula {
        True,
        Eq(FormulaAtom, FormulaAtom),
        Lt(FormulaAtom, FormulaAtom),
        AtomProp(PropName, IndexMap<MParam, FormulaAtom>),
        And(Box<Formula>, Box<Formula>),
    }

    pub struct FunctionSignature {
        params: IndexMap<super::FParam, Type>,
        ret: Type,
        condition: Formula,
    }

    pub type FunctionLibrary = IndexMap<BaseFunction, FunctionSignature>;

    pub struct Problem {
        library: FunctionLibrary,
        props: Vec<AtomicProposition>,
        goal: Type,
    }
}

mod datalog {
    use indexmap::IndexMap;
    use indexmap::IndexSet;

    #[derive(Debug, Clone, PartialEq, Eq)]
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
        fn check_domain(&self, dom: &Domain) -> Result<(), String> {
            if !dom.contains(self) {
                return Err();
            }
        }

        pub fn infer(&self, dom: &Domain) -> ValueType {
            match self {
                Value::Int(_) => ValueType::Int,
                Value::Str(_) => ValueType::Str,
                Value::Var { typ, .. } => typ.clone(),
            }
        }

        pub fn is_ground(&self) -> bool {
            match self {
                Value::Int(_) => true,
                Value::Str(_) => true,
                Value::Var { .. } => false,
            }
        }

        pub fn is_abstract(&self) -> bool {
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
        params: Vec<ValueType>,
        kind: RelationKind,
    }

    pub type RelationLibrary = IndexMap<Relation, RelationSignature>;

    pub struct Fact {
        relation: Relation,
        args: Vec<Value>,
    }

    impl Fact {
        pub fn check(
            &self,
            lib: &RelationLibrary,
            dom: &Domain,
        ) -> Result<(), String> {
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

            for (got_v, expected_vt) in self.args.iter().zip(sig.params.iter())
            {
                let got_vt = got_v.infer();
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

        pub fn is_ground(&self) -> bool {
            self.args.iter().all(|v| v.is_ground())
        }

        pub fn is_abstract(&self) -> bool {
            self.args.iter().all(|v| v.is_abstract())
        }
    }

    pub enum Predicate {
        Fact(Fact),
        PrimEq(Value, Value),
        PrimLt(Value, Value),
    }

    impl Predicate {
        fn check_equal_types(v1: &Value, v2: &Value) -> Result<(), String> {
            let vt1 = v1.infer();
            let vt2 = v2.infer();
            if vt1 != vt2 {
                return Err(
                    format!(
                        "value {:?} has different type ({:?}) than value {:?} ({:?})",
                        v1, vt1, v2, vt2
                    )
                );
            }
            Ok(())
        }

        pub fn check(&self, lib: &RelationLibrary) -> Result<(), String> {
            match self {
                Predicate::Fact(f) => f.check(lib),
                Predicate::PrimEq(v1, v2) => Self::check_equal_types(v1, v2),
                Predicate::PrimLt(v1, v2) => {
                    let vt1 = v1.infer();
                    if vt1 != ValueType::Int {
                        return Err(format!(
                            "value {:?} has type {:?}, expected Int",
                            v1, vt1,
                        ));
                    }
                    Self::check_equal_types(v1, v2)
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
        fn check(&self, lib: &RelationLibrary) -> Result<(), String> {
            self.head.check(lib)?;

            if !self.head.is_abstract() {
                return Err(format!(
                    "head of rule {} must be abstract",
                    self.name
                ));
            }

            for p in &self.body {
                p.check(lib)?;
            }

            Ok(())
        }
    }

    pub struct Program {
        domain: Vec<Value>,
        lib: RelationLibrary,
        rules: Vec<Rule>,
        ground_facts: Vec<Fact>,
    }

    impl Program {
        fn check(&self) -> Result<(), String> {}
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
}

////// Interaction

// All of the below should be called by JS for interactive or a main function
// for CLI

pub trait Interact<Spec, S: Step> {
    fn init(&self, spec: Spec) -> bool;
    fn provide(&self) -> Vec<S>;
    fn decide(&self, step: S);
    fn working_expression(&self) -> S::Exp;
    fn valid(&self) -> bool;
}
