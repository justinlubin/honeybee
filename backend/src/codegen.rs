//! # Code generation
//!
//! This is the backend of the backend! After Programming By Navigation is
//! performed, this module can be used to generate actual code (e.g., Python).

use crate::core::*;
use crate::top_down;

// Helpers

fn python_value(v: &Value) -> String {
    match v {
        Value::Bool(true) => "True".to_owned(),
        Value::Bool(false) => "False".to_owned(),
        Value::Int(i) => i.to_string(),
        Value::Str(s) => format!("\"{}\"", s).to_owned(),
    }
}

// Simple Python format (helpful for quickly debugging or getting an overview)

/// Translate an expression into a multi-line Python expression (simple style)
pub fn simple_multi(e: &Exp, current_indent: usize, color: bool) -> String {
    match e {
        top_down::Sketch::Hole(h) => {
            if color {
                top_down::pretty_hole_string(*h)
            } else {
                top_down::plain_hole_string(*h)
            }
        }
        top_down::Sketch::App(f, args) => {
            let new_indent = current_indent + 1;
            format!(
                "{}({}\n{}_metadata={{{}}}\n{})",
                f.name.0,
                args.iter()
                    .map(|(fp, arg)| format!(
                        "\n{}{}={},",
                        "  ".repeat(new_indent),
                        fp.0,
                        simple_multi(arg, new_indent, color)
                    ))
                    .collect::<Vec<_>>()
                    .join(""),
                "  ".repeat(new_indent),
                f.metadata
                    .iter()
                    .map(|(mp, v)| format!("{}={}", mp.0, python_value(v)))
                    .collect::<Vec<_>>()
                    .join(", "),
                "  ".repeat(current_indent)
            )
        }
    }
}

/// Translate an expression into a single-line Python expression (simple style)
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

// Full Python format

struct Context<'a> {
    library: &'a Library,
    cells: Vec<String>,
    fresh_counter: u32,
}

impl<'a> Context<'a> {
    fn fresh_var(&mut self) -> String {
        let s = format!("__x{}", self.fresh_counter);
        self.fresh_counter += 1;
        s
    }

    fn cell(
        &mut self,
        var_name: &str,
        type_name: &str,
        function_name: &str,
        metadata_args: &Vec<(String, String)>,
        args: &Vec<(String, String)>,
    ) {
        let mut s = "".to_owned();
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
        s += "),\n)";
        self.cells.push(s);
    }

    fn exp(&mut self, var_name: &str, e: &Exp) {
        match e {
            top_down::Sketch::Hole(h) => {
                self.cells.push(format!(
                    "{} = {}",
                    var_name,
                    top_down::plain_hole_string(*h)
                ));
            }
            top_down::Sketch::App(f, args) => {
                let f_sig = self.library.functions.get(&f.name).unwrap();
                let mut arg_strings = vec![];
                for (fp, arg) in args {
                    let arg_var = self.fresh_var();
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
                )
            }
        }
    }
}

/// Translate an expression into a Python expression using the full format;
/// returns a list of cells
pub fn python(lib: &Library, e: &Exp) -> Vec<String> {
    let mut ctx = Context {
        library: lib,
        cells: vec![],
        fresh_counter: 0,
    };
    ctx.exp("goal", e);
    ctx.cells
}
