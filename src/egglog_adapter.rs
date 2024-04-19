use crate::ir::*;

fn value_type(vt: &ValueType) -> String {
    match vt {
        ValueType::Int => "i64".to_owned(),
        ValueType::Str => "String".to_owned(),
    }
}

fn fact_signature(fs: &FactSignature) -> String {
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

fn predicate_atom(pa: &PredicateAtom) -> String {
    match pa {
        PredicateAtom::Select { selector, arg } => format!("{}_{}", arg, selector),
    }
}

fn predicate_relation(pr: &PredicateRelation) -> String {
    match pr {
        PredicateRelation::Eq(lhs, rhs) => {
            format!("(= {} {})", predicate_atom(lhs), predicate_atom(rhs))
        }
        PredicateRelation::Lt(lhs, rhs) => {
            format!("(< {} {})", predicate_atom(lhs), predicate_atom(rhs))
        }
    }
}

fn computation_signature(lib: &Library, cs: &ComputationSignature) -> String {
    format!(
        "; {}\n(rule\n  ({}\n   {})\n  (({} {}))\n  :ruleset all)",
        cs.name,
        cs.params
            .iter()
            .map(|(p, fact_name)| format!(
                "({} {})",
                fact_name,
                match lib.lookup(fact_name) {
                    Some(Signature::Fact(fs)) => fs
                        .params
                        .iter()
                        .map(|(pp, _)| format!("{}_{}", p, pp))
                        .collect::<Vec<String>>()
                        .join(" "),
                    _ => panic!(),
                }
            ))
            .collect::<Vec<String>>()
            .join("\n   "),
        cs.precondition
            .iter()
            .map(predicate_relation)
            .collect::<Vec<String>>()
            .join("\n   "),
        cs.ret,
        match lib.lookup(&cs.ret) {
            Some(Signature::Fact(fs)) => fs
                .params
                .iter()
                .map(|(p, _)| format!("ret_{}", p))
                .collect::<Vec<String>>()
                .join(" "),
            _ => panic!(),
        }
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
        match lib.lookup(&f.name) {
            Some(Signature::Fact(fs)) => fs
                .params
                .iter()
                .map(|(p, _)| f
                    .args
                    .iter()
                    .find_map(|(a, v)| if a == p { Some(value(v)) } else { None })
                    .unwrap())
                .collect::<Vec<String>>()
                .join(" "),
            _ => panic!(),
        }
    )
}

pub fn compile(lib: &Library, facts: &Vec<Fact>, query: &Fact) -> String {
    let mut seen_fact_names = std::collections::HashSet::new();

    let mut output = vec![];

    for f in facts.iter().chain(std::iter::once(query)) {
        if seen_fact_names.contains(&f.name) {
            continue;
        }
        match lib.lookup(&f.name) {
            Some(Signature::Fact(fs)) => {
                seen_fact_names.insert(&f.name);
                output.push(fact_signature(fs));
            }
            _ => panic!(),
        }
    }

    output.push("\n(ruleset all)\n".to_owned());

    for fact_name in seen_fact_names {
        for cs in lib.matching_computations(fact_name) {
            output.push(computation_signature(lib, cs))
        }
    }

    output.push("".to_owned());

    for f in facts.iter() {
        output.push(fact(lib, f));
    }

    output.push("\n(run-schedule (saturate all))\n".to_owned());

    output.push(format!("(check {})", fact(lib, query)));

    return output.join("\n");
}

pub fn check(lib: &Library, facts: &Vec<Fact>, query: &Fact) -> bool {
    let egglog_src = compile(lib, facts, query);

    log::debug!("Egglog Source:\n{}", egglog_src);

    let mut egraph = egglog::EGraph::default();
    match egraph.parse_and_run_program(&egglog_src) {
        Ok(messages) => {
            assert!(messages.is_empty());
            true
        }
        Err(e) => match e {
            egglog::Error::CheckError(_) => false,
            _ => panic!("{}", e),
        },
    }
}

pub fn check_program(lib: &Library, prog: &Program) -> bool {
    check(&lib, &prog.annotations, &prog.goal)
}
