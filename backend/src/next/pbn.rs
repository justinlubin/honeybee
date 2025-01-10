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

    pub struct Type(TypeName, IndexMap<MParam, Value>);

    pub struct AtomicProposition(PropName, IndexMap<MParam, Value>);

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

    pub struct FunctionType(IndexMap<super::FParam, Type>, Type, Formula);

    pub type FunctionLibrary = IndexMap<BaseFunction, FunctionType>;

    pub struct Problem(FunctionLibrary, Vec<AtomicProposition>, Type);
}

mod datalog {
    pub enum ValueType {
        Int,
        Str,
    }

    pub enum Value {
        Int(i64),
        Str(String),
        Var(String),
    }

    pub struct Relation(String);

    pub struct Declaration(Relation, Vec<ValueType>);

    pub struct Fact(Relation, Vec<Value>);

    pub enum Predicate {
        Fact(Fact),
        PrimEq(Value, Value),
        PrimLt(Value, Value),
    }

    pub struct Rule {
        name: String,
        head: Fact,
        body: Vec<Predicate>,
    }

    pub struct Program {
        domain: Vec<Value>,
        idb_decls: Vec<Declaration>,
        edb_decls: Vec<Declaration>,
        rules: Vec<Rule>,
        edbs: Vec<Fact>,
    }

    pub trait Engine {
        fn query(
            &self,
            program: Program,
            query_declaration: Declaration,
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
