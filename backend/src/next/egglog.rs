use crate::next::datalog::*;

use egglog::EGraph;

struct Egglog {
    cache: bool,
    prog: Option<Program>,
    egraph: Option<EGraph>,
}

impl Egglog {
    pub fn new(cache: bool) -> Self {
        Self {
            cache,
            prog: None,
            egraph: None,
        }
    }
}

struct Compiler {
    content: String,
    tentative: String,
}

impl Compiler {
    pub fn new() -> Compiler {
        Compiler {
            content: "".to_owned(),
            tentative: "".to_owned(),
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
            ValueType::Int => self.write("i64"),
            ValueType::Str => self.write("i64"),
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

    fn rule(&mut self, r: &Rule) {
        self.writeln(&format!("; {}", r.name));
        self.write("(rule (");
        todo!();
    }

    pub fn program(&mut self, prog: &Program) {
        self.writeln("(relation &Int (i64))");
        self.writeln("(relation &Str (String))");

        self.newln();

        for v in &prog.dom {
            match v {
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

fn compile(prog: &Program) -> String {
    let mut comp = Compiler::new();
    comp.program(prog);
    comp.get()
}

impl Engine for Egglog {
    fn load(&mut self, program: Program) {
        self.egraph = if self.cache {
            let egglog_program = compile(&program);
            let mut egraph = EGraph::default();
            egraph.parse_and_run_program(&egglog_program);
            Some(egraph)
        } else {
            None
        };

        self.prog = Some(program);
    }

    fn query(
        &mut self,
        query_signature: RelationSignature,
        query_rule: Rule,
    ) -> Vec<Vec<Value>> {
        todo!()
    }
}
