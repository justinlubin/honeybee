pub mod parse {
    use chumsky::prelude::*;

    use crate::ir::*;

    pub trait P<T>: Parser<char, T, Error = Simple<char>> {}
    impl<S, T> P<T> for S where S: Parser<char, T, Error = Simple<char>> {}

    pub fn ident_rest() -> impl P<String> {
        filter(|c| {
            char::is_ascii_lowercase(c)
                || char::is_ascii_uppercase(c)
                || char::is_ascii_digit(c)
        })
        .repeated()
        .collect()
    }

    pub fn lower_ident() -> impl P<String> {
        filter(char::is_ascii_lowercase)
            .then(ident_rest())
            .map(|(first, rest)| format!("{}{}", first, rest))
    }

    pub fn upper_ident() -> impl P<String> {
        filter(char::is_ascii_uppercase)
            .then(ident_rest())
            .map(|(first, rest)| format!("{}{}", first, rest))
    }

    pub fn value_type() -> impl P<ValueType> {
        choice((
            just("Int").to(ValueType::Int),
            just("Str").to(ValueType::Str),
        ))
    }

    pub fn value() -> impl P<Value> {
        choice((
            text::int(10).map(|s: String| Value::Int(s.parse().unwrap())),
            none_of("\"")
                .repeated()
                .collect()
                .delimited_by(just('"'), just('"'))
                .map(|s: String| Value::Str(s)),
        ))
    }

    pub fn fact_name() -> impl P<FactName> {
        upper_ident()
    }

    pub fn fact_kind() -> impl P<FactKind> {
        choice((
            just("annotation").to(FactKind::Annotation),
            just("analysis").to(FactKind::Analysis),
        ))
    }

    pub fn fact_signature() -> impl P<FactSignature> {
        fact_kind()
            .padded()
            .then(fact_name())
            .padded()
            .then(
                lower_ident()
                    .padded()
                    .then(value_type())
                    .padded()
                    .delimited_by(just('('), just(')'))
                    .padded()
                    .repeated(),
            )
            .delimited_by(just('('), just(')'))
            .map(|((kind, name), params)| FactSignature { name, kind, params })
    }

    pub fn fact() -> impl P<Fact> {
        fact_name()
            .padded()
            .then(
                lower_ident()
                    .padded()
                    .then(value())
                    .padded()
                    .delimited_by(just('('), just(')'))
                    .padded()
                    .repeated(),
            )
            .delimited_by(just('('), just(')'))
            .map(|(name, args)| Fact { name, args })
    }

    pub fn predicate_atom() -> impl P<PredicateAtom> {
        just('.')
            .ignored()
            .then(lower_ident())
            .padded()
            .then(lower_ident())
            .padded()
            .delimited_by(just('('), just(')'))
            .map(|((_, selector), arg)| PredicateAtom::Select { selector, arg })
    }

    pub fn predicate_relation() -> impl P<PredicateRelation> {
        one_of("=<")
            .padded()
            .then(predicate_atom())
            .padded()
            .then(predicate_atom())
            .padded()
            .delimited_by(just('('), just(')'))
            .map(|((op, lhs), rhs)| match op {
                '=' => PredicateRelation::Eq(lhs, rhs),
                '<' => PredicateRelation::Lt(lhs, rhs),
                _ => panic!("Unknown operator '{}'", op),
            })
    }

    pub fn predicate() -> impl P<Predicate> {
        predicate_relation()
            .padded()
            .repeated()
            .delimited_by(just('('), just(')'))
    }

    pub fn computation_signature() -> impl P<ComputationSignature> {
        just("computation")
            .ignored()
            .padded()
            .then(lower_ident())
            .padded()
            .then(fact_name())
            .padded()
            .then(
                lower_ident()
                    .padded()
                    .then(fact_name())
                    .padded()
                    .delimited_by(just('('), just(')'))
                    .padded()
                    .repeated()
                    .delimited_by(just('('), just(')')),
            )
            .padded()
            .then(predicate())
            .padded()
            .delimited_by(just('('), just(')'))
            .map(|((((_, name), ret), params), precondition)| {
                ComputationSignature {
                    name,
                    params,
                    ret,
                    precondition,
                }
            })
    }

    pub fn library() -> impl P<Library> {
        enum Sig {
            F(FactSignature),
            C(ComputationSignature),
        }
        choice((
            fact_signature().map(Sig::F),
            computation_signature().map(Sig::C),
        ))
        .padded()
        .repeated()
        .padded()
        .map(|sigs| {
            let mut fact_signatures = vec![];
            let mut computation_signatures = vec![];
            for sig in sigs {
                match sig {
                    Sig::F(fs) => fact_signatures.push(fs),
                    Sig::C(cs) => computation_signatures.push(cs),
                };
            }
            Library {
                fact_signatures,
                computation_signatures,
            }
        })
    }

    pub fn program() -> impl P<Program> {
        just("annotations")
            .ignored()
            .padded()
            .then(fact().padded().repeated())
            .delimited_by(just('('), just(')'))
            .padded()
            .then(
                just("goal")
                    .ignored()
                    .padded()
                    .then(fact().padded())
                    .delimited_by(just('('), just(')')),
            )
            .padded()
            .map(|((_, annotations), (_, goal))| Program { annotations, goal })
    }
}

