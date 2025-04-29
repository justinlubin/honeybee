//! # Code generation
//!
//! This is the backend of the backend! After Programming By Navigation is
//! performed, this module can be used to generate actual code (e.g., Python).

use crate::core::*;
use crate::top_down;

fn python_value(v: &Value) -> String {
    match v {
        Value::Bool(true) => "True".to_owned(),
        Value::Bool(false) => "False".to_owned(),
        Value::Int(i) => i.to_string(),
        Value::Str(s) => format!("\"{}\"", s).to_owned(),
    }
}

/// Translate an expression into a list of Python "cells" (prep for ipynb)
pub fn python_list_multi_wrapper(e: &Exp) -> String {
    // Eventually use the ipynb crate here
    let (_final_var_name, cells) = python_list_multi(e, Vec::new());
    cells.join("\n\n")
}

/// Generate a variable name based on the number of cells so far
fn gen_var_name(cells: &[String]) -> String {
    format!("var{}", cells.len()) // assume 1 assignment per cell
}

fn python_list_multi(e: &Exp, mut cells: Vec<String>) -> (String, Vec<String>) {
    match e {
        top_down::Sketch::Hole(h) => {
            let var_name = gen_var_name(&cells);
            cells.push(format!(
                "{} = {}",
                var_name,
                top_down::pretty_hole_string(*h)
            ));
            (var_name, cells)
        },
        top_down::Sketch::App(f, args) => {
            let mut arg_var_names = Vec::with_capacity(args.len());
            for (_fp, arg) in args.iter() {
                    let (name, new_cells) = python_list_multi(arg, cells);
                    arg_var_names.push(name);
                    cells = new_cells
                };
            // cells length may have changed, so must gen var_name here
            let var_name = gen_var_name(&cells);
            let args_str = args.iter()
                .enumerate()
                .map(|(i, (fp, _arg))| format!(
                    "{}={}, ",
                    fp.0,
                    arg_var_names[i]
                ))
                .collect::<String>();
            let metadata_str = f.metadata.iter()
                .map(|(mp, v)| format!("{}={}", mp.0, python_value(v)))
                .collect::<Vec<_>>()
                .join(", ");
            cells.push(
                format!(
                    "{} = {}({}_metadata={{{}}})",
                    var_name,
                    f.name.0,
                    args_str,
                    metadata_str
                )
            );
            (var_name, cells)
        }
    }
}

/// Translate an expression into a multi-line Python expression
pub fn python_multi(e: &Exp, current_indent: usize) -> String {
    match e {
        top_down::Sketch::Hole(h) => top_down::pretty_hole_string(*h),
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
                        python_multi(arg, new_indent)
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

/// Translate an expression into a single-line Python expression
pub fn python_single(e: &Exp) -> String {
    match e {
        top_down::Sketch::Hole(h) => top_down::pretty_hole_string(*h),
        top_down::Sketch::App(f, args) => {
            format!(
                "{}({}{}_metadata={{{}}})",
                f.name.0,
                args.iter()
                    .map(|(fp, arg)| format!("{}={}", fp.0, python_single(arg)))
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
