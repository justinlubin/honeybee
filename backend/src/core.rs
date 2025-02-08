//! # Honeybee core syntax
//!
//! This module defines the core syntax for Honeybee.

use crate::top_down::*;

use chumsky::prelude::*;
use indexmap::{IndexMap, IndexSet};
use serde::Deserialize;

////////////////////////////////////////////////////////////////////////////////
// Parsing

trait Parse: Sized {
    fn parser() -> impl Parser<char, Self, Error = Simple<char>>;
}

fn parse_error(
    title: &str,
    code: i32,
    src: &str,
    err: &Simple<char>,
) -> String {
    use ariadne::*;

    let err_span = err.span();
    let err_expected = err
        .expected()
        .filter_map(|mtok| mtok.map(|tok| format!("`{}`", tok)))
        .collect::<Vec<_>>();

    let error_color = Color::Red;

    let mut report =
        Report::build(ReportKind::Error, "expression", err_span.start)
            .with_code(code)
            .with_message(title)
            .with_label(
                Label::new(("expression", err_span))
                    .with_message(format!(
                        "{}",
                        "Unexpected token".fg(error_color),
                    ))
                    .with_color(error_color),
            );

    if !err_expected.is_empty() {
        report = report.with_note(format!(
            "{}{}",
            if err_expected.len() == 1 {
                format!("Expected {}", err_expected[0])
            } else {
                format!("Expected one of {}", err_expected.join(", "))
            },
            match err.found() {
                Some(tok) => format!(", but found `{}`", tok),
                None => "".to_owned(),
            }
        ));
    }

    let mut buf: Vec<u8> = vec![];
    report
        .finish()
        .write(sources(vec![("expression", src)]), &mut buf)
        .unwrap();
    String::from_utf8(buf).unwrap()
}

////////////////////////////////////////////////////////////////////////////////
// Errors

/// The type of type errors used by this module.
#[derive(Debug)]
pub struct LangError {
    pub context: Vec<String>,
    pub message: String,
    _private: (),
}

impl LangError {
    fn with_context(mut self, ctx: String) -> Self {
        self.context.push(ctx);
        self
    }

    fn new(message: String) -> Self {
        Self {
            context: vec![],
            message,
            _private: (),
        }
    }

    fn fp(fp: &FunParam) -> Self {
        Self::new(format!("unknown function parameter '{}'", fp.0))
    }

    fn mn(name: &MetName) -> Self {
        Self::new(format!("unknown metadata name '{}'", name.0))
    }

    fn mp(mp: &MetParam) -> Self {
        Self::new(format!("unknown metadata parameter '{}'", mp.0))
    }

    fn bf(bf: &BaseFunction) -> Self {
        Self::new(format!("unknown base function '{}'", bf.0))
    }

    fn argcount(got: usize, expected: usize) -> Self {
        Self::new(format!("got {} args, expected {}", got, expected))
    }
}

// impl std::fmt::Display for Error {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{}", self.0)
//     }
// }

////////////////////////////////////////////////////////////////////////////////
// Values

/// The types that values may take on.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub enum ValueType {
    Bool,
    Int,
    Str,
}

/// The possible values.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
#[serde(untagged)]
pub enum Value {
    Bool(bool),
    Int(i64),
    Str(String),
}

impl Value {
    pub fn infer(&self) -> ValueType {
        match self {
            Value::Bool(_) => ValueType::Bool,
            Value::Int(_) => ValueType::Int,
            Value::Str(_) => ValueType::Str,
        }
    }
}

impl Parse for Value {
    fn parser() -> impl Parser<char, Self, Error = Simple<char>> {
        choice((
            just("true").to(Self::Bool(true)),
            just("false").to(Self::Bool(false)),
            text::int(10).map(|s: String| Self::Int(s.parse().unwrap())),
            none_of("\"")
                .repeated()
                .collect()
                .delimited_by(just('"'), just('"'))
                .map(|s: String| Self::Str(s)),
        ))
    }
}

////////////////////////////////////////////////////////////////////////////////
// Types

