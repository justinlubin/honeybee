use std::collections::HashSet;

use crate::ir::*;

struct Compiler {
    ints: HashSet<i64>,
    strings: HashSet<String>,
    vecs: Vec<Vec<Value>>,
}

impl Compiler {
    pub fn new() -> Compiler {
        Compiler {
            ints: HashSet::new(),
            strings: HashSet::new(),
            vecs: vec![],
        }
    }

    pub fn header(&self) -> String {
        let mut output = vec![];

        output.push("(ruleset all)\n".to_owned());

        output.push(";;; Domain compactification\n".to_owned());

        output.push("(relation &Int (i64))".to_owned());
        output.push("".to_owned());
        for int in &self.ints {
            output.push(format!("(rule () ((&Int {})) :ruleset all)", int));
        }

        output.push("(relation &Str (String))".to_owned());
        output.push("".to_owned());
        for string in &self.strings {
            output.push(format!(
                "(rule () ((&Str \"{}\")) :ruleset all)",
                string
            ));
        }

        output.push("(relation &Pointer (i64))".to_owned());
        output.push("".to_owned());
        for i in 0..self.vecs.len() {
            output.push(format!("(rule () ((&Pointer {})) :ruleset all)", i));
        }

        output.push("\n(relation &IntContains (i64 i64))".to_owned());
        output.push("(relation &StrContains (i64 String))\n".to_owned());
        for (i, vec) in self.vecs.iter().enumerate() {
            let mut seen = HashSet::new();
            for val in vec {
                if seen.contains(val) {
                    continue;
                }
                match val {
                    Value::Int(x) => output.push(format!(
                        "(rule () ((&IntContains {} {})) :ruleset all)",
                        i, x
                    )),
                    Value::Str(s) => output.push(format!(
                        "(rule () ((&StrContains {} \"{}\")) :ruleset all)",
                        i, s
                    )),
                    Value::List(_) => panic!("Nested vectors not supported"),
                }
                seen.insert(val);
            }
            output.push("".to_owned());
        }

        output.join("\n")
    }

    pub fn value_type(&self, vt: &ValueType) -> String {
        match vt {
            ValueType::Int => "i64".to_owned(),
            ValueType::Str => "String".to_owned(),
            ValueType::List(_) => "i64".to_owned(), // Pointers
        }
    }

    pub fn fact_signature(&self, fs: &FactSignature) -> String {
        format!(
            "(relation {} ({}))",
            fs.name,
            fs.params
                .iter()
                .map(|(_, vt)| self.value_type(vt))
                .collect::<Vec<String>>()
                .join(" ")
        )
    }

    pub fn predicate_atom(&mut self, pa: &PredicateAtom) -> String {
        match pa {
            PredicateAtom::Select { selector, arg } => {
                format!("{}*{}", arg, selector)
            }
            PredicateAtom::Const(v) => self.value(v),
        }
    }

    pub fn predicate_relation_binop(
        &mut self,
        op: &PredicateRelationBinOp,
        rhs_type: Option<ValueType>,
    ) -> String {
        match op {
            PredicateRelationBinOp::Eq => "=".to_owned(),
            PredicateRelationBinOp::Lt => "<".to_owned(),
            PredicateRelationBinOp::Lte => "<=".to_owned(),
            PredicateRelationBinOp::Contains => match rhs_type {
                Some(ValueType::Int) => "&IntContains".to_owned(),
                Some(ValueType::Str) => "&StrContains".to_owned(),
                _ => panic!("Unknown contains type"),
            },
        }
    }

    pub fn predicate_relation(
        &mut self,
        lib: &Library,
        params: &Vec<(String, FactName, Mode)>,
        pr: &PredicateRelation,
    ) -> String {
        match pr {
            PredicateRelation::BinOp(op, lhs, rhs) => {
                format!(
                    "({} {} {})",
                    self.predicate_relation_binop(op, rhs.typ(lib, params)),
                    self.predicate_atom(lhs),
                    self.predicate_atom(rhs),
                )
            }
        }
    }

