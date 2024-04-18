use chumsky::prelude::*;

mod value {
    #[derive(Debug, Clone)]
    pub enum Type {
        Int,
        Str,
    }

    #[derive(Debug, Clone)]
    pub enum Value {
        Int(u32),
        Str(String),
        Var { name: String, typ: Type },
    }

    pub fn constant(v: Value) -> bool {
        match v {
            Value::Var { .. } => true,
            _ => false,
        }
    }
}

mod fact {
    #[derive(Debug, Clone)]
    pub enum Kind {
        Annotation,
        Analysis,
    }

    #[derive(Debug, Clone)]
    pub struct Type {
        pub name: String,
        pub params: Vec<(String, super::value::Type)>,
        pub kind: Kind,
    }

    #[derive(Debug, Clone)]
    pub struct Fact {
        pub typ: Type,
        pub args: Vec<(String, super::value::Value)>,
    }
}

mod predicate {
    #[derive(Debug)]
    enum Atom {
        Select { selector: String, arg: String },
    }

    #[derive(Debug)]
    enum Relation {
        Eq(Atom, Atom),
        Lt(Atom, Atom),
    }

    type Predicate = Vec<Relation>;
}

mod parser {
    use super::*;
    use chumsky::prelude::*;

    pub fn value_type_parser() -> impl Parser<char, value::Type, Error = Simple<char>> {
        choice((
            just("int").to(value::Type::Int),
            just("str").to(value::Type::Str),
        ))
    }

    pub fn value_parser() -> impl Parser<char, value::Value, Error = Simple<char>> {
        choice((
            text::int(10).map(|s: String| value::Value::Int(s.parse().unwrap())),
            none_of("\"")
                .repeated()
                .collect()
                .delimited_by(just('"'), just('"'))
                .map(|s: String| value::Value::Str(s)),
        ))
    }

    pub fn fact_kind_parser() -> impl Parser<char, fact::Kind, Error = Simple<char>> {
        choice((
            just("annotation").to(fact::Kind::Annotation),
            just("analysis").to(fact::Kind::Analysis),
        ))
    }

    pub fn fact_type_parser() -> impl Parser<char, fact::Type, Error = Simple<char>> {
        filter(char::is_ascii_uppercase)
            .then(
                filter(char::is_ascii_lowercase)
                    .repeated()
                    .collect::<String>(),
            )
            .then(fact_kind_parser)
            .map(|((first, rest), kind)| fact::Type {
                name: format!("{}{}", first, rest),
                params: vec![],
                kind,
            })
    }
}

fn main() {
    let src = std::fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap();

    match value::parser().parse(src) {
        Ok(ast) => println!("{:?}", ast),
        Err(errs) => errs
            .into_iter()
            .for_each(|e| println!("Parse error: {}", e)),
    }
}
