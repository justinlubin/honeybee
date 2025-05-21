//! # Code generation
//!
//! This is the backend of the backend! After Programming By Navigation is
//! performed, this module can be used to generate actual code (e.g., Python).

use crate::core::*;
use crate::top_down;
use indexmap::IndexSet;
use std::collections::HashMap;

////////////////////////////////////////////////////////////////////////////////
// Core types

/// Code generators
pub trait Codegen {
    fn exp(&self, e: &Exp) -> Result<String, String>;
}

////////////////////////////////////////////////////////////////////////////////
// Basic helpers

fn python_value(v: &Value) -> String {
    match v {
        Value::Bool(true) => "True".to_owned(),
        Value::Bool(false) => "False".to_owned(),
        Value::Int(i) => i.to_string(),
        Value::Str(s) => format!("\"{}\"", s).to_owned(),
    }
}

////////////////////////////////////////////////////////////////////////////////
// Full Python style

/// Struct for translating an expression into a Python expression using the
/// full format; returns a list of cells
struct FullContext<'a> {
    library: &'a Library,
    cells: Vec<String>,
    fresh_counter: HashMap<String, usize>,
    used_types: IndexSet<MetName>,
}

impl<'a> FullContext<'a> {
    fn fresh_var(&mut self, prefix: &str) -> String {
        let c = self.fresh_counter.entry(prefix.to_owned()).or_insert(1);
        let s = format!(
            "{}{}",
            prefix,
            if *c > 1 {
                format!("{}", *c)
            } else {
                "".to_owned()
            }
        );
        *c += 1;
        s
    }

    fn cell(
        &mut self,
        var_name: &str,
        type_name: &str,
        function_name: &str,
        metadata_args: &Vec<(String, String)>,
        args: &Vec<(String, String)>,
        title: Option<String>,
        code: Option<String>,
    ) {
        let mut s = "".to_owned();
        s += &format!(
            "# %%{}\n\n",
            match title {
                Some(t) => format!(" {}", t),
                None => "".to_owned(),
            }
        );
        match code {
            Some(code) => s += &format!("{}\n\n", code),
            None => (),
        };
        s += &format!("{} = {}(\n", var_name, type_name);
        s += &format!("    static={}.S(", type_name);
        s += &metadata_args
            .into_iter()
            .map(|(lhs, rhs)| format!("{}={}", lhs, rhs))
            .collect::<Vec<_>>()
            .join(", ");
        s += &format!("),\n    dynamic={}(", function_name);
        s += &args
            .into_iter()
            .map(|(lhs, rhs)| format!("{}={}", lhs, rhs))
            .collect::<Vec<_>>()
            .join(", ");
        s += &format!("),\n)\n{}", var_name);
        self.cells.push(s);
    }

    fn exp(&mut self, var_name: &str, e: &Exp) {
        match e {
            top_down::Sketch::Hole(h) => {
                self.cells.push(format!(
                    "# %%\n\n{} = {}\n{}",
                    var_name,
                    top_down::plain_hole_string(*h),
                    var_name,
                ));
            }
            top_down::Sketch::App(f, args) => {
                let f_sig = self.library.functions.get(&f.name).unwrap();
                self.used_types.insert(f_sig.ret.clone());

                let mut arg_strings = vec![];
                for (fp, arg) in args {
                    let mn = f_sig.params.get(fp).unwrap().clone();
                    let arg_sig = self.library.types.get(&mn).unwrap();
                    let arg_var = self.fresh_var(
                        &arg_sig
                            .info_string("var_name")
                            .unwrap_or(mn.0.to_uppercase().to_owned()),
                    );
                    self.exp(&arg_var, arg);
                    arg_strings.push((fp.0.clone(), arg_var));
                }

                self.cell(
                    var_name,
                    &f_sig.ret.0,
                    &f.name.0,
                    &f.metadata
                        .iter()
                        .map(|(mp, v)| (mp.0.clone(), python_value(v)))
                        .collect(),
                    &arg_strings,
                    f_sig.info_string("overview"),
                    f_sig.info_string("code"),
                )
            }
        }
    }
}

pub struct Full {
    library: Library,
}

impl Full {
    pub fn new(library: Library) -> Result<Self, String> {
        Ok(Full { library })
    }
}

impl Codegen for Full {
    fn exp(&self, e: &Exp) -> Result<String, String> {
        let mut ctx = FullContext {
            library: &self.library,
            cells: vec![],
            fresh_counter: HashMap::new(),
            used_types: IndexSet::new(),
        };
        ctx.exp("GOAL", e);
        let mut s = "".to_owned();
        for t in &ctx.used_types {
            if s.is_empty() {
                s += "# %% Types\n\n";
            }
            match self.library.types.get(t).unwrap().info_string("code") {
                Some(code) => s += &format!("{}\n\n", code),
                None => (),
            }
        }
        s += &ctx.cells.join("\n\n");
        Ok(s)
    }
}

////////////////////////////////////////////////////////////////////////////////
// Simple Python style

/// Translate an expression into a multi-line Python expression (simple style).
/// This is helpful for quickly getting an overview or debugging.
pub struct Simple {
    pub indent: usize,
    pub color: bool,
}

impl Simple {
    fn exp_helper(&self, e: &Exp, indent_offset: usize) -> String {
        match e {
            top_down::Sketch::Hole(h) => {
                if self.color {
                    top_down::pretty_hole_string(*h)
                } else {
                    top_down::plain_hole_string(*h)
                }
            }
            top_down::Sketch::App(f, args) => {
                format!(
                    "{}({}\n{}_metadata={{{}}}\n{})",
                    f.name.0,
                    args.iter()
                        .map(|(fp, arg)| format!(
                            "\n{}{}={},",
                            "  ".repeat(self.indent + indent_offset + 1),
                            fp.0,
                            self.exp_helper(arg, indent_offset + 1)
                        ))
                        .collect::<Vec<_>>()
                        .join(""),
                    "  ".repeat(self.indent + indent_offset + 1),
                    f.metadata
                        .iter()
                        .map(|(mp, v)| format!("{}={}", mp.0, python_value(v)))
                        .collect::<Vec<_>>()
                        .join(", "),
                    "  ".repeat(self.indent + indent_offset)
                )
            }
        }
    }
}

impl Codegen for Simple {
    fn exp(&self, e: &Exp) -> Result<String, String> {
        Ok("  ".repeat(self.indent) + &self.exp_helper(e, 0))
    }
}

/// Translate an expression into a single-line Python expression (simple style).
/// This is an extra goody mostly for CLI purposes (should probably be
/// deprecated and replaced with a better way to display options).
pub fn simple_single(e: &Exp) -> String {
    match e {
        top_down::Sketch::Hole(h) => top_down::pretty_hole_string(*h),
        top_down::Sketch::App(f, args) => {
            format!(
                "{}({}{}_metadata={{{}}})",
                f.name.0,
                args.iter()
                    .map(|(fp, arg)| format!("{}={}", fp.0, simple_single(arg)))
                    .collect::<Vec<_>>()
                    .join(", "),
                if args.is_empty() { "" } else { ", " },
                f.metadata
                    .iter()
                    .map(|(mp, v)| format!("{}={}", mp.0, python_value(v)))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
    }
}
