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
                format!("{}_{}", arg, selector)
            }
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
                        .map(|(pp, _)| format!("{}_{}", p, pp))
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
            lib.fact_signature(&cs.ret)
                .unwrap()
                .params
                .iter()
                .map(|(p, _)| format!("ret_{}", p))
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

    pub fn expression(e: &Expression) -> String {
        match e {
            Expression::Val(v) => value(v),
            Expression::Var(x) => x.clone(),
        }
    }

    pub fn basic_query(lib: &Library, bq: &BasicQuery) -> String {
        format!(
            "({} {})",
            bq.name,
            lib.fact_signature(&bq.name)
                .unwrap()
                .params
                .iter()
                .map(|(p, _)| bq
                    .args
                    .iter()
                    .find_map(|(a, v)| if a == p {
                        Some(expression(v))
                    } else {
                        None
                    })
                    .unwrap())
                .collect::<Vec<String>>()
                .join(" "),
        )
    }

    pub fn query(lib: &Library, q: &Query) -> String {
        let fvs = q.free_variables(lib);
        format!(
            "(relation *GOAL ({}))\n\n(rule\n  ({})\n  ((*GOAL {}))\n  :ruleset all)",
            fvs.iter()
                .map(|(_, vt)| value_type(vt))
                .collect::<Vec<String>>()
                .join(" "),
            q.entries
                .iter()
                .map(|(_, bq)| basic_query(lib, bq))
                .chain(q.side_condition.iter().map(predicate_relation))
                .collect::<Vec<String>>()
                .join("\n   "),
            fvs.iter()
                .map(|(x, _)| x.clone())
                .collect::<Vec<String>>()
                .join(" "),
        )
    }
}

pub fn compile(lib: &Library, facts: &Vec<Fact>, q: &Query) -> String {
    let mut seen_fact_names = std::collections::HashSet::new();

    let mut output = vec![];

    for fact_name in facts
        .iter()
        .map(|f| &f.name)
        .chain(q.entries.iter().map(|(_, bq)| &bq.name))
    {
        if seen_fact_names.contains(fact_name) {
            continue;
        }
        seen_fact_names.insert(fact_name);
        output.push(compile::fact_signature(
            lib.fact_signature(fact_name).unwrap(),
        ))
    }

    output.push("\n(ruleset all)\n".to_owned());

    for fact_name in seen_fact_names {
        for cs in lib.matching_computation_signatures(fact_name) {
            output.push(compile::computation_signature(lib, cs))
        }
    }

    output.push("".to_owned());

    for f in facts.iter() {
        output.push(compile::fact(lib, f));
    }

    output.push("".to_owned());

    output.push(compile::query(lib, q));

    output.push("\n(run-schedule (saturate all))\n".to_owned());
    output.push(format!("(print-function *GOAL 1000)"));

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

    fn entry(fvs: Vec<String>) -> impl P<Assignment> {
        just("*GOAL")
            .ignored()
            .padded()
            .then(value().padded().repeated())
            .delimited_by(just('('), just(')'))
            .map(move |(_, vs)| {
                HashMap::from_iter(fvs.clone().into_iter().zip(vs))
            })
    }

    pub fn output(fvs: Vec<String>) -> impl P<Vec<Assignment>> {
        entry(fvs)
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
            let assignments = parse::output(
                q.free_variables(lib).into_iter().map(|(x, _)| x).collect(),
            )
            .parse(messages[0].clone())
            .unwrap();
            log::debug!("Egglog Assignments:\n{:?}", assignments);
            assignments
        }

        Err(e) => panic!("{}", e),
    }
}

pub fn check_possible(lib: &Library, prog: &Program) -> bool {
    !query(&lib, &prog.annotations, &prog.goal.to_query()).is_empty()
}
