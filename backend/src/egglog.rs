use crate::datalog::*;

use chumsky::Parser;
use egglog::EGraph;

// Compiler

struct Compiler {
    content: String,
    tentative: String,
    ruleset: String,
}

impl Compiler {
    pub fn new(ruleset: &str) -> Compiler {
        Compiler {
            content: "".to_owned(),
            tentative: "".to_owned(),
            ruleset: ruleset.to_owned(),
        }
    }

    pub fn get(self) -> String {
        self.content
    }

    // Fundamental

    fn write(&mut self, s: &str) {
        self.content += &self.tentative;
        self.cancel();

        self.content += s;
    }

    fn tentative(&mut self, t: &str) {
        self.tentative += t;
    }

    fn cancel(&mut self) {
        self.tentative = "".to_owned();
    }

    // Helpers

    fn newln(&mut self) {
        self.write("\n");
    }

    fn writeln(&mut self, s: &str) {
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
    }

    fn value(&mut self, v: &Value) {
        match v {
            Value::Bool(b) => self.write(&b.to_string()),
            Value::Int(x) => self.write(&x.to_string()),
            Value::Str(s) => self.write(&format!("\"{}\"", s)),
            Value::Var { name, typ: _ } => self.write(&name),
        }
    }

    fn fact(&mut self, f: &Fact) {
        self.write(&format!("({}", f.relation.0));
        for v in &f.args {
            self.write(" ");
            self.value(v);
        }
        self.writeln(")");
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
        self.write("(rule\n  (");
        for p in &r.body {
            self.predicate(p);
            self.tentative(" ");
        }
        self.cancel();
        self.write(")\n  (");
        self.fact(&r.head);
        self.writeln(&format!(")\n  :ruleset {})", self.ruleset))
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
            match v {
                Value::Bool(_) => (),
                Value::Int(x) => self.writeln(&format!("(&Int {})", x)),
                Value::Str(s) => self.writeln(&format!("(&Str {})", s)),
                Value::Var { .. } => panic!(),
            };
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

        self.newln();

        for f in &prog.ground_facts {
            self.fact(f);
            self.newln();
        }
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

enum State {
    Uncached { egglog_program: Option<String> },
    Cached { egraph: Option<EGraph> },
}

pub struct Egglog {
    state: State,
}

impl Egglog {
    pub fn new(cache: bool) -> Self {
        if cache {
            Self {
                state: State::Uncached {
                    egglog_program: None,
                },
            }
        } else {
            Self {
                state: State::Cached { egraph: None },
            }
        }
    }
}

impl Engine for Egglog {
    fn load(&mut self, program: Program) {
        let mut comp = Compiler::new("program");
        comp.program(&program);
        let egglog_program = comp.get();

        self.state = match self.state {
            State::Uncached { .. } => State::Uncached {
                egglog_program: Some(egglog_program),
            },
            State::Cached { .. } => {
                let mut egraph = EGraph::default();
                let messages =
                    egraph.parse_and_run_program(&egglog_program).unwrap();

                if messages.len() != 0 {
                    panic!("expected 0 messages, got:\n\n{:?}", messages);
                }

                State::Cached {
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
        comp.ruleset();
        comp.relation_signature(&rule.head.relation, signature);
        comp.rule(rule);
        comp.saturate();
        comp.print(&rule.head.relation);
        let egglog_query = comp.get();

        let messages = match &mut self.state {
            State::Uncached {
                egglog_program: Some(p),
            } => {
                let combined_program = format!("{}\n{}", p, egglog_query);
                let mut e = EGraph::default();
                e.parse_and_run_program(&combined_program).unwrap()
            }
            State::Cached { egraph: Some(e) } => {
                // TODO: might not need to push/pop here
                e.push();
                let messages = e.parse_and_run_program(&egglog_query).unwrap();
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