pub mod unparse {
    use crate::ir::*;

    pub fn value_type(vt: &ValueType) -> String {
        match vt {
            ValueType::Int => "Int".to_owned(),
            ValueType::Str => "Str".to_owned(),
        }
    }

    pub fn value(v: &Value) -> String {
        match v {
            Value::Int(x) => format!("{}", x),
            Value::Str(s) => format!("\"{}\"", s),
        }
    }

    pub fn fact_kind(fk: &FactKind) -> String {
        match fk {
            FactKind::Annotation => "annotation".to_owned(),
            FactKind::Analysis => "analysis".to_owned(),
        }
    }

    pub fn fact_signature(fs: &FactSignature) -> String {
        format!(
            "({} {}\n  {})",
            fact_kind(&fs.kind),
            fs.name,
            fs.params
                .iter()
                .map(|(lhs, rhs)| format!("({} {})", lhs, value_type(rhs)))
                .collect::<Vec<String>>()
                .join(" ")
        )
    }

    pub fn fact(f: &Fact) -> String {
        format!(
            "({} {})",
            f.name,
            f.args
                .iter()
                .map(|(lhs, rhs)| format!("({} {})", lhs, value(rhs)))
                .collect::<Vec<String>>()
                .join(" ")
        )
    }

    pub fn predicate_atom(pa: &PredicateAtom) -> String {
        match pa {
            PredicateAtom::Select { selector, arg } => {
                format!("(.{} {})", selector, arg)
            }
            PredicateAtom::Const(v) => value(v),
        }
    }

    pub fn predicate_relation(pr: &PredicateRelation) -> String {
        match pr {
            PredicateRelation::Eq(lhs, rhs) => {
                format!("(= {} {})", predicate_atom(lhs), predicate_atom(rhs))
            }
            PredicateRelation::Lt(lhs, rhs) => {
                format!("(< {} {})", predicate_atom(lhs), predicate_atom(rhs))
            }
        }
    }

    pub fn predicate(p: &Predicate) -> String {
        format!(
            "({})",
            p.iter()
                .map(predicate_relation)
                .collect::<Vec<String>>()
                .join("\n   ")
        )
    }

    pub fn computation_signature(cs: &ComputationSignature) -> String {
        format!(
            "(computation {} {}\n  {}\n  {})",
            cs.name,
            cs.ret,
            cs.params
                .iter()
                .map(|(lhs, rhs)| format!("({} {})", lhs, rhs))
                .collect::<Vec<String>>()
                .join(" "),
            predicate(&cs.precondition),
        )
    }

    pub fn library(lib: &Library) -> String {
        lib.fact_signatures
            .iter()
            .map(fact_signature)
            .chain(lib.computation_signatures.iter().map(computation_signature))
            .collect::<Vec<String>>()
            .join("\n\n")
    }

    pub fn program(p: &Program) -> String {
        format!(
            "(annotations\n  {})\n\n(goal\n  {}))",
            p.annotations
                .iter()
                .map(fact)
                .collect::<Vec<String>>()
                .join("\n  "),
            fact(&p.goal)
        )
    }
}
