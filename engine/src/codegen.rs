//! # Code generation
//!
//! This is the backend of the backend! After Programming By Navigation is
//! performed, this module can be used to generate actual code (e.g., Python).

use crate::cellgen;
use crate::core::*;
use crate::top_down;

////////////////////////////////////////////////////////////////////////////////
// Core types

/// Code generators
pub trait Codegen {
    fn exp(&self, e: &Exp) -> Result<String, String>;
}

////////////////////////////////////////////////////////////////////////////////
// Plain-text notebook style

/// Translate an expression into a straight-line list of plain-text cells
pub fn plain_text_notebook(lib: &Library, e: &Exp) -> String {
    let mut ret = "".to_owned();

    let cells = cellgen::exp(lib, e);

    for cell in cells {
        ret += &match cell {
            cellgen::Cell::Code {
                title,
                function_title,
                code,
                ..
            } => format!(
                "# %%{}\n\n{}",
                title
                    .or(function_title)
                    .map(|t| format!(" {}", t))
                    .unwrap_or("".to_owned()),
                code,
            ),
            cellgen::Cell::Hole {
                var_name,
                hole_name,
            } => {
                format!("# %%\n\n{} = ?{}\n{}", var_name, hole_name, var_name)
            }
            cellgen::Cell::Choice { var_name, .. } => {
                format!("# %%\n\n{} = <choice>\n{}", var_name, var_name)
            }
        };

        ret += "\n\n";
    }

    ret.trim().to_owned()
}

pub struct PlainTextNotebook {
    library: Library,
}

impl PlainTextNotebook {
    pub fn new(library: Library) -> Self {
        PlainTextNotebook { library }
    }
}

impl Codegen for PlainTextNotebook {
    fn exp(&self, e: &Exp) -> Result<String, String> {
        Ok(plain_text_notebook(&self.library, &e))
    }
}

////////////////////////////////////////////////////////////////////////////////
// Simple style

/// Translate an expression into a (nested) multi-line Python expression (simple
/// style). This is helpful for quickly getting an overview or debugging.
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
                        .map(|(mp, v)| format!(
                            "{}={}",
                            mp.0,
                            cellgen::python_value(v)
                        ))
                        .collect::<Vec<_>>()
                        .join(", "),
                    "  ".repeat(self.indent + indent_offset)
                )
            }
        }
    }

    /// Translate an expression into a single-line Python expression (simple
    /// style). This is an extra goody mostly for CLI purposes (should probably
    /// be deprecated and replaced with a better way to display options).
    pub fn single(e: &Exp) -> String {
        match e {
            top_down::Sketch::Hole(h) => top_down::pretty_hole_string(*h),
            top_down::Sketch::App(f, args) => {
                format!(
                    "{}({}{}_metadata={{{}}})",
                    f.name.0,
                    args.iter()
                        .map(|(fp, arg)| format!(
                            "{}={}",
                            fp.0,
                            Self::single(arg)
                        ))
                        .collect::<Vec<_>>()
                        .join(", "),
                    if args.is_empty() { "" } else { ", " },
                    f.metadata
                        .iter()
                        .map(|(mp, v)| format!(
                            "{}={}",
                            mp.0,
                            cellgen::python_value(v)
                        ))
                        .collect::<Vec<_>>()
                        .join(", ")
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