/// The type of metadata-indexed tuple names.
///
/// Types and atomic propositions are named by this type. Consequently, type
/// names serve as the keys for type libraries.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
pub struct MetName(pub String);

/// The type of metadata-indexed tuple parameter keys.
///
/// Types, type signatures, and atomic proposition formulas contain maps indexed
/// by metadata parameters. Generally, the values of these maps will be things
/// that "look like" metadata values or value types.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
pub struct MetParam(pub String);

/// Signatures for metadata-indexed tuples that define their arity.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct MetSignature {
    pub params: IndexMap<MetParam, ValueType>,
}

/// Libraries of metadata-indexed tuples.
pub type MetLibrary = IndexMap<MetName, MetSignature>;

/// The type of metadata-indexed tuples.
///
/// This struct is used for atomic propositions and types
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Met<T> {
    pub name: MetName,
    pub args: IndexMap<MetParam, T>,
}

impl<T> Met<T> {
    fn context(&self) -> String {
        format!("metadata tuple '{}'", self.name.0).to_owned()
    }
}

impl Met<Value> {
    fn infer(&self, mlib: &MetLibrary) -> Result<MetSignature, LangError> {
        let sig = mlib
            .get(&self.name)
            .ok_or_else(|| LangError::mn(&self.name))?;

        if self.args.len() != sig.params.len() {
            return Err(LangError::argcount(self.args.len(), sig.params.len())
                .with_context(self.context()));
        }

        for (mp, v) in &self.args {
            let expected_vt = sig.params.get(mp).ok_or_else(|| {
                LangError::mp(mp).with_context(self.context())
            })?;
            let got_vt = v.infer();

            if got_vt != *expected_vt {
                return Err(LangError::new(format!(
                    "argument {:?} of {:?} is type {:?} but expected {:?}",
                    v, self.name, got_vt, expected_vt
                ))
                .with_context(self.context()));
            }
        }

        Ok(sig.clone())
    }
}

fn ident_rest() -> impl Parser<char, String, Error = Simple<char>> {
    filter(|c| {
        char::is_ascii_lowercase(c)
            || char::is_ascii_uppercase(c)
            || char::is_ascii_digit(c)
            || *c == '-'
            || *c == '_'
    })
    .repeated()
    .collect()
}

fn lower_ident() -> impl Parser<char, String, Error = Simple<char>> {
    filter(char::is_ascii_lowercase)
        .then(ident_rest())
        .map(|(first, rest)| format!("{}{}", first, rest))
}

fn upper_ident() -> impl Parser<char, String, Error = Simple<char>> {
    filter(char::is_ascii_uppercase)
        .then(ident_rest())
        .map(|(first, rest)| format!("{}{}", first, rest))
}

impl<T: Parse> Parse for Met<T> {
    fn parser() -> impl Parser<char, Self, Error = Simple<char>> {
        upper_ident()
            .then(
                (lower_ident()
                    .then(just("=").padded())
                    .then(T::parser())
                    .padded()
                    .map(|((lhs, _), rhs)| (MetParam(lhs), rhs)))
                .separated_by(just(','))
                .delimited_by(just('{'), just('}'))
                .padded(),
            )
            .padded()
            .map(|(name, args)| Met {
                name: MetName(name),
                args: args.into_iter().collect(),
            })
    }
}

////////////////////////////////////////////////////////////////////////////////
// Formulas

