//! # Parsing Honeybee core syntax
//!
//! Most of the parsing for Honeybee comes from deriving TOML parsing on the
//! data structures. However, for some things (in particular, parsing formulas),
//! we have a custom parser. We also expose some nice functions in this module
//! to abstract away from the fact that we are parsing using TOML.
//!
//! Additionally, we provide deserialization/parsing from JSON for expressions
//! with [`exp`].

use crate::core::*;
use crate::top_down::FunParam;

use chumsky::prelude::*;

// Shorthand

trait P<T>: Parser<char, T, Error = Simple<char>> {}
impl<S, T> P<T> for S where S: Parser<char, T, Error = Simple<char>> {}

// Errors

fn error(title: &str, code: i32, src: &str, err: &Simple<char>) -> String {
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

// Helpers

fn ident_rest() -> impl P<String> {
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

fn lower_ident() -> impl P<String> {
    filter(char::is_ascii_lowercase)
        .then(ident_rest())
        .map(|(first, rest)| format!("{}{}", first, rest))
}

fn upper_ident() -> impl P<String> {
    filter(char::is_ascii_uppercase)
        .then(ident_rest())
        .map(|(first, rest)| format!("{}{}", first, rest))
}

// Main

fn value() -> impl P<Value> {
    choice((
        just("true").to(Value::Bool(true)),
        just("false").to(Value::Bool(false)),
        text::int(10).map(|s: String| Value::Int(s.parse().unwrap())),
        none_of("\"")
            .repeated()
            .collect()
            .delimited_by(just('"'), just('"'))
            .map(|s: String| Value::Str(s)),
    ))
}

fn met_option<T>(rhs: impl P<T>) -> impl P<Met<Option<T>>> {
    upper_ident()
        .then(
            (lower_ident()
                .then(just('=').padded())
                .then(choice((just('_').map(|_| None), rhs.map(|x| Some(x)))))
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

fn formula_atom() -> impl P<FormulaAtom> {
    choice((
        value().map(FormulaAtom::Lit),
        just("ret.")
            .then(lower_ident())
            .padded()
            .map(|(_, mp)| FormulaAtom::Ret(MetParam(mp))),
        lower_ident()
            .then(just('.'))
            .then(lower_ident())
            .padded()
            .map(|((fp, _), mp)| {
                FormulaAtom::Param(FunParam(fp), MetParam(mp))
            }),
    ))
}

fn formula() -> impl P<Formula> {
    #[derive(Clone)]
    enum Op {
        Eq,
        Lt,
    }

    choice((
        formula_atom()
            .then(choice((just('=').to(Op::Eq), just('<').to(Op::Lt))).padded())
            .then(formula_atom())
            .map(|((left, op), right)| match op {
                Op::Eq => Formula::Eq(left, right),
                Op::Lt => Formula::Lt(left, right),
            }),
        met_option(formula_atom()).map(Formula::Ap),
    ))
}

// Special case: convert Formula to/from Vec<String>

impl TryFrom<Vec<String>> for Formula {
    type Error = String;

    fn try_from(strings: Vec<String>) -> Result<Self, Self::Error> {
        let mut overall_phi = Self::True;
        for s in strings {
            match formula().parse(s.clone()) {
                Ok(phi) => {
                    overall_phi =
                        Self::And(Box::new(overall_phi), Box::new(phi))
                }
                Err(errs) => {
                    return Err(error("Formula parse error", 0, &s, &errs[0]))
                }
            }
        }
        Ok(overall_phi)
    }
}

// Top-level functions

/// Parse a library
pub fn library(lib: &str) -> Result<Library, String> {
    toml::from_str(lib).map_err(|e| e.to_string())
}

/// Parse a program
pub fn program(prog: &str) -> Result<Program, String> {
    toml::from_str(prog).map_err(|e| e.to_string())
}

/// Parse an expression
pub fn exp(exp: &str) -> Result<Exp, String> {
    serde_json::from_str(exp).map_err(|e| e.to_string())
}
