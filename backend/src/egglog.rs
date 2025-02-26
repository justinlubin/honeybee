use crate::datalog::*;

use chumsky::Parser;
use egglog::EGraph;

// Compiler

struct Compiler {
    content: String,
    tentative: String,
    ruleset: String,
    fresh_counter: u16,
}

impl Compiler {
    pub fn new(ruleset: &str) -> Compiler {
        Compiler {
            content: "".to_owned(),
            tentative: "".to_owned(),
            ruleset: ruleset.to_owned(),
            fresh_counter: 0,
        }
    }

    pub fn get(self) -> String {
        self.content
    }

    // Fundamental

    pub fn write(&mut self, s: &str) {
        self.content += &self.tentative;
        self.cancel();

        self.content += s;
    }

    pub fn tentative(&mut self, t: &str) {
        self.tentative += t;
    }

    pub fn cancel(&mut self) {
        self.tentative = "".to_owned();
    }

    // Helpers

    pub fn newln(&mut self) {
        self.write("\n");
    }

    pub fn writeln(&mut self, s: &str) {
        self.write(s);
        self.newln();
    }

    // Main

    fn value_type(&mut self, vt: &ValueType) {
        match vt {
            ValueType::Bool => self.write("bool"),
            ValueType::Int => self.write("i64"),
            ValueType::Str => self.write("String"),
        }
    }

    fn relation_signature(&mut self, rel: &Relation, sig: &RelationSignature) {
        self.write(&format!("(relation {} (", rel.0));
        for vt in &sig.params {
            self.value_type(vt);
            self.tentative(" ");
        }
        self.cancel();
        self.write("))");
    }

    fn value(&mut self, v: &Value) {
        match v {
            Value::Bool(b) => self.write(&b.to_string()),
            Value::Int(x) => self.write(&x.to_string()),
            Value::Str(s) => self.write(&format!("\"{}\"", s)),
            Value::Var { name, typ: _ } => self.write(name),
        }
    }

    fn constrain(&mut self, v: &Value) {
        match v.unsafe_infer() {
            ValueType::Bool => return,
            ValueType::Int => self.write("(&Int "),
            ValueType::Str => self.write("(&Str "),
        };
        self.value(v);
        self.write(")");
    }

    fn fact(&mut self, f: &Fact) {
        self.write(&format!("({}", f.relation.0));
        for ov in &f.args {
            self.write(" ");
            match ov {
                Some(v) => self.value(v),
                None => {
                    self.write(&format!("_{}", self.fresh_counter));
                    self.fresh_counter += 1;
                }
            }
        }
        self.write(")");
    }

    fn predicate(&mut self, p: &Predicate) {
        match p {
            Predicate::Fact(f) => self.fact(f),
            Predicate::PrimEq(left, right) => {
                self.write("(= ");
                self.value(left);
                self.write(" ");
                self.value(right);
                self.write(")");
            }
            Predicate::PrimLt(left, right) => {
                self.write("(< ");
                self.value(left);
                self.write(" ");
                self.value(right);
                self.write(")");
            }
        }
    }

    fn rule(&mut self, r: &Rule) {
        self.writeln(&format!("; {}", r.name));
        self.write("(rule\n  ; Antecedent\n  (");
        for p in &r.body {
            self.predicate(p);
            self.tentative("\n   ");
        }
        for v in r.vals() {
            self.constrain(&v);
            self.tentative("\n   ");
        }
        self.cancel();
        self.write(")\n  ; Consequent\n  (");
        self.fact(&r.head);
        self.writeln(&format!(
            ")\n  :name \"{}\"\n  :ruleset {})",
            r.name, self.ruleset
        ))
    }

    fn ruleset(&mut self) {
        self.writeln(&format!("(ruleset {})", self.ruleset));
    }

    fn saturate(&mut self) {
        self.writeln(&format!("(run-schedule (saturate {}))", self.ruleset));
    }

    fn print(&mut self, rel: &Relation) {
        self.writeln(&format!("(print-function {} {})", rel.0, 10_000_000));
    }