pub struct EvaluationContext<'a> {
    pub args: &'a IndexMap<FunParam, IndexMap<MetParam, Value>>,
    pub ret: &'a IndexMap<MetParam, Value>,
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
    pub fn infer(
        &self,
        types: &MetLibrary,
        fs: &FunctionSignature,
    ) -> Result<ValueType, LangError> {
        match self {
            FormulaAtom::Param(fp, mp) => {
                let name =
                    fs.params.get(fp).ok_or_else(|| LangError::fp(fp))?;

                types
                    .get(name)
                    .ok_or_else(|| LangError::mn(name))?
                    .params
                    .get(mp)
                    .ok_or_else(|| LangError::mp(mp))
                    .cloned()
            }
            FormulaAtom::Ret(mp) => types
                .get(&fs.ret)
                .ok_or_else(|| LangError::mn(&fs.ret))?
                .params
                .get(mp)
                .ok_or_else(|| LangError::mp(mp))
                .cloned(),
            FormulaAtom::Lit(v) => Ok(v.infer()),
        }
    }

    fn eval(&self, ctx: &EvaluationContext) -> Result<Value, LangError> {
        match self {
            FormulaAtom::Param(fp, mp) => ctx
                .args
                .get(fp)
                .ok_or_else(|| LangError::fp(fp))?
                .get(mp)
                .ok_or_else(|| LangError::mp(mp))
                .cloned(),
            FormulaAtom::Ret(mp) => {
                ctx.ret.get(mp).ok_or_else(|| LangError::mp(mp)).cloned()
            }
            FormulaAtom::Lit(v) => Ok(v.clone()),
        }
    }

    pub fn vals(&self) -> IndexSet<Value> {
        match self {
            FormulaAtom::Param(_, _) => IndexSet::new(),
            FormulaAtom::Ret(_) => IndexSet::new(),
            FormulaAtom::Lit(v) => IndexSet::from([v.clone()]),
        }
    }
}

impl Met<FormulaAtom> {
    fn check(
        &self,
        props: &MetLibrary,
        types: &MetLibrary,
        fs: &FunctionSignature,
    ) -> Result<(), LangError> {
        let sig = props
            .get(&self.name)
            .ok_or_else(|| LangError::mn(&self.name))?;

        if self.args.len() != sig.params.len() {
            return Err(LangError::argcount(self.args.len(), sig.params.len())
                .with_context(self.context()));
        }

        for (mp, fa) in &self.args {
            let expected_vt = sig.params.get(mp).ok_or_else(|| {
                LangError::mp(mp).with_context(self.context())
            })?;
            let got_vt = fa.infer(types, fs)?;

            if got_vt != *expected_vt {
                return Err(LangError::new(format!(
                    "argument {:?} of atomic proposition {:?} is type {:?} but expected {:?}",
                    fa,
                    self.name,
                    got_vt,
                    expected_vt
                )).with_context(self.context()));
            }
        }

        Ok(())
    }

    fn eval(&self, ctx: &EvaluationContext) -> Result<Met<Value>, LangError> {
        let mut args = IndexMap::new();
        for (mp, fa) in &self.args {
            args.insert(
                mp.clone(),
                fa.eval(ctx).map_err(|e| e.with_context(self.context()))?,
            );
        }
        Ok(Met {
            name: self.name.clone(),
            args,
        })
    }

    pub fn vals(&self) -> IndexSet<Value> {
        let mut ret = IndexSet::new();
        for fa in self.args.values() {
            ret.extend(fa.vals());
        }
        ret
    }
}

impl Parse for FormulaAtom {
    fn parser() -> impl Parser<char, Self, Error = Simple<char>> {
        choice((
            Value::parser().map(|v| Self::Lit(v)),
            just("ret.")
                .then(lower_ident())
                .padded()
                .map(|(_, mp)| Self::Ret(MetParam(mp))),
            lower_ident()
                .then(just('.'))
                .then(lower_ident())
                .padded()
                .map(|((fp, _), mp)| Self::Param(FunParam(fp), MetParam(mp))),
        ))
    }
}

/// The type of formulas.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(try_from = "Vec<String>")]
pub enum Formula {
    True,
    Eq(FormulaAtom, FormulaAtom),
    Lt(FormulaAtom, FormulaAtom),
    AtomicProposition(Met<FormulaAtom>),
    And(Box<Formula>, Box<Formula>),
}

impl Formula {
    pub fn conjunct(fs: impl Iterator<Item = Formula>) -> Formula {
        let mut phi = Self::True;
        for f in fs {
            phi = Self::And(Box::new(phi), Box::new(f))
        }
        phi
    }