    pub fn computation_signature(
        &mut self,
        lib: &Library,
        cs: &ComputationSignature,
        ret_fact_signature: Option<&FactSignature>,
    ) -> String {
        let ret_fact_signature = match ret_fact_signature {
            Some(fs) => fs,
            None => lib.fact_signature(&cs.ret).unwrap(),
        };

        let mut forall_params = HashSet::new();
        let mut int_params = vec![];
        let mut string_params = vec![];
        let mut pointer_params = vec![];

        for (x, f, m) in &cs.params {
            match m {
                Mode::ForAll => {
                    forall_params.insert(x.clone());
                }
                Mode::Exists | Mode::ForAllPlus => {
                    for (selector, vt) in &lib
                        .fact_signature(f)
                        .expect(&format!("Unknown fact name: {}", f))
                        .params
                    {
                        match vt {
                            ValueType::Int => {
                                int_params.push(format!("{}*{}", x, selector))
                            }
                            ValueType::Str => string_params
                                .push(format!("{}*{}", x, selector)),
                            ValueType::List(_) => pointer_params
                                .push(format!("{}*{}", x, selector)),
                        }
                    }
                }
            }
        }

        for (selector, vt) in &ret_fact_signature.params {
            // TODO: duplicate code
            match vt {
                ValueType::Int => int_params.push(format!("ret*{}", selector)),
                ValueType::Str => {
                    string_params.push(format!("ret*{}", selector))
                }
                ValueType::List(_) => {
                    pointer_params.push(format!("ret*{}", selector))
                }
            }
        }

        format!(
            "(rule\n  ({}\n   {})\n  (({} {}))\n  :ruleset all)",
            cs.params
                .iter()
                .filter_map(|(p, fact_name, _)| if forall_params.contains(p) {
                    None
                } else {
                    Some(format!(
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
                })
                .collect::<Vec<String>>()
                .join("\n   "),
            cs.precondition
                .iter()
                .filter_map(|pr| {
                    if pr.free_variables().is_disjoint(&forall_params) {
                        Some(self.predicate_relation(lib, &cs.params, pr))
                    } else {
                        None
                    }
                })
                .chain(int_params.iter().map(|x| format!("(&Int {})", x)))
                .chain(string_params.iter().map(|x| format!("(&Str {})", x)))
                .chain(
                    pointer_params.iter().map(|x| format!("(&Pointer {})", x))
                )
                .collect::<Vec<String>>()
                .join("\n   "),
            cs.ret,
            ret_fact_signature
                .params
                .iter()
                .map(|(p, _)| format!("{}*{}", Query::RET, p))
                .collect::<Vec<String>>()
                .join(" "),
        )
    }

    pub fn value(&mut self, v: &Value) -> String {
        match v {
            Value::Int(x) => {
                self.ints.insert(*x);
                format!("{}", x)
            }
            Value::Str(s) => {
                self.strings.insert(s.clone());
                format!("\"{}\"", s)
            }
            Value::List(args) => {
                self.vecs.push(args.clone());
                format!("{}", self.vecs.len() - 1)
            }
        }
    }

    pub fn fact(&mut self, lib: &Library, f: &Fact) -> String {
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
                        Some(self.value(v))
                    } else {
                        None
                    })
                    .unwrap())
                .collect::<Vec<String>>()
                .join(" "),
        )
    }

    pub fn query(&mut self, lib: &Library, q: &Query) -> String {
        format!(
            "{}\n\n{}",
            self.fact_signature(&q.fact_signature),
            self.computation_signature(
                lib,
                &q.computation_signature,
                Some(&q.fact_signature)
            ),
        )
    }
}

pub fn compile(lib: &Library, facts: &Vec<Fact>, q: &Query) -> String {
    let mut c = Compiler::new();

    let mut body = vec![];

    body.push(";;; Fact signatures\n".to_owned());

    for fs in &lib.fact_signatures {
        body.push(c.fact_signature(fs));
    }

    body.push("\n;;; Computation signatures\n".to_owned());

    let mut rules: std::collections::HashSet<String> =
        std::collections::HashSet::new();
    for cs in &lib.computation_signatures {
        let rule = c.computation_signature(lib, cs, None);
        if rules.contains(&rule) {
            continue;
        }
        body.push(format!("; {}\n{}", cs.name, rule));
        body.push("".to_owned());
        rules.insert(rule);
    }

    for f in facts.iter() {
        body.push(c.fact(lib, f));
    }

    body.push("".to_owned());

    body.push(c.query(lib, q));

    body.push("\n(run-schedule (saturate all))\n".to_owned());
    body.push(format!("(print-function {} 1000)", q.fact_signature.name));

    format!("{}\n{}", c.header(), body.join("\n"))
}

mod parse {
    use chumsky::prelude::*;
    use std::collections::HashMap;

    use crate::ir::*;

    pub trait P<T>: Parser<char, T, Error = Simple<char>> {}
    impl<S, T> P<T> for S where S: Parser<char, T, Error = Simple<char>> {}

    fn value() -> impl P<Value> {
        choice((
            text::int(10).map(|s: String| Value::Int(s.parse().unwrap())),
            none_of("\"")
                .repeated()
                .collect()
                .delimited_by(just('"'), just('"'))
                .map(|s: String| Value::Str(s)),
        ))
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
        choice((
            just('(').padded().then(just(')').padded()).to(vec![]),
            entry(fs)
                .padded()
                .repeated()
                .delimited_by(just('('), just(')'))
                .padded(),
        ))
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
    !query(&lib, &prog.annotations, &Query::from_fact(&prog.goal, "q"))
        .is_empty()
}