    pub fn program(&mut self, prog: &Program) {
        self.ruleset();

        self.newln();

        self.writeln("(relation &Int (i64))");
        self.writeln("(relation &Str (String))");

        self.newln();

        for v in &prog.dom {
            self.constrain(v);
            self.newln();
        }

        self.newln();

        for (rel, sig) in &prog.lib {
            self.relation_signature(rel, sig);
            self.newln();
        }

        self.newln();

        for r in &prog.rules {
            self.rule(r);
            self.newln();
        }

        for f in &prog.ground_facts {
            self.fact(f);
            self.newln();
        }

        self.newln();
        self.saturate();
    }
}

// Parser

mod parse {
    use crate::datalog::*;
    use chumsky::prelude::*;

    fn value() -> impl Parser<char, Value, Error = Simple<char>> {
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

    fn entry(
        rel: &Relation,
    ) -> impl Parser<char, Vec<Value>, Error = Simple<char>> {
        just(rel.0.clone())
            .ignored()
            .then(
                just(' ')
                    .repeated()
                    .at_least(1)
                    .then(value())
                    .map(|(_, x)| x)
                    .repeated(),
            )
            .then(just(' ').repeated().ignored())
            .delimited_by(just('('), just(')'))
            .map(|((_, vals), _)| vals)
    }

    pub fn output(
        rel: &Relation,
    ) -> impl Parser<char, Vec<Vec<Value>>, Error = Simple<char>> {
        choice((
            just('(').padded().then(just(')').padded()).to(vec![]),
            entry(rel)
                .padded()
                .repeated()
                .delimited_by(just('('), just(')'))
                .padded(),
        ))
    }
}

#[allow(clippy::large_enum_variant)]
enum State {
    NoCache { egglog_program: Option<String> },
    Cache { egraph: Option<EGraph> },
}

pub struct Egglog {
    state: State,
}

impl Egglog {
    pub fn new(cache: bool) -> Self {
        if cache {
            Self {
                state: State::Cache { egraph: None },
            }
        } else {
            Self {
                state: State::NoCache {
                    egglog_program: None,
                },
            }
        }
    }
}

impl Engine for Egglog {
    fn load(&mut self, program: Program) {
        let mut comp = Compiler::new("program");
        comp.program(&program);
        let egglog_program = comp.get();

        log::debug!("Egglog program constructed\n{}", egglog_program);

        self.state = match self.state {
            State::NoCache { .. } => State::NoCache {
                egglog_program: Some(egglog_program),
            },
            State::Cache { .. } => {
                let mut egraph = EGraph::default();
                let messages = egraph
                    .parse_and_run_program(None, &egglog_program)
                    .unwrap();

                if !messages.is_empty() {
                    panic!("expected 0 messages, got:\n\n{:?}", messages);
                }

                State::Cache {
                    egraph: Some(egraph),
                }
            }
        };
    }

    fn query(
        &mut self,
        signature: &RelationSignature,
        rule: &Rule,
    ) -> Vec<Vec<Value>> {
        let mut comp = Compiler::new("query");
        comp.writeln(";;; Query ;;;\n");
        comp.ruleset();
        comp.newln();
        comp.relation_signature(&rule.head.relation, signature);
        comp.newln();
        comp.newln();
        comp.rule(rule);
        comp.newln();
        comp.saturate();
        comp.print(&rule.head.relation);
        let egglog_query = comp.get();

        log::debug!("Egglog query constructed\n{}", egglog_query);

        let messages = match &mut self.state {
            State::NoCache {
                egglog_program: Some(p),
            } => {
                let combined_program = format!("{}\n{}", p, egglog_query);
                let mut e = EGraph::default();
                e.parse_and_run_program(None, &combined_program)
                    .map_err(|e| {
                        eprintln!("{}", combined_program);
                        panic!("{}", e)
                    })
                    .unwrap()
            }
            State::Cache { egraph: Some(e) } => {
                // TODO: might not need to push/pop here
                e.push();
                let messages =
                    e.parse_and_run_program(None, &egglog_query).unwrap();
                e.pop().unwrap();
                messages
            }
            _ => panic!("must call Engine::load before Engine::query"),
        };

        if messages.len() != 1 {
            panic!("expected 1 message, got:\n\n{:?}", messages);
        }

        let message = messages.into_iter().next().unwrap();

        parse::output(&rule.head.relation).parse(message).unwrap()
    }
}