    fn check_equal_types(
        types: &MetLibrary,
        fs: &FunctionSignature,
        fa1: &FormulaAtom,
        fa2: &FormulaAtom,
    ) -> Result<(), LangError> {
        let vt1 = fa1.infer(types, fs)?;
        let vt2 = fa2.infer(types, fs)?;
        if vt1 != vt2 {
            return Err(LangError::new(format!(
                "formula atom {:?} has different type ({:?}) than formula atom {:?} ({:?})",
                fa1, vt1, fa2, vt2
            )));
        }
        Ok(())
    }

    fn check(
        &self,
        props: &MetLibrary,
        types: &MetLibrary,
        fs: &FunctionSignature,
    ) -> Result<(), LangError> {
        match self {
            Formula::True => Ok(()),
            Formula::Eq(fa1, fa2) => {
                Self::check_equal_types(types, fs, fa1, fa2)
            }
            Formula::Lt(fa1, fa2) => {
                let vt1 = fa1.infer(types, fs)?;
                if vt1 != ValueType::Int {
                    return Err(LangError::new(format!(
                        "formula atom {:?} has type {:?}, expected Int",
                        fa1, vt1,
                    )));
                }
                Self::check_equal_types(types, fs, fa1, fa2)
            }
            Formula::AtomicProposition(ap) => ap.check(props, types, fs),
            Formula::And(phi1, phi2) => {
                phi1.check(props, types, fs)?;
                phi2.check(props, types, fs)
            }
        }
    }

    pub fn sat(
        &self,
        props: &Vec<Met<Value>>,
        ctx: &EvaluationContext,
    ) -> Result<bool, LangError> {
        match self {
            Formula::True => Ok(true),
            Formula::Eq(fa1, fa2) => Ok(fa1.eval(ctx)? == fa2.eval(ctx)?),
            Formula::Lt(fa1, fa2) => match (fa1.eval(ctx)?, fa2.eval(ctx)?) {
                (Value::Int(x1), Value::Int(x2)) => Ok(x1 < x2),
                (v1, v2) => Err(LangError::new(format!(
                    "Lt only supported for ints, got {:?} and {:?}",
                    v1, v2,
                ))),
            },
            Formula::AtomicProposition(ap) => {
                let prop = ap.eval(ctx)?;
                Ok(props.iter().any(|p| *p == prop))
            }
            Formula::And(phi1, phi2) => {
                Ok(phi1.sat(props, ctx)? && phi2.sat(props, ctx)?)
            }
        }
    }

    pub fn vals(&self) -> IndexSet<Value> {
        match self {
            Formula::True => IndexSet::new(),
            Formula::Eq(fa1, fa2) | Formula::Lt(fa1, fa2) => {
                let mut ret = fa1.vals();
                ret.extend(fa2.vals());
                ret
            }
            Formula::AtomicProposition(ap) => ap.vals(),
            Formula::And(phi1, phi2) => {
                let mut ret = phi1.vals();
                ret.extend(phi2.vals());
                ret
            }
        }
    }
}

impl Parse for Formula {
    fn parser() -> impl Parser<char, Self, Error = Simple<char>> {
        #[derive(Clone)]
        enum Op {
            Eq,
            Lt,
        }
        use Op::*;

        choice((
            FormulaAtom::parser()
                .then(choice((just('=').to(Eq), just('<').to(Lt))).padded())
                .then(FormulaAtom::parser())
                .map(|((left, op), right)| match op {
                    Eq => Self::Eq(left, right),
                    Lt => Self::Lt(left, right),
                }),
            Met::<FormulaAtom>::parser().map(|ap| Self::AtomicProposition(ap)),
        ))
    }
}

impl TryFrom<Vec<String>> for Formula {
    type Error = String;

