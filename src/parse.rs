use chumsky::prelude::*;

use crate::ir::*;

trait P<T>: Parser<char, T, Error = Simple<char>> {}
impl<S, T> P<T> for S where S: Parser<char, T, Error = Simple<char>> {}

pub fn ident_rest() -> impl P<String> {
    filter(|c| {
        char::is_ascii_lowercase(c) || char::is_ascii_uppercase(c) || char::is_ascii_digit(c)
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

pub fn fact_type() -> impl P<FactType> {
    fact_name()
        .padded()
        .then(value_type())
        .padded()
        .delimited_by(just('('), just(')'))
        .padded()
        .repeated()
        .delimited_by(just('('), just(')'))
        .map(|params| FactType { params })
}

pub fn fact_signature() -> impl P<FactSignature> {
    fact_kind()
        .padded()
        .then(fact_name())
        .padded()
        .then(fact_type())
        .padded()
        .delimited_by(just('('), just(')'))
        .map(|((kind, name), typ)| FactSignature { name, kind, typ })
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
        .padded()
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
        .map(
            |((((_, name), ret), params), precondition)| ComputationSignature {
                name,
                params,
                ret,
                precondition,
            },
        )
}

pub fn library() -> impl P<Library> {
    choice((
        fact_signature().map(Signature::Fact),
        computation_signature().map(Signature::Computation),
    ))
    .padded()
    .repeated()
    .padded()
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
