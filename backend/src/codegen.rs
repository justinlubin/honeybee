//! # Code generation
//!
//! This is the backend of the backend! After Programming By Navigation is
//! performed, this module can be used to generate actual code (e.g., Python).

use crate::core::*;
use crate::top_down;

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
    fresh_counter: u32,
}

impl<'a> FullContext<'a> {
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

fn find_decorator(
    decorator_list: &Vec<rustpython_ast::Expr>,
    name: &str,
) -> Option<usize> {
    let id = rustpython_ast::Identifier::new(name);
    for (i, e) in decorator_list.iter().enumerate() {
        let name = match e {
            rustpython_ast::Expr::Call(c) => match &*c.func {
                rustpython_ast::Expr::Name(n) => n,
                _ => continue,
            },
            rustpython_ast::Expr::Name(n) => n,
            _ => continue,
        };
        if name.id == id {
            return Some(i);
        }
    }
    None
}

pub struct Full<'a> {
    library: &'a Library,
}

impl<'a> Full<'a> {
    pub fn new(
        library: &'a Library,
        imp_ast: rustpython_ast::Suite,
    ) -> Result<Self, String> {
        use rustpython_ast as ast;

        let mut preamble = vec![];
        for stmt in imp_ast {
            match stmt {
                ast::Stmt::ImportFrom(if_) => {
                    if if_.module == Some("lib".into()) {
                        continue;
                    } else {
                        preamble.push(ast::Stmt::ImportFrom(if_))
                    }
                }
                ast::Stmt::ClassDef(mut cd) => {
                    if let Some(i) = find_decorator(&cd.decorator_list, "Prop")
                    {
                        cd.decorator_list.remove(i);
                        println!("special prop: {:?}", cd.name);
                    } else if let Some(i) =
                        find_decorator(&cd.decorator_list, "Type")
                    {
                        cd.decorator_list.remove(i);
                        println!("special type: {:?}", cd.name);
                    }
                    preamble.push(ast::Stmt::ClassDef(cd))
                }
                ast::Stmt::FunctionDef(mut fd) => {
                    if let Some(i) =
                        find_decorator(&fd.decorator_list, "Function")
                    {
                        fd.decorator_list.remove(i);
                        println!("special function: {:?}", fd.name);
                    }
                    preamble.push(ast::Stmt::FunctionDef(fd))
                }
                _ => preamble.push(stmt),
            }
        }
        Ok(Full { library })
    }
}

impl<'a> Codegen for Full<'a> {
    fn exp(&self, e: &Exp) -> Result<String, String> {
        let mut ctx = FullContext {
            library: self.library,
            cells: vec![],
            fresh_counter: 0,
        };
        ctx.exp("goal", e);
        Ok(ctx.cells.join("\n\n"))
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