    fn try_from(strings: Vec<String>) -> Result<Self, Self::Error> {
        let mut overall_phi = Self::True;
        for s in strings {
            match Self::parser().parse(s.clone()) {
                Ok(phi) => {
                    overall_phi =
                        Self::And(Box::new(overall_phi), Box::new(phi))
                }
                Err(errs) => {
                    return Err(parse_error(
                        "Formula parse error",
                        0,
                        &s,
                        &errs[0],
                    ))
                }
            }
        }
        Ok(overall_phi)
    }
}

////////////////////////////////////////////////////////////////////////////////
// Function signatures

/// The type of base function names.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
pub struct BaseFunction(pub String);

/// The type of signatures of parameterized functions.
///
/// The condition formula refers to the metadata values on the parameter types
/// and return type.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct FunctionSignature {
    pub params: IndexMap<FunParam, MetName>,
    pub ret: MetName,
    pub condition: Formula,
}

impl FunctionSignature {
    fn check(
        &self,
        props: &MetLibrary,
        types: &MetLibrary,
    ) -> Result<(), LangError> {
        for type_name in self.params.values() {
            let _ = types
                .get(type_name)
                .ok_or_else(|| LangError::mn(type_name))?;
        }
        let _ = types
            .get(&self.ret)
            .ok_or_else(|| LangError::mn(&self.ret))?;
        self.condition.check(props, types, self)
    }

    pub fn vals(&self) -> IndexSet<Value> {
        self.condition.vals()
    }
}

/// Libraries of defined parameterized functions.
pub type FunctionLibrary = IndexMap<BaseFunction, FunctionSignature>;

////////////////////////////////////////////////////////////////////////////////
// Composite libraries and programs

/// The libraries necessary for a Honeybee problem.
#[derive(Deserialize)]
pub struct Library {
    #[serde(rename = "Prop")]
    pub props: MetLibrary,
    #[serde(rename = "Type")]
    pub types: MetLibrary,
    #[serde(rename = "Function")]
    pub functions: FunctionLibrary,
}

impl Library {
    fn check(&self) -> Result<(), LangError> {
        let pnames: IndexSet<_> = self.props.keys().cloned().collect();
        let tnames: IndexSet<_> = self.types.keys().cloned().collect();
        let ambiguous_names: IndexSet<_> =
            pnames.intersection(&tnames).collect();

        if !ambiguous_names.is_empty() {
            return Err(LangError::new(format!(
                "ambiguous prop/type names: {:?}",
                ambiguous_names
            )));
        }

        for (f, fs) in self.functions.iter() {
            fs.check(&self.props, &self.types).map_err(|e| {
                e.with_context(format!("function signature '{}'", f.0))
            })?;
        }

        Ok(())
    }
}

/// The type of Honeybee programs.
#[derive(Deserialize)]
pub struct Program {
    #[serde(rename = "Prop")]
    pub props: Vec<Met<Value>>,
    #[serde(rename = "Goal")]
    pub goal: Met<Value>,
}

impl Program {
    fn check(&self, lib: &Library) -> Result<(), LangError> {
        for p in &self.props {
            let _ = p
                .infer(&lib.props)
                .map_err(|e| e.with_context("propositions".to_owned()))?;
        }

        let _ = self
            .goal
            .infer(&lib.types)
            .map_err(|e| e.with_context("goal".to_owned()))?;

        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////
// Parameterized functions and expressions

/// The type of parameterized functions.
///
/// Parameterized functions consist of a base name as well as metadata arguments
/// that correspond to the metadata values for its return type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParameterizedFunction {
    pub name: BaseFunction,
    pub metadata: IndexMap<MetParam, Value>,
    // A copy of the function's arity is stored along with the function so that
    // the impl Function block below can work without reference to a library.
    arity: Vec<FunParam>,
}

impl ParameterizedFunction {
    pub fn from_sig(
        sig: &FunctionSignature,
        name: BaseFunction,
        metadata: IndexMap<MetParam, Value>,
    ) -> Self {
        ParameterizedFunction {
            name: name.clone(),
            metadata,
            arity: sig.params.keys().cloned().collect(),
        }
    }

