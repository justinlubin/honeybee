use crate::ir::*;

mod compile {
    use crate::ir::*;

    pub fn value_type(vt: &ValueType) -> String {
        match vt {
            ValueType::Int => "i64".to_owned(),
            ValueType::Str => "String".to_owned(),
        }
    }

    pub fn fact_signature(fs: &FactSignature) -> String {
        format!(
            "(relation {} ({}))",
            fs.name,
            fs.params
                .iter()
                .map(|(_, vt)| value_type(vt))
                .collect::<Vec<String>>()
                .join(" ")
        )
    }

    pub fn predicate_atom(pa: &PredicateAtom) -> String {
        match pa {
            PredicateAtom::Select { selector, arg } => {
                format!("{}*{}", arg, selector)
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

    pub fn computation_signature(
        lib: &Library,
        cs: &ComputationSignature,
        ret_fact_signature: Option<&FactSignature>,
    ) -> String {
        format!(
            "; {}\n(rule\n  ({}\n   {})\n  (({} {}))\n  :ruleset all)",
            cs.name,
            cs.params
                .iter()
                .map(|(p, fact_name)| format!(
                    "({} {})",
                    fact_name,
                    lib.fact_signature(fact_name)
                        .unwrap()
                        .params
                        .iter()
                        .map(|(pp, _)| format!("{}*{}", p, pp))
                        .collect::<Vec<String>>()
                        .join(" "),
                ))
                .collect::<Vec<String>>()
                .join("\n   "),
            cs.precondition
                .iter()
                .map(predicate_relation)
                .collect::<Vec<String>>()
                .join("\n   "),
            cs.ret,
            (match ret_fact_signature {
                Some(fs) => fs,
                None => lib.fact_signature(&cs.ret).unwrap(),
            })
            .params
            .iter()
            .map(|(p, _)| format!("{}*{}", Query::RET, p))
            .collect::<Vec<String>>()
            .join(" "),
        )
    }

    pub fn value(v: &Value) -> String {
        match v {
            Value::Int(x) => format!("{}", x),
            Value::Str(s) => format!("\"{}\"", s),
        }
    }

    pub fn fact(lib: &Library, f: &Fact) -> String {
        format!(
            "({} {})",
            f.name,
            lib.fact_signature(&f.name)
                .unwrap()
                .params
                .iter()
                .map(|(p, _)| f
                    .args
                    .iter()
                    .find_map(|(a, v)| if a == p {
                        Some(value(v))
                    } else {
                        None
                    })
                    .unwrap())
                .collect::<Vec<String>>()
                .join(" "),
        )
    }

    pub fn query(lib: &Library, q: &Query) -> String {
        format!(
            "{}\n\n{}",
            fact_signature(&q.fact_signature),
            computation_signature(
                lib,
                &q.computation_signature,
                Some(&q.fact_signature)
            ),
        )
    }
}

pub fn compile(lib: &Library, facts: &Vec<Fact>, q: &Query) -> String {
    let mut output = vec![];

    for fs in &lib.fact_signatures {
        output.push(compile::fact_signature(fs));
    }

    output.push("\n(ruleset all)\n".to_owned());

    for cs in &lib.computation_signatures {
        output.push(compile::computation_signature(lib, cs, None))
    }

    output.push("".to_owned());

    for f in facts.iter() {
        output.push(compile::fact(lib, f));
    }

    output.push("".to_owned());

    output.push(compile::query(lib, q));

    output.push("\n(run-schedule (saturate all))\n".to_owned());
    output.push(format!("(print-function {} 1000)", q.fact_signature.name));

    return output.join("\n");
}

mod parse {
    use chumsky::prelude::*;
    use std::collections::HashMap;

    use crate::ir::*;

    pub trait P<T>: Parser<char, T, Error = Simple<char>> {}
    impl<S, T> P<T> for S where S: Parser<char, T, Error = Simple<char>> {}

    fn value() -> impl P<Value> {
        text::int(10).map(|s: String| Value::Int(s.parse().unwrap()))
    }

    fn entry(fs: &FactSignature) -> impl P<Assignment> {
        let fs = fs.clone();
        just(fs.name)
            .ignored()
            .padded()
            .then(value().padded().repeated())
            .delimited_by(just('('), just(')'))
            .map(move |(_, vs)| {
                HashMap::from_iter(
                    fs.params.iter().map(|(n, _)| n.clone()).zip(vs),
                )
            })
    }

    pub fn output(fs: &FactSignature) -> impl P<Vec<Assignment>> {
        entry(fs)
            .padded()
            .repeated()
            .delimited_by(just('('), just(')'))
            .padded()
    }
}

use chumsky::Parser;

pub fn query(lib: &Library, facts: &Vec<Fact>, q: &Query) -> Vec<Assignment> {
    let egglog_src = compile(lib, facts, q);

    log::debug!("Egglog Source:\n{}", egglog_src);

    let mut egraph = egglog::EGraph::default();
    match egraph.parse_and_run_program(&egglog_src) {
        Ok(messages) => {
            if messages.len() != 1 {
                panic!("{:?}", messages)
            }
            let assignments = parse::output(&q.fact_signature)
                .parse(messages[0].clone())
                .unwrap();
            log::debug!("Egglog Assignments:\n{:?}", assignments);
            assignments
        }

        Err(e) => panic!("{}", e),
    }
}

pub fn check_possible(lib: &Library, prog: &Program) -> bool {
    !query(&lib, &prog.annotations, &Query::from_fact(&prog.goal)).is_empty()
}