    pub fn new(
        flib: &FunctionLibrary,
        name: BaseFunction,
        metadata: IndexMap<MetParam, Value>,
    ) -> Result<Self, LangError> {
        Ok(Self::from_sig(
            flib.get(&name).ok_or_else(|| LangError::bf(&name))?,
            name,
            metadata,
        ))
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
        flib: &FunctionLibrary,
        props: &Vec<Met<Value>>,
    ) -> Result<Met<Value>, LangError> {
        match self {
            Sketch::Hole(_) => {
                Err(LangError::new(format!("holes are not well-typed")))
            }
            Sketch::App(f, args) => {
                // Get signature
                let sig =
                    flib.get(&f.name).ok_or_else(|| LangError::bf(&f.name))?;

                // Compute domain
                let mut vals = IndexSet::new();
                for fs in flib.values() {
                    vals.extend(fs.vals());
                }
                for p in props {
                    vals.extend(p.args.values().cloned());
                }
                vals.extend(f.metadata.values().cloned());

                // Recursively infer values and check proper domain
                let mut ctx_args = IndexMap::new();
                for (fp, e) in args {
                    let tau = e.infer(flib, props)?;
                    let metadata = tau.args;
                    for v in metadata.values() {
                        if !vals.contains(v) {
                            return Err(LangError::new(format!(
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
                    return Err(LangError::new(format!(
                        "condition {:?} not satisfied for {:?}",
                        sig.condition, self
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

////////////////////////////////////////////////////////////////////////////////
// Synthesis problem

/// The type of synthesis problems for Honeybee core syntax.
///
/// The problem requires the synthesis of an expression at the goal type
/// assuming the provided libraries and atomic propositions. See
/// [`Exp::well_typed`] for more information about what it means for an
/// expression to be well-typed.
pub struct Problem {
    pub library: Library,
    pub program: Program,
    _private: (),
}

impl Problem {
    pub fn new(library: Library, program: Program) -> Result<Self, LangError> {
        let ret = Self {
            library,
            program,
            _private: (),
        };
        ret.check()?;
        Ok(ret)
    }

    fn check(&self) -> Result<(), LangError> {
        self.library
            .check()
            .map_err(|e| e.with_context("library".to_owned()))?;
        self.program
            .check(&self.library)
            .map_err(|e| e.with_context("program".to_owned()))
    }

    pub fn vals(&self) -> IndexSet<Value> {
        let mut dom = IndexSet::new();
        for fs in self.library.functions.values() {
            dom.extend(fs.vals());
        }
        for p in &self.program.props {
            dom.extend(p.args.values().cloned());
        }
        dom.extend(self.program.goal.args.values().cloned());
        dom
    }
}

////////////////////////////////////////////////////////////////////////////////
// Goal convenience wrapper

pub struct Goal {
    function: BaseFunction,
    param: FunParam,
    ret: MetName,
    signature: FunctionSignature,
}

impl Goal {
    pub fn new(goal: &Met<Value>) -> Self {
        let function = BaseFunction("&goal".to_owned());
        let param = FunParam("&goalparam".to_owned());
        let ret = MetName("&Goal".to_owned());

        let signature = FunctionSignature {
            condition: Formula::conjunct(goal.args.iter().map(|(mp, v)| {
                Formula::Eq(
                    FormulaAtom::Param(param.clone(), mp.clone()),
                    FormulaAtom::Lit(v.clone()),
                )
            })),
            ret: ret.clone(),
            params: IndexMap::from([(param.clone(), goal.name.clone())]),
        };

        Self {
            function,
            param,
            ret,
            signature,
        }
    }

    pub fn add_to_library(&self, functions: &mut FunctionLibrary) {
        functions.insert(self.function.clone(), self.signature.clone());
    }

    pub fn app(
        &self,
        e: &Exp,
    ) -> (ParameterizedFunction, IndexMap<FunParam, Exp>) {
        (
            ParameterizedFunction::from_sig(
                &self.signature,
                self.function.clone(),
                IndexMap::new(),
            ),
            IndexMap::from([(self.param.clone(), e.clone())]),
        )
    }
}
